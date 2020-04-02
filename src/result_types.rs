//!
//! Response types to Proq from Prometheus.
//!
//! Return types are mostly borrowed from:
//! https://github.com/allengeorge/prometheus-query/blob/master/src/messages.rs
//!
//! extended with filtered and unfiltered methods and new beta endpoints.
use std::collections::HashMap;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::result::Result as StdResult;
use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use serde::{
    de,
    de::{MapAccess, SeqAccess, Unexpected, Visitor},
    ser::{SerializeStruct, SerializeTuple},
    {Deserialize, Deserializer, Serialize, Serializer},
};
use url::Url;
use url_serde::{De, Ser};

use crate::value_types::prometheus_types::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status")]
pub enum ApiResult {
    #[serde(rename = "success")]
    ApiOk(ApiOk),
    #[serde(rename = "error")]
    ApiErr(ApiErr),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ApiOk {
    #[serde(default)]
    pub data: Option<Data>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ApiErr {
    #[serde(rename = "errorType")]
    pub error_type: String,
    #[serde(rename = "error")]
    pub error_message: String,
    #[serde(default)]
    pub data: Option<Data>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Data {
    Expression(Expression),
    Series(Series),
    LabelsOrValues(LabelsOrValues),
    Targets(Targets),
    Rules(Rules),
    Alerts(Alerts),
    AlertManagers(AlertManagers),
    Config(Config),
    Snapshot(Snapshot),
    // IMPORTANT: this must *always* be the final variant.
    // For untagged enums serde will attempt deserialization using
    // each variant in order and accept the first one that is successful.
    // Since `Flags` is a map, it captures any other map-like
    // types, including `Config`, `Snapshot`, etc. To give those
    // variants a chance to be matches this variant must be the last
    Flags(HashMap<String, String>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "resultType", content = "result")]
pub enum Expression {
    #[serde(rename = "scalar")]
    Scalar(Sample),
    #[serde(rename = "string")]
    String(StringSample),
    #[serde(rename = "vector")]
    Instant(Vec<Instant>),
    #[serde(rename = "matrix")]
    Range(Vec<Range>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Instant {
    pub metric: Metric,
    #[serde(rename = "value")]
    pub sample: Sample,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Range {
    pub metric: Metric,
    #[serde(rename = "values")]
    pub samples: Vec<Sample>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Metric {
    #[serde(flatten)]
    pub labels: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub epoch: f64,
    pub value: f64,
}

impl<'de> Deserialize<'de> for Sample {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VisitorImpl;

        impl<'de> Visitor<'de> for VisitorImpl {
            type Value = Sample;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("Prometheus sample")
            }

            fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let epoch = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::missing_field("sample time"))?;
                let value = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| de::Error::missing_field("sample value"))?;

                let value = match value {
                    PROQ_INFINITY => std::f64::INFINITY,
                    PROQ_NEGATIVE_INFINITY => std::f64::NEG_INFINITY,
                    PROQ_NAN => std::f64::NAN,
                    _ => value
                        .parse::<f64>()
                        .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self))?,
                };

                Ok(Sample { epoch, value })
            }
        }

        deserializer.deserialize_seq(VisitorImpl)
    }
}

impl Serialize for Sample {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(2)?;
        s.serialize_element(&self.epoch)?;
        s.serialize_element(&self.value)?;
        s.end()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StringSample {
    pub epoch: f64,
    pub value: String,
}

impl<'de> Deserialize<'de> for StringSample {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VisitorImpl;

        impl<'de> Visitor<'de> for VisitorImpl {
            type Value = StringSample;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("Prometheus string sample")
            }

            fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let epoch = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::missing_field("sample time"))?;
                let value = seq
                    .next_element::<String>()?
                    .ok_or_else(|| de::Error::missing_field("sample value"))?;

                Ok(StringSample { epoch, value })
            }
        }

        deserializer.deserialize_seq(VisitorImpl)
    }
}

impl Serialize for StringSample {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(2)?;
        s.serialize_element(&self.epoch)?;
        s.serialize_element(&self.value)?;
        s.end()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Series(pub Vec<Metric>);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LabelsOrValues(pub Vec<String>);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Targets {
    #[serde(default, rename = "activeTargets")]
    pub active: Vec<ActiveTarget>,
    #[serde(default, rename = "droppedTargets")]
    pub dropped: Vec<DroppedTarget>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTarget {
    pub discovered_labels: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    #[serde(with = "url_serde")]
    pub scrape_url: Url,
    #[serde(
        deserialize_with = "empty_string_is_none",
        serialize_with = "none_to_empty_string"
    )]
    pub last_error: Option<String>,
    #[serde(
        deserialize_with = "rfc3339_to_date_time",
        serialize_with = "date_time_to_rfc3339"
    )]
    pub last_scrape: DateTime<FixedOffset>,
    #[serde(
        deserialize_with = "deserialize_health",
        serialize_with = "serialize_health"
    )]
    pub health: TargetHealth,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TargetHealth {
    Up,
    Down,
    Unknown,
}

fn empty_string_is_none<'de, D: Deserializer<'de>>(d: D) -> StdResult<Option<String>, D::Error> {
    let o: Option<String> = Option::deserialize(d)?;
    Ok(o.filter(|s| !s.is_empty()))
}

fn none_to_empty_string<S: Serializer>(
    s: &Option<String>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    if let Some(v) = s {
        serializer.serialize_str(&v)
    } else {
        serializer.serialize_str("")
    }
}

fn rfc3339_to_date_time<'de, D: Deserializer<'de>>(
    d: D,
) -> StdResult<DateTime<FixedOffset>, D::Error> {
    let s = String::deserialize(d)?;
    DateTime::from_str(&s).map_err(de::Error::custom)
}

fn date_time_to_rfc3339<S: Serializer>(
    v: &DateTime<FixedOffset>,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    serializer.serialize_str(&v.to_rfc3339())
}

fn deserialize_health<'de, D: Deserializer<'de>>(d: D) -> StdResult<TargetHealth, D::Error> {
    let o: Option<String> = Option::deserialize(d)?;
    Ok(o.map_or(TargetHealth::Unknown, |s| match s.as_str() {
        "up" => TargetHealth::Up,
        "down" => TargetHealth::Down,
        _ => TargetHealth::Unknown,
    }))
}

fn serialize_health<S: Serializer>(
    health: &TargetHealth,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    let value = match health {
        TargetHealth::Up => "up",
        TargetHealth::Down => "down",
        TargetHealth::Unknown => "unknown",
    };

    serializer.serialize_str(value)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DroppedTarget {
    pub discovered_labels: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlertManagers {
    #[serde(default, rename = "activeAlertmanagers")]
    pub active: Vec<AlertManager>,
    #[serde(default, rename = "droppedAlertmanagers")]
    pub dropped: Vec<AlertManager>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlertManager {
    pub url: Url,
}

impl<'de> Deserialize<'de> for AlertManager {
    fn deserialize<D>(deserializer: D) -> StdResult<AlertManager, D::Error>
    where
        D: Deserializer<'de>,
    {
        // variant of: https://serde.rs/deserialize-struct.html

        struct VisitorImpl;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Url,
        };

        const FIELDS: &[&str] = &["url"];

        impl<'de> Visitor<'de> for VisitorImpl {
            type Value = AlertManager;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("AlertManager")
            }

            fn visit_map<V>(self, mut map: V) -> StdResult<AlertManager, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut url: Option<Url> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Url => {
                            if url.is_some() {
                                return Err(de::Error::duplicate_field("url"));
                            }
                            url = De::into_inner(map.next_value()?); // FIXME: how does this work??!
                        }
                    }
                }
                let url = url.ok_or_else(|| de::Error::missing_field("url"))?;
                Ok(AlertManager { url })
            }
        }

        deserializer.deserialize_struct("AlertManager", &FIELDS, VisitorImpl)
    }
}

impl Serialize for AlertManager {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("AlertManager", 1)?;
        s.serialize_field("url", &Ser::new(&self.url))?;
        s.end()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Snapshot {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    pub yaml: String,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AlertState {
    INACTIVE,
    PENDING,
    FIRING,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub groups: Vec<RuleGroups>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RuleGroups {
    pub rules: Vec<Rule>,
    pub file: String,
    pub interval: i64,
    pub name: String,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    RECORDING,
    ALERTING,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Rule {
    pub alerts: Option<Vec<Alert>>,
    pub annotations: Option<HashMap<String, String>>,
    pub duration: Option<i64>,
    pub labels: Option<HashMap<String, String>>,
    pub health: String,
    pub name: String,
    pub query: String,
    #[serde(rename = "type")]
    pub rule_type: RuleType,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Alert {
    #[serde(default, rename = "activeAt")]
    pub active_at: String,
    pub annotations: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
    pub state: AlertState,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Alerts {
    pub alerts: Vec<Alert>,
}
