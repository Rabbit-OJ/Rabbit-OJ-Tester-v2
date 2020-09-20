use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

pub fn exist(path: &str) -> bool {
    let metadata = fs::metadata(path);

    metadata.is_ok()
}

pub fn chmod(path: &str, mode: u32) {
    let metadata = fs::metadata(path).unwrap();
    let mut permission = metadata.permissions();
    permission.set_mode(mode);
}
