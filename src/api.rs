use std::str::FromStr;
use url::Url;

use super::errors::*;
use http::{uri, Uri};

use surf::*;

pub struct ProqClient {
    host: Url,
}

impl ProqClient {
    pub fn new(host: &str) -> ProqResult<Self> {
        let host = Url::from_str(host).map_err(|e| ProqError::UrlParseError(e))?;

        Ok(Self { host })
    }

    pub fn query_instant(&self) -> ProqResult<Self> {
        let _l: Url = Url::from_str(self.get_slug("/")?.to_string().as_str())?;

        unimplemented!()
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
