//!
//! All value types which enables us to interpret various parts of Prometheus.

pub mod prometheus_types {
    //!
    //! Constants that helps Proq to interpret Prometheus return types.
    pub const PROQ_INFINITY: &str = "Inf";
    pub const PROQ_NEGATIVE_INFINITY: &str = "-Inf";
    pub const PROQ_NAN: &str = "NaN";
}
