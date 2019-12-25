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
const PROQ_ALERT_MANAGERS_URL: &str = "/api/v1/alertmanagers";
const PROQ_STATUS_CONFIG_URL: &str = "/api/v1/status/config";
const PROQ_STATUS_FLAGS_URL: &str = "/api/v1/status/config";
macro_rules! PROQ_LABEL_VALUES_URL {
    () => {
        "/api/v1/label/{}/values"
    };
}

#[derive(PartialEq)]
pub enum ProqProtocol {
    HTTP,
    HTTPS,
}

pub struct ProqClient {
    host: Url,
    protocol: ProqProtocol,
    query_timeout: Option<Duration>,
}

impl ProqClient {
    pub fn new(host: &str, query_timeout: Option<Duration>) -> ProqResult<Self> {
        Self::new_with_proto(host, ProqProtocol::HTTPS, query_timeout)
    }

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

    async fn get(&self, endpoint: &str, query: &impl Serialize) -> ProqResult<ApiResult> {
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
        self.get(PROQ_INSTANT_QUERY_URL, &query).await
    }

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
        };
        self.get(PROQ_RANGE_QUERY_URL, &query).await
    }

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

    pub async fn label_names(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_LABELS_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn label_values(&self, label_name: &str) -> ProqResult<ApiResult> {
        let slug = format!(PROQ_LABEL_VALUES_URL!(), label_name);
        let url: Url = Url::from_str(self.get_slug(slug.as_str())?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn targets(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_TARGETS_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn targets_with_state(&self, state: ProqTargetStates) -> ProqResult<ApiResult> {
        let query = TargetsWithStatesRequest { state };
        self.get(PROQ_TARGETS_URL, &query).await
    }

    pub async fn rules(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_RULES_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn rules_with_type(&self, rule_type: ProqRulesType) -> ProqResult<ApiResult> {
        let query = RulesWithTypeRequest { rule_type };
        self.get(PROQ_RULES_URL, &query).await
    }

    pub async fn alert_managers(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_ALERT_MANAGERS_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn config(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_STATUS_CONFIG_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn flags(&self) -> ProqResult<ApiResult> {
        let url: Url = Url::from_str(self.get_slug(PROQ_STATUS_FLAGS_URL)?.to_string().as_str())?;
        surf::get(url)
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
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
