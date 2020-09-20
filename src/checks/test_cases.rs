use super::super::utils::{file, path};

pub fn count(case_count: u32) {
    if case_count <= 0 {
        panic!("Should have at least one test case count!");
    }

    for i in 1..case_count {
        let case_path = path::docker_case_path(i);
        if !file::exist(case_path.as_str()) {
            panic!(format!("Test case {} doesn't exist", i));
        }
    }
}
