use sysinfo::{ProcessExt, SystemExt};

fn get_memory(pid: i32) -> Option<u64> {
    let refresh_kind = sysinfo::RefreshKind::new().with_memory();

    let mut system = sysinfo::System::new_with_specifics(refresh_kind);
    system.refresh_all();
    let process = system.get_process(pid)?;

    Some(process.memory())
}