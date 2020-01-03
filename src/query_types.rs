//!
//! Request types that are sent by the Proq to different endpoints.
use serde::*;

///
/// Instant query request struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstantQuery {
    /// PromQL Query which will be sent to API
    pub query: String,
    /// Evaluation timestamp in unix timestamp format
    pub time: Option<i64>,
    /// Timeout duration for evaluating the result
    pub timeout: Option<String>,
}

///
/// Range query request struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RangeQuery {
    /// PromQL Query which will be sent to API
    pub query: String,
    /// Start timestamp for the range query
    pub start: Option<i64>,
    /// End timestamp for the range query
    pub end: Option<i64>,
    /// Step as duration in the range in seconds as 64-bit floating point format
    pub step: Option<f64>,
    /// Timeout duration for evaluating the result
    pub timeout: Option<String>,
}

///
/// Series query request struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesRequest {
    /// List of series selectors
    #[serde(rename(serialize = "match[]"))]
    pub selectors: Vec<String>,
    /// Start timestamp for the range query
    pub start: Option<i64>,
    /// End timestamp for the range query
    pub end: Option<i64>,
    /// Timeout duration for evaluating the result
    pub timeout: Option<String>,
}

///
/// Possible Prometheus target states.
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProqTargetStates {
    /// Target state filtered by Active state
    ACTIVE,
    /// Target state filtered by Dropped state
    DROPPED,
    /// Target state without any filtering
    ANY,
}

///
/// Target with filtered state request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TargetsWithStatesRequest {
    /// Requested target state filter
    pub state: ProqTargetStates,
}

///
/// Possible Prometheus rule types.
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProqRulesType {
    /// Rule type filtered by Alert
    ALERT,
    /// Rule type filtered by Record
    RECORD,
}

///
/// Rules with filtered state request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RulesWithTypeRequest {
    /// Requested target state filter
    #[serde(rename = "type")]
    pub rule_type: ProqRulesType,
}
