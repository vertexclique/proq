use std::str::FromStr;
use ::url::Url;

use super::errors::*;
use http::{uri, Uri};

use chrono::offset::Utc;
use chrono::DateTime;

use surf::*;
use crate::query_types::InstantQuery;
use std::time::Duration;
use crate::result_types::ApiResult;

pub struct ProqClient {
    host: Url,
    query_timeout: Option<Duration>
}

impl ProqClient {
    pub fn new(host: &str, query_timeout: Option<Duration>) -> ProqResult<Self> {
        let host = Url::from_str(host).map_err(|e| ProqError::UrlParseError(e))?;

        Ok(Self { host, query_timeout })
    }

    pub async fn query_instant(&self, query: &str, eval_time: Option<DateTime<Utc>>) -> ProqResult<ApiResult> {
        let query = InstantQuery {
            query: query.into(),
            time: eval_time.as_ref().map(|et| DateTime::timestamp(et)),
            timeout: self.query_timeout.map(|t| t.as_secs().to_string())
        };

        let url: Url = Url::from_str(self.get_slug("/api/v1/query")?.to_string().as_str())?;

        surf::get(url).set_query(&query)
            .map_err(|e| {
                ProqError::HTTPClientError(Box::new(e))
            })?
            .recv_json().await
            .map_err(|e| {
                ProqError::GenericError(e.to_string())
            })
    }

    pub(crate) fn get_slug(&self, slug: &str) -> ProqResult<Uri> {
        uri::Builder::new()
            .scheme("https")
            .authority(self.host.as_str())
            .path_and_query(slug)
            .build()
            .map_err(|e| ProqError::UrlBuildError(e))
    }
}
