use std::env;
use std::process::{Child, Command};
use std::sync::{Arc, RwLock};
use sysinfo::{ProcessExt, SystemExt};
use std::time::Duration;
use futures::{future};
use futures::future::FutureExt;
use nix::sys::signal::{self, Signal};
use nix::unistd::{Pid};

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
    let space_limit = env::var("SPACE_LIMIT").unwrap().parse::<u64>().unwrap() * 1024;
    let exec_command = env::var("EXEC_COMMAND").unwrap();
    let mut dev_mode = false;
    if let Ok(_) = env::var("DEV") {
        dev_mode = true;
    }

    if dev_mode {
        println!("[DEV] ENV CASE_COUNT = {}", case_count);
        println!("[DEV] ENV TIME_LIMIT = {}", time_limit);
        println!("[DEV] ENV SPACE_LIMIT = {}", space_limit);
        println!("[DEV] ENV EXEC_COMMAND = {}", exec_command);
    }

    let test_result = start_test(case_count, time_limit, space_limit, exec_command, dev_mode).await;

    let test_result_json_str = serde_json::to_string(&test_result).unwrap();
    write_result_file(test_result_json_str).unwrap();
    std::process::exit(0);
}

async fn start_test(case_count: u32, time_limit: u64, space_limit: u64, exec_command: String, dev_mode: bool)
                    -> Vec<TestResult<'static>> {
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

        let result = test_one(command, i, time_limit, space_limit, dev_mode).await;
        test_result.push(result);
    }

    test_result
}

async fn test_one(mut command: Command, index: u32, time_limit: u64, space_limit: u64, dev_mode: bool) -> TestResult<'static> {
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

    // let mut setpgid_success = true;
    // if let Err(e) = setpgid(Pid::from_raw(child_pid as i32), Pid::from_raw(child_pid as i32)) {
    //     println!("[WARNING] setpgid syscall executed error due to {}, child_pid = {}", e, child_pid);
    //     setpgid_success = false;
    // }

    let main_future = main_future(child_process).boxed();
    let memory_future = memory_watch_future(child_pid as i32, space_limit, peak_memory.clone()).boxed();
    let time_future = time_watch_future(time_limit).boxed();

    let (mut status, _, future_list) = future::select_all(vec![main_future, memory_future, time_future]).await;
    if status == consts::STATUS_CONTINUE {
        status = future::select_all(future_list).await.0;
    }

    if dev_mode {
        println!("[DEV] case = {}, status = {}", index, status);
    }
    let end_time = time::now_unix();

    let space_used = peak_memory.read().unwrap();
    result.space_used = *space_used;
    drop(space_used);

    result.time_used = (end_time - start_time) as u64;
    result.status = status;
    if result.status != consts::STATUS_OK {
        if let Err(e) = signal::killpg(Pid::from_raw(child_pid as i32), Signal::SIGKILL) {
            println!("[WARNING] Error when sending SIGKILL to process group {}, {}", child_pid, e)
        }

        // if setpgid_success {
        //     if let Err(e) = signal::killpg(Pid::from_raw(child_pid as i32), Signal::SIGKILL) {
        //         println!("[WARNING] Error when sending SIGKILL to process group {}, {}", child_pid, e)
        //     }
        // } else { // fallback
        //     if let Err(e) = signal::kill(Pid::from_raw(child_pid as i32), Signal::SIGKILL) {
        //         println!("[WARNING] Error when sending SIGKILL to process {}, {}", child_pid, e)
        //     }
        // }
    }

    result
}

async fn main_future(mut child_process: Child) -> &'static str {
    let output = tokio::task::spawn_blocking(move || {
        return child_process.wait();
    }).await.unwrap();

    if let Ok(status) = output {
        if status.success() {
            return consts::STATUS_OK;
        }
    }

    return consts::STATUS_RE;
}

async fn time_watch_future(time_limit: u64) -> &'static str {
    tokio::time::delay_for(Duration::from_millis(time_limit)).await;
    return consts::STATUS_TLE;
}

async fn memory_watch_future(pid: i32, memory_limit: u64, peak_memory: Arc<RwLock<u64>>) -> &'static str {
    let refresh_kind = sysinfo::RefreshKind::new().with_memory();
    let mut system = sysinfo::System::new_with_specifics(refresh_kind);

    loop {
        system.refresh_all();
        let memory_process_result = system.get_process(pid);
        match memory_process_result {
            Some(memory_usage) => {
                let current_memory = memory_usage.memory();

                let mut memory = peak_memory.write().unwrap();
                *memory = max(current_memory, *memory);
                drop(memory);

                if current_memory > memory_limit {
                    return consts::STATUS_MLE;
                }
            }
            None => {
                println!("[INFO] (memory watch) Process exited.");
                return consts::STATUS_CONTINUE;
            }
        }

        tokio::time::delay_for(Duration::from_millis(50)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_executor(filename: &str, eq_status: &'static str) {
        let exec_command = format!("[\"./test/{}.o\"]", filename);

        env::set_var("DEV", "1");
        let test_result = start_test(2, 1000, 128 * 1024,
                                     exec_command, true).await;

        assert_eq!(test_result.len(), 2);
        for one_case in test_result.iter() {
            assert_eq!(one_case.status, eq_status);
        }
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_tle() {
        test_executor("tle", consts::STATUS_TLE).await;
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_mle() {
        test_executor("mle", consts::STATUS_MLE).await;
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_ok() {
        test_executor("ok", consts::STATUS_OK).await;
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_re() {
        test_executor("re", consts::STATUS_RE).await;
    }
}