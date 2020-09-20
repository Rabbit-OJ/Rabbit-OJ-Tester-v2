pub fn docker_case_path(case_id: u32) -> String {
    format!("/case/{}.in", case_id)
}

pub fn docker_output_path(case_id: u32) -> String {
    format!("/output/{}.out", case_id)
}

pub fn docker_result_file() -> String {
    String::from("/result/info.json")
}
