use std::env;

fn main() {
    let case_count = env::var("CASE_COUNT").unwrap().parse::<u32>().unwrap();
    let time_limit = env::var("TIME_LIMIT").unwrap().parse::<u32>().unwrap();
    let space_limit = env::var("SPACE_LIMIT").unwrap().parse::<u32>().unwrap();
    let exec_command = env::var("EXEC_COMMAND").unwrap();

    if case_count <= 0 {
        panic!("Should have at least one test case count!");
    }

    println!("Hello, world!");
}
