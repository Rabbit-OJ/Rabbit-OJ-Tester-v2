use std::env;

fn is_dev_mode() -> bool {
    let mut dev_mode = false;
    if let Ok(_) = env::var("DEV") {
        dev_mode = true;
    }

    dev_mode
}

pub fn docker_case_path(case_id: u32) -> String {
    if is_dev_mode() {
        format!("./case/{}.in", case_id)
    } else {
        format!("/case/{}.in", case_id)
    }
}

pub fn docker_output_path(case_id: u32) -> String {
    if is_dev_mode() {
        format!("./output/{}.out", case_id)
    } else {
        format!("/output/{}.out", case_id)
    }
}

pub fn docker_result_file() -> String {
    if is_dev_mode() {
        String::from("./result/info.json")
    } else {
        String::from("/result/info.json")
    }
}
