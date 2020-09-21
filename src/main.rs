use std::{env, thread};
use std::process;
use std::process::{Child, Command};
use nix::unistd::{Pid, setpgid};

mod utils;
mod checks;
mod types;

use types::TestResult;
use utils::{consts, file, time};
use utils::file::{create_stdout_file, open_stdin_file, write_result_file};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::sync::mpsc::Sender;
use sysinfo::{ProcessExt, SystemExt};
use std::time::Duration;

fn main() {
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

        let result = test_one(command, i, time_limit, space_limit);
        test_result.push(result);
    }

    let test_result_json_str = serde_json::to_string(&test_result).unwrap();
    write_result_file(test_result_json_str).unwrap();
    std::process::exit(0);
}

fn test_one(mut command: Command, index: u32, time_limit: u64, space_limit: u64) -> TestResult<'static> {
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

    let current_pid = process::id();
    if let Err(e) = setpgid(Pid::from_raw(child_pid as i32), Pid::from_raw(current_pid as i32)) {
        println!("[WARNING] setpgid syscall executed error due to {}, child_pid = {}, current_pid = {}", e, child_pid, current_pid);
    }

    let (tx, rx) = mpsc::channel::<&'static str>();
    let ctx = Arc::new(Mutex::new(true));

    main_thread(child_process, tx.clone(), ctx.clone());
    memory_watch_thread(child_pid as i32, space_limit, tx.clone(), peak_memory.clone(), ctx.clone());
    time_watch_thread(time_limit, tx.clone(), ctx.clone());

    let msg = rx.recv().unwrap();
    let end_time = time::now_unix();

    let space_used = peak_memory.read().unwrap();
    result.space_used = *space_used;
    result.time_used = (end_time - start_time) as u64;
    result.status = msg;

    result
}

fn main_thread(child_process: Child, tx: Sender<&'static str>, ctx: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        let output = child_process.wait_with_output();

        let mut locked_state = ctx.lock().unwrap();
        if *locked_state == true {
            *locked_state = false;
            drop(locked_state);

            if let Ok(info) = output {
                if info.status.success() {
                    tx.send(consts::STATUS_OK);
                } else {
                    tx.send(consts::STATUS_RE);
                }
            } else {
                tx.send(consts::STATUS_RE);
            }
        }
    });
}

fn time_watch_thread(time_limit: u64, tx: Sender<&'static str>, ctx: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(time_limit));

        let mut locked_state = ctx.lock().unwrap();
        if *locked_state == true {
            *locked_state = false;
            drop(locked_state);

            tx.send(consts::STATUS_TLE);
        }
    });
}

fn memory_watch_thread(pid: i32, memory_limit: u64, status_tx: Sender<&'static str>, peak_memory: Arc<RwLock<u64>>, ctx: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        let refresh_kind = sysinfo::RefreshKind::new().with_memory();
        let mut system = sysinfo::System::new_with_specifics(refresh_kind);

        loop {
            {
                let locked_state = ctx.lock().unwrap();
                if *locked_state == false {
                    return;
                }
            }

            system.refresh_all();
            let memory_process_result = system.get_process(pid);
            if let Some(memory_usage) = memory_process_result {
                let current_memory = memory_usage.memory();

                match peak_memory.try_write() {
                    Ok(mut guard) => {
                        *guard = current_memory;
                        drop(guard);
                    }
                    _ => {}
                }

                if current_memory > memory_limit {
                    let mut locked_state = ctx.lock().unwrap();
                    if *locked_state == true {
                        *locked_state = false;
                        drop(locked_state);

                        status_tx.send(consts::STATUS_MLE);
                    }
                    return;
                }
            }
        }
    });
}
