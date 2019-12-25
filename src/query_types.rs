use serde::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstantQuery {
    /// PromQL Query which will be sent to API
    pub query: String,
    /// Evaluation timestamp in unix timestamp format
    pub time: Option<i64>,
    /// Timeout duration for evaluating the result
    pub timeout: Option<String>,
}

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesRequest {
    /// List of series selectors
    #[serde(rename(serialize = "match[]"))]
    pub selectors: Vec<String>,
    /// Start timestamp for the range query
    pub start: Option<i64>,
    /// End timestamp for the range query
    pub end: Option<i64>,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TargetsWithStatesRequest {
    /// Requested target state filter
    pub state: ProqTargetStates,
}
