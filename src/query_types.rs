use serde::*;

#[derive(Serialize, Deserialize)]
pub struct InstantQuery {
    /// PromQL Query which will be sent to API
    pub query: String,
    /// Evaluation timestamp in unix timestamp format
    pub time: Option<i64>,
    /// Timeout duration for evaluating the result
    pub timeout: Option<String>,
}
