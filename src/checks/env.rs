type CommandVecType = Vec<String>;

pub fn exec_command(cmd: &str) -> CommandVecType {
    if cmd.is_empty() {
        panic!("EXEC_COMMAND is empty!")
    }

    let command_list = serde_json::from_str::<CommandVecType>(cmd);
    match command_list {
        Ok(vec) => {
            if vec.is_empty() {
                panic!("EXEC_COMMAND got an empty json parse result!")
            }
            vec
        },
        Err(e) => panic!(e)
    }
}