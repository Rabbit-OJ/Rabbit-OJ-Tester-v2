use std::env;
use std::process::{Child, Command};
use std::sync::{Arc, RwLock};
use sysinfo::{ProcessExt, SystemExt};
use std::time::Duration;
use futures::{future};
use futures::future::FutureExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::{Pid, setpgid};

use types::TestResult;
use utils::{consts, file, time};
use utils::file::{create_stdout_file, open_stdin_file, write_result_file};
use std::cmp::max;

mod utils;
mod checks;
mod types;

#[tokio::main]
async fn main() {
    let case_count = env::var("CASE_COUNT").unwrap().parse::<u32>().unwrap();
    let time_limit = env::var("TIME_LIMIT").unwrap().parse::<u64>().unwrap();
    let space_limit = env::var("SPACE_LIMIT").unwrap().parse::<u64>().unwrap();
    let exec_command = env::var("EXEC_COMMAND").unwrap();

    checks::test_cases::count(case_count);
    let exec_command_vec = checks::env::exec_command(exec_command.as_str());

    let exec_command_name = &exec_command_vec[0];
    let exec_args = &exec_command_vec[1..];

    if exec_command_vec.len() == 1 {
        file::chmod(exec_command_name.as_str(), 0o755);
    }

    let mut test_result: Vec<TestResult<'static>> = vec![];
    for i in 1..(case_count + 1) {
        println!("Testing Case #{} ...", i);
        let mut command = Command::new(exec_command_name);
        command.args(exec_args);

        let result = test_one(command, i, time_limit, space_limit).await;
        test_result.push(result);
    }

    let test_result_json_str = serde_json::to_string(&test_result).unwrap();
    write_result_file(test_result_json_str).unwrap();
    std::process::exit(0);
}

async fn test_one(mut command: Command, index: u32, time_limit: u64, space_limit: u64) -> TestResult<'static> {
    let mut result = TestResult {
        case_id: index,
        status: consts::STATUS_OK,
        time_used: 0,
        space_used: 0,
    };

    let peak_memory: Arc<RwLock<u64>> = Arc::new(RwLock::new(0));
    let child_process: Child;

    let stdin_file = open_stdin_file(index).unwrap();
    let stdout_file = create_stdout_file(index).unwrap();
    command.stdin(stdin_file);
    command.stdout(stdout_file);

    match command.spawn() {
        Ok(cmd) => {
            child_process = cmd;
        }
        Err(e) => {
            println!("{}", e);
            result.status = consts::STATUS_RE;

            return result;
        }
    }

    let start_time = time::now_unix();
    let child_pid = child_process.id();

    let mut setpgid_success = true;
    if let Err(e) = setpgid(Pid::from_raw(child_pid as i32), Pid::from_raw(child_pid as i32)) {
        println!("[WARNING] setpgid syscall executed error due to {}, child_pid = {}", e, child_pid);
        setpgid_success = false;
    }

    let main_future = main_thread(child_process).boxed();
    let memory_future = memory_watch_thread(child_pid as i32, space_limit, peak_memory.clone()).boxed();
    let time_future = time_watch_thread(time_limit).boxed();

    let status = future::select_all(vec![main_future, memory_future, time_future]).await.0;
    let end_time = time::now_unix();

    let space_used = peak_memory.read().unwrap();
    result.space_used = *space_used;

    result.time_used = (end_time - start_time) as u64;
    result.status = status;
    if result.status != consts::STATUS_OK {
        if setpgid_success {
            if let Err(e) = signal::killpg(Pid::from_raw(child_pid as i32), Signal::SIGKILL) {
                println!("[WARNING] Error when sending SIGKILL to process group {}, {}", child_pid, e)
            }
        } else { // fallback
            if let Err(e) = signal::kill(Pid::from_raw(child_pid as i32), Signal::SIGKILL) {
                println!("[WARNING] Error when sending SIGKILL to process {}, {}", child_pid, e)
            }
        }
    }

    result
}

async fn main_thread(mut child_process: Child) -> &'static str {
    let output = child_process.wait();
    if let Ok(status) = output {
        if status.success() {
            return consts::STATUS_OK;
        }
    }

    return consts::STATUS_RE;
}

async fn time_watch_thread(time_limit: u64) -> &'static str {
    tokio::time::delay_for(Duration::from_millis(time_limit)).await;
    consts::STATUS_TLE
}

async fn memory_watch_thread(pid: i32, memory_limit: u64, peak_memory: Arc<RwLock<u64>>) -> &'static str {
    let refresh_kind = sysinfo::RefreshKind::new().with_memory();
    let mut system = sysinfo::System::new_with_specifics(refresh_kind);

    loop {
        system.refresh_all();
        let memory_process_result = system.get_process(pid);
        if let Some(memory_usage) = memory_process_result {
            let current_memory = memory_usage.memory();

            let mut memory = peak_memory.write().unwrap();
            *memory = max(current_memory, *memory);
            drop(memory);

            if current_memory > memory_limit {
                return consts::STATUS_MLE;
            }
        }
    }
}
