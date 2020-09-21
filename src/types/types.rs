use serde::Serialize;

#[derive(Serialize)]
pub struct TestResult<'a> {
    pub case_id: u32,
    pub status: &'a str,
    pub time_used: u64,
    pub space_used: u64,
}
