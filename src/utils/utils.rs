use std::fs;

pub fn file_exist(path: &str) -> bool {
    let metadata = fs::metadata(path);

    metadata.is_ok()
}
