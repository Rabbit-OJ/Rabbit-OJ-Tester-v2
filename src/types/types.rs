use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct TestResult {
    pub case_id: u32,
    pub status: &'static str,
    pub time_used: u32,
    pub space_used: u32,
}
