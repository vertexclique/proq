//!
//! Proq client API
//!
//! This module provides Prometheus Query API related methods.

use std::str::FromStr;
use std::time::Duration;

use ::url::Url;
use chrono::offset::Utc;
use chrono::DateTime;
use http::{uri, Uri};
use serde::Serialize;
use surf::*;

use crate::query_types::*;
use crate::result_types::ApiResult;

use super::errors::*;

const PROQ_INSTANT_QUERY_URL: &str = "/api/v1/query";
const PROQ_RANGE_QUERY_URL: &str = "/api/v1/query_range";
const PROQ_SERIES_URL: &str = "/api/v1/series";
const PROQ_LABELS_URL: &str = "/api/v1/labels";
const PROQ_TARGETS_URL: &str = "/api/v1/targets";
const PROQ_RULES_URL: &str = "/api/v1/rules";
const PROQ_ALERTS_URL: &str = "/api/v1/alerts";
const PROQ_ALERT_MANAGERS_URL: &str = "/api/v1/alertmanagers";
const PROQ_STATUS_CONFIG_URL: &str = "/api/v1/status/config";
const PROQ_STATUS_FLAGS_URL: &str = "/api/v1/status/config";
macro_rules! PROQ_LABEL_VALUES_URL {
    () => {
        "/api/v1/label/{}/values"
    };
}

///
/// Protocol type for the client
#[derive(PartialEq)]
pub enum ProqProtocol {
    /// HTTP transport
    HTTP,
    /// HTTPS transport
    HTTPS,
}

///
/// Main client structure.
pub struct ProqClient {
    host: Url,
    protocol: ProqProtocol,
    query_timeout: Option<Duration>,
}

impl ProqClient {
    ///
    /// Get a HTTPS using Proq client.
    ///
    /// # Arguments
    ///
    /// * `host` - host port combination string: e.g. `localhost:9090`
    /// * `query_timeout` - Maximum query timeout for the client
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use chrono::Utc;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    /// let client = ProqClient::new(
    ///     "localhost:9090",
    ///     Some(Duration::from_secs(5)),
    /// ).unwrap();
    ///# }
    /// ```
    pub fn new(host: &str, query_timeout: Option<Duration>) -> ProqResult<Self> {
        Self::new_with_proto(host, ProqProtocol::HTTPS, query_timeout)
    }

    ///
    /// Get a Proq client with specified protocol.
    ///
    /// # Arguments
    ///
    /// * `host` - host port combination string: e.g. `localhost:9090`
    /// * `protocol` - [ProqProtocol] Currently either HTTP or HTTPS
    /// * `query_timeout` - Maximum query timeout for the client
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use chrono::Utc;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    /// let client = ProqClient::new_with_proto(
    ///     "localhost:9090",
    ///     ProqProtocol::HTTP,
    ///     Some(Duration::from_secs(5)),
    /// ).unwrap();
    ///# }
    /// ```
    pub fn new_with_proto(
        host: &str,
        protocol: ProqProtocol,
        query_timeout: Option<Duration>,
    ) -> ProqResult<Self> {
        let host = Url::from_str(host).map_err(ProqError::UrlParseError)?;

        Ok(Self {
            host,
            query_timeout,
            protocol,
        })
    }

    async fn get_basic(&self, url: Url) -> ProqResult<ApiResult> {
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    async fn get_query(&self, endpoint: &str, query: &impl Serialize) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(&endpoint)?.to_string().as_str())?;
        surf::get(url)
            .set_query(&query)
            .map_err(|e| ProqError::HTTPClientError(Box::new(e)))?
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    async fn post(&self, endpoint: &str, payload: String) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(&endpoint)?.to_string().as_str())?;
        surf::post(url)
            .body_string(payload)
            .set_mime(mime::APPLICATION_WWW_FORM_URLENCODED)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    ///
    /// Make an instant query to Prometheus.
    /// Get all timeseries at that point.
    ///
    /// # Arguments
    ///
    /// * `query` - query string
    /// * `eval_time` - instant query timestamp to query
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use chrono::Utc;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let instantq = client.instant_query("up", None).await;
    ///#     });
    ///# }
    /// ```
    pub async fn instant_query(
        &self,
        query: &str,
        eval_time: Option<DateTime<Utc>>,
    ) -> ProqResult<ApiResult> {
        let query = InstantQuery {
            query: query.into(),
            time: eval_time.as_ref().map(|et| DateTime::timestamp(et)),
            timeout: self.query_timeout.map(|t| t.as_secs().to_string()),
        };
        self.get_query(PROQ_INSTANT_QUERY_URL, &query).await
    }

    ///
    /// Make a range query to Prometheus.
    ///
    /// # Arguments
    ///
    /// * `query` - query string
    /// * `start` - start time of the query
    /// * `end` - end time of the query
    /// * `step` - step duration between start and end range
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use chrono::Utc;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let end = Utc::now();
    /// let start = Some(end - chrono::Duration::minutes(1));
    /// let step = Some(Duration::from_secs_f64(1.5));
    ///
    /// let rangeq = client.range_query("up", start, Some(end), step).await;
    ///#     });
    ///# }
    /// ```
    pub async fn range_query(
        &self,
        query: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        step: Option<Duration>,
    ) -> ProqResult<ApiResult> {
        let query = RangeQuery {
            query: query.into(),
            start: start_time.as_ref().map(|et| DateTime::timestamp(et)),
            end: end_time.as_ref().map(|et| DateTime::timestamp(et)),
            step: step.map(|s| s.as_secs_f64()),
            timeout: self.query_timeout.map(|t| t.as_secs().to_string()),
        };
        self.get_query(PROQ_RANGE_QUERY_URL, &query).await
    }

    ///
    /// Get series from Prometheus
    ///
    /// # Arguments
    ///
    /// * `selectors` - vector of selectors
    /// * `start` - start time of the query
    /// * `end` - end time of the query
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use chrono::Utc;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let end = Utc::now();
    /// let start = Some(end - chrono::Duration::hours(1));
    ///
    /// let selectors = vec!["up", "process_start_time_seconds{job=\"prometheus\"}"];
    /// let series = client.series(selectors, start, Some(end)).await;
    ///#     });
    ///# }
    /// ```
    pub async fn series(
        &self,
        selectors: Vec<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> ProqResult<ApiResult> {
        let query = SeriesRequest {
            selectors: selectors.iter().map(|s| (*s).to_string()).collect(),
            start: start_time.as_ref().map(|et| DateTime::timestamp(et)),
            end: end_time.as_ref().map(|et| DateTime::timestamp(et)),
            timeout: self.query_timeout.map(|t| t.as_secs().to_string()),
        };

        let mut uencser = url::form_urlencoded::Serializer::new(String::new());
        // TODO: Remove the allocation overhead from AsRef.
        for s in query.selectors {
            uencser.append_pair("match[]", s.as_str());
        }
        query
            .start
            .map(|s| uencser.append_pair("start", s.to_string().as_str()));
        query
            .end
            .map(|s| uencser.append_pair("end", s.to_string().as_str()));
        let query = uencser.finish();

        self.post(PROQ_SERIES_URL, query).await
    }

    ///
    /// Get all label names from Prometheus.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let label_names = client.label_names().await;
    ///#     });
    ///# }
    /// ```
    pub async fn label_names(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_LABELS_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Get all label values for a given label from Prometheus.
    ///
    /// # Arguments
    ///
    /// * `label_name` - Label name to get label values
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let label_name = "version";
    /// let label_values = client.label_values(label_name).await;
    ///#     });
    ///# }
    /// ```
    pub async fn label_values(&self, label_name: &str) -> ProqResult<ApiResult> {
        let slug = format!(PROQ_LABEL_VALUES_URL!(), label_name);
        let url: Url = Url::from_str(self.get_slug(slug.as_str())?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Get all Prometheus targets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let targets = client.targets().await;
    ///#     });
    ///# }
    /// ```
    pub async fn targets(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_TARGETS_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Get Prometheus targets filtered by the given target state.
    ///
    /// Target state can be `ACTIVE`, `DROPPED` or `ANY` which are defined in [ProqTargetStates].
    ///
    /// # Arguments
    ///
    /// * `state` - [ProqTargetStates] : State to filter
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let filtered_targets = client.targets_with_state(ProqTargetStates::DROPPED).await;
    ///#     });
    ///# }
    /// ```
    pub async fn targets_with_state(&self, state: ProqTargetStates) -> ProqResult<ApiResult> {
        let query = TargetsWithStatesRequest { state };
        self.get_query(PROQ_TARGETS_URL, &query).await
    }

    ///
    /// Get all rules from Prometheus.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let rules = client.rules().await;
    ///#     });
    ///# }
    /// ```
    pub async fn rules(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_RULES_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Get rules filtered by given type.
    ///
    /// Type can be either `ALERT` or `RECORD` from the [ProqRulesType].
    ///
    /// # Arguments
    ///
    /// * `rule_type` - [ProqRulesType] : Rule type to filter
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let filtered_rules = client.rules_with_type(ProqRulesType::ALERT).await;
    ///#     });
    ///# }
    /// ```
    pub async fn rules_with_type(&self, rule_type: ProqRulesType) -> ProqResult<ApiResult> {
        let query = RulesWithTypeRequest { rule_type };
        self.get_query(PROQ_RULES_URL, &query).await
    }

    ///
    /// Get current alerts Prometheus has.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let alerts = client.alerts().await;
    ///#     });
    ///# }
    /// ```
    pub async fn alerts(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_ALERTS_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Get alert managers currently Prometheus has.
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let alert_managers = client.alert_managers().await;
    ///#     });
    ///# }
    /// ```
    pub async fn alert_managers(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_ALERT_MANAGERS_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Query config that Prometheus configured
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let config = client.config().await;
    ///#     });
    ///# }
    /// ```
    pub async fn config(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_STATUS_CONFIG_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    ///
    /// Query flag values that Prometheus configured with
    ///
    /// # Example
    ///
    /// ```rust
    /// use proq::prelude::*;
    ///# use std::time::Duration;
    ///
    ///# fn main() {
    ///#     let client = ProqClient::new_with_proto(
    ///#         "localhost:9090",
    ///#         ProqProtocol::HTTP,
    ///#         Some(Duration::from_secs(5)),
    ///#     ).unwrap();
    ///#
    ///#     futures::executor::block_on(async {
    /// let flags = client.flags().await;
    ///#     });
    ///# }
    /// ```
    pub async fn flags(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_STATUS_FLAGS_URL)?.to_string().as_str())?;
        self.get_basic(url).await
    }

    pub(crate) fn get_slug(&self, slug: &str) -> ProqResult<Uri> {
        let proto = if self.protocol == ProqProtocol::HTTP {
            "http"
        } else {
            "https"
        };

        uri::Builder::new()
            .scheme(proto)
            .authority(self.host.as_str())
            .path_and_query(slug)
            .build()
            .map_err(ProqError::UrlBuildError)
    }
}
