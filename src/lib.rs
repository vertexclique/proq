//!
//! Idiomatic Async Prometheus Query (PromQL) Client for Rust.
//!
//! This crate provides general client API for Prometheus Query API.
//! All queries can be written with PromQL notation.
//!
//!
//! # Basic Usage
//! ```rust
//! use proq::prelude::*;
//!# use chrono::Utc;
//!# use std::time::Duration;
//!
//!fn main() {
//!    let client = ProqClient::new(
//!        "localhost:9090",
//!        Some(Duration::from_secs(5)),
//!    ).unwrap();
//!
//!    futures::executor::block_on(async {
//!        let end = Utc::now();
//!        let start = Some(end - chrono::Duration::minutes(1));
//!        let step = Some(Duration::from_secs_f64(1.5));
//!
//!        let rangeq = client.range_query("up", start, Some(end), step).await;
//!    });
//!}
//! ```
//!

#![doc(html_logo_url = "https://github.com/vertexclique/proq/raw/master/img/proq.png")]
// Force missing implementations
//#![warn(missing_docs)]
//#![warn(missing_debug_implementations)]
#![forbid(unsafe_code)]

pub mod api;
pub mod errors;
pub mod query_types;
pub mod result_types;
pub mod value_types;

pub mod prelude {
    //!
    //! Prelude of the Proq package.
    //!
    //! Includes all request response types to client itself.
    pub use super::api::*;
    pub use super::errors::*;
    pub use super::query_types::*;
    pub use super::result_types::*;
    pub use super::value_types::prometheus_types::*;
    pub use chrono::prelude::*;
}
