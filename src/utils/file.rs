use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use super::path;

pub fn exist(path: &str) -> bool {
    let metadata = fs::metadata(path);

    metadata.is_ok()
}

pub fn chmod(path: &str, mode: u32) {
    let metadata = fs::metadata(path).unwrap();
    let mut permission = metadata.permissions();
    permission.set_mode(mode);
}

pub fn write_result_file(json_str: String) -> io::Result<()> {
    let result_file_path = path::docker_result_file();
    let mut buffer = File::create(result_file_path)?;
    buffer.write_all(json_str.as_bytes())?;
    buffer.flush()?;
    Ok(())
}

pub fn create_stdout_file(index: u32) -> io::Result<File> {
    let stdout_file_path = path::docker_output_path(index);
    let file = File::create(stdout_file_path)?;

    Ok(file)
}

pub fn open_stdin_file(index: u32) -> io::Result<File> {
    let stdin_file_path = path::docker_case_path(index);
    let file = File::open(stdin_file_path)?;

    Ok(file)
}
