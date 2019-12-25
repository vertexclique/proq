use ::url::Url;
use std::str::FromStr;

use super::errors::*;
use http::{uri, Uri};

use chrono::offset::Utc;
use chrono::DateTime;

use crate::query_types::{InstantQuery, RangeQuery};
use crate::result_types::ApiResult;
use std::time::Duration;
use surf::*;

const PROQ_INSTANT_QUERY_URL: &'static str = "/api/v1/query";
const PROQ_RANGE_QUERY_URL: &'static str = "/api/v1/query_range";

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
        let host = Url::from_str(host).map_err(|e| ProqError::UrlParseError(e))?;

        Ok(Self {
            host,
            query_timeout,
            protocol,
        })
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

        let url: Url = Url::from_str(self.get_slug(PROQ_INSTANT_QUERY_URL)?.to_string().as_str())?;

        surf::get(url)
            .set_query(&query)
            .map_err(|e| ProqError::HTTPClientError(Box::new(e)))?
            .recv_json()
            .await
            .map_err(|e| ProqError::GenericError(e.to_string()))
    }

    pub async fn range_query(&self,
                             query: &str,
                             start_time: Option<DateTime<Utc>>,
                             end_time: Option<DateTime<Utc>>,
                             step: Option<Duration>
    ) -> ProqResult<ApiResult> {
        let query = RangeQuery {
            query: query.into(),
            start: start_time.as_ref().map(|et| DateTime::timestamp(et)),
            end: end_time.as_ref().map(|et| DateTime::timestamp(et)),
            step: step.map(|s| s.as_secs_f64()),
        };

        let url: Url = Url::from_str(self.get_slug(PROQ_RANGE_QUERY_URL)?.to_string().as_str())?;

        surf::get(url)
            .set_query(&query)
            .map_err(|e| ProqError::HTTPClientError(Box::new(e)))?
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
            .map_err(|e| ProqError::UrlBuildError(e))
    }
}
