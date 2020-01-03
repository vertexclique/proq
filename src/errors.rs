//!
//! Proq client related error listing
//!
//! All errors are aggregated here and exposed by the Proq will be seen here.

use failure::*;
use std::result;
use url::ParseError;

/// Alias type for Result with Proq errors.
pub type ProqResult<T> = result::Result<T, ProqError>;

/// Error types of Proq
#[derive(Fail, Debug)]
pub enum ProqError {
    /// Generic Error raised from Proq.
    #[fail(display = "Generic Error: {}", _0)]
    GenericError(String),
    /// URL parsing error.
    #[fail(display = "Failed to parse URL: {}", _0)]
    UrlParseError(ParseError),
    /// URL building error.
    #[fail(display = "Failed to build URL: {}", _0)]
    UrlBuildError(http::Error),
    /// HTTP Client error raised from underlying HTTP client.
    #[fail(display = "Http client Error: {}", _0)]
    HTTPClientError(surf::Exception),
}

impl From<ParseError> for ProqError {
    fn from(e: ParseError) -> Self {
        ProqError::UrlParseError(e)
    }
}
