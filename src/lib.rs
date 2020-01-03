//!
//!

// Force missing implementations
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
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
}
