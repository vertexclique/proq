use failure::*;
use std::result;
use url::ParseError;

pub type ProqResult<T> = result::Result<T, ProqError>;
pub type BoxedUnwrap = std::error::Error + Send + Sync;

/// Error types of Proq
#[derive(Fail, Debug)]
pub enum ProqError {
    #[fail(display = "Generic Error: {}", _0)]
    GenericError(String),
    #[fail(display = "Failed to parse URL: {}", _0)]
    UrlParseError(ParseError),
    #[fail(display = "Failed to build URL: {}", _0)]
    UrlBuildError(http::Error),
    #[fail(display = "Http client Error: {}", _0)]
    HTTPClientError(surf::Exception),
}

impl From<ParseError> for ProqError {
    fn from(e: ParseError) -> Self {
        ProqError::UrlParseError(e)
    }
}