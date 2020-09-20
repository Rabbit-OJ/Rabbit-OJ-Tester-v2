use serde::{Serialize};
use serde_json::Result;

#[derive(Serialize)]
pub struct TestResult<'a> {
    pub case_id: u32,
    pub status: &'a str,
    pub time_used: u32,
    pub space_used: u32,
}
