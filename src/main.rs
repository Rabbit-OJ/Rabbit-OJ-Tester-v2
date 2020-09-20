use std::env;
use std::fs::File;
use std::io;
use std::io::Write;

mod utils;
mod checks;
mod types;

use types::{TestResult};
use utils::{file, path, consts};

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
        let result = test_one(exec_command_name, exec_args, i, time_limit, space_limit);
        test_result.push(result);
    }

    let test_result_json_str = serde_json::to_string(&test_result).unwrap();
    write_result_file(test_result_json_str).unwrap();
    std::process::exit(0);
}

fn write_result_file(json_str: String) -> io::Result<()> {
    let result_file_path = path::docker_result_file();
    let mut buffer = File::create(result_file_path)?;
    buffer.write_all(json_str.as_bytes())?;
    buffer.flush()?;
    Ok(())
}

fn test_one(exec_command: &String, exec_args: &[String],
            index: u32, time_limit: u32, space_limit: u32) -> TestResult<'static> {
    let result = TestResult {
        case_id: index,
        status: consts::STATUS_OK,
        time_used: 0,
        space_used: 0
    };

    result
}
