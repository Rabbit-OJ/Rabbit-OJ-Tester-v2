use std::env;
use std::io::Write;
use std::process::{Child, Command};

use nix::unistd::{Pid, setpgid};

use types::TestResult;
use utils::{consts, file};
use utils::file::{create_stdout_file, open_stdin_file, write_result_file};

mod utils;
mod checks;
mod types;

fn main() {
    let case_count = env::var("CASE_COUNT").unwrap().parse::<u32>().unwrap();
    let time_limit = env::var("TIME_LIMIT").unwrap().parse::<u32>().unwrap();
    let space_limit = env::var("SPACE_LIMIT").unwrap().parse::<u32>().unwrap();
    let exec_command = env::var("EXEC_COMMAND").unwrap();

    checks::test_cases::count(case_count);
    let exec_command_vec = checks::env::exec_command(exec_command.as_str());

    let exec_command_name = &exec_command_vec[0];
    let exec_args = &exec_command_vec[1..];

    if exec_command_vec.len() == 1 {
        file::chmod(exec_command_name.as_str(), 0o755);
    }

    let mut test_result: Vec<TestResult<'static>> = vec![];
    for i in 1..case_count {
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

fn test_one(mut command: Command, index: u32, time_limit: u32, space_limit: u32) -> TestResult<'static> {
    let mut result = TestResult {
        case_id: index,
        status: consts::STATUS_OK,
        time_used: 0,
        space_used: 0,
    };

    let mut peak_memory: u64 = 0;
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

    let child_pid = child_process.id();
    if let Err(e) = setpgid(Pid::from_raw(child_pid as i32), Pid::from_raw(child_pid as i32)) {
        println!("[WARNING] setpgid syscall executed error due to {}", e);
    }

    let output = child_process.wait_with_output();
    if let Ok(info) = output {
        if info.status.success() {
            result.status = consts::STATUS_OK;
        } else {
            result.status = consts::STATUS_RE;
        }
    }

    result.space_used = peak_memory;
    result
}
