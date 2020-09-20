use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct TestResult {
    case_id: i64,
    status: String,
    time_used: u32,
    space_used: u32,
}
