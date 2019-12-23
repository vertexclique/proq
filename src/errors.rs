use failure::*;
use std::result;
use url::ParseError;

pub type ProqResult<T> = result::Result<T, ProqError>;

/// Error types of Proq
#[derive(Fail, Debug)]
pub enum ProqError {
    #[fail(display = "Failed to parse URL: {}", _0)]
    UrlParseError(ParseError),
    #[fail(display = "Failed to build URL: {}", _0)]
    UrlBuildError(http::Error),
}

impl From<ParseError> for ProqError {
    fn from(e: ParseError) -> Self {
        ProqError::UrlParseError(e)
    }
}
