use std::collections::HashMap;
use std::result::Result as StdResult;

use chrono::{DateTime, FixedOffset};
use url::Url;

use proq::result_types::{
    ActiveTarget, AlertManager, AlertManagers, ApiErr, ApiOk, ApiResult, Config, Data,
    DroppedTarget, Expression, Instant, LabelsOrValues, Metric, Range, Sample, Series, Snapshot,
    StringSample, TargetHealth, Targets,
};

#[test]
fn should_deserialize_json_error() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "error",
            "error": "Major",
            "errorType": "Seriously Bad"
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiErr(ApiErr {
            error_message: "Major".to_string(),
            error_type: "Seriously Bad".to_string(),
            data: None,
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_error_with_instant_and_warnings() -> StdResult<(), std::io::Error> {
    let expected_json = r#"
        {
            "status": "error",
            "error": "This is a strange error",
            "errorType": "Weird",
            "warnings": [
                "You timed out, foo"
            ],
            "data" : {
                "resultType" : "vector",
                "result" : [
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "prometheus",
                            "instance" : "localhost:9090"
                        },
                        "value": [ 1435781451.781, "1" ]
                    },
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "node",
                            "instance" : "localhost:9100"
                        },
                        "value" : [ 1435781451.781, "0" ]
                    }
                ]
            }
        }
        "#;
    let expected = serde_json::from_str::<ApiResult>(expected_json)?;

    let mut metric_1: HashMap<String, String> = HashMap::new();
    metric_1.insert("__name__".to_owned(), "up".to_owned());
    metric_1.insert("job".to_owned(), "prometheus".to_owned());
    metric_1.insert("instance".to_owned(), "localhost:9090".to_owned());

    let mut metric_2: HashMap<String, String> = HashMap::new();
    metric_2.insert("__name__".to_owned(), "up".to_owned());
    metric_2.insert("job".to_owned(), "node".to_owned());
    metric_2.insert("instance".to_owned(), "localhost:9100".to_owned());

    let actual = ApiResult::ApiErr(ApiErr {
        error_type: "Weird".to_owned(),
        error_message: "This is a strange error".to_owned(),
        data: Some(Data::Expression(Expression::Instant(vec![
            Instant {
                metric: Metric {
                    labels: metric_1.clone(),
                },
                sample: Sample {
                    epoch: 1435781451.781,
                    value: 1 as f64,
                },
            },
            Instant {
                metric: Metric {
                    labels: metric_2.clone(),
                },
                sample: Sample {
                    epoch: 1435781451.781,
                    value: 0 as f64,
                },
            },
        ]))),
        warnings: vec!["You timed out, foo".to_owned()],
    });
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn should_deserialize_json_prom_scalar() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "resultType": "scalar",
                "result": [1435781451.781, "1"]
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Expression(Expression::Scalar(Sample {
                epoch: 1435781451.781,
                value: 1 as f64,
            }))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_scalar_with_warnings() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "warnings": ["You timed out, foo"],
            "status": "success",
            "data": {
                "resultType": "scalar",
                "result": [1435781451.781, "1"]
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Expression(Expression::Scalar(Sample {
                epoch: 1435781451.781,
                value: 1 as f64,
            }))),
            warnings: vec!["You timed out, foo".to_owned()],
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_string() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "resultType": "string",
                "result": [1435781451.781, "foo"]
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Expression(Expression::String(StringSample {
                epoch: 1435781451.781,
                value: "foo".to_owned(),
            }))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_vector() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status" : "success",
            "data" : {
                "resultType" : "vector",
                "result" : [
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "prometheus",
                            "instance" : "localhost:9090"
                        },
                        "value": [ 1435781451.781, "1" ]
                    },
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "node",
                            "instance" : "localhost:9100"
                        },
                        "value" : [ 1435781451.781, "0" ]
                    }
                ]
            }
        }
        "#;

    let mut metric_1: HashMap<String, String> = HashMap::new();
    metric_1.insert("__name__".to_owned(), "up".to_owned());
    metric_1.insert("job".to_owned(), "prometheus".to_owned());
    metric_1.insert("instance".to_owned(), "localhost:9090".to_owned());

    let mut metric_2: HashMap<String, String> = HashMap::new();
    metric_2.insert("__name__".to_owned(), "up".to_owned());
    metric_2.insert("job".to_owned(), "node".to_owned());
    metric_2.insert("instance".to_owned(), "localhost:9100".to_owned());

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Expression(Expression::Instant(vec!(
                Instant {
                    metric: Metric {
                        labels: metric_1.clone(),
                    },
                    sample: Sample {
                        epoch: 1435781451.781,
                        value: 1 as f64,
                    },
                },
                Instant {
                    metric: Metric {
                        labels: metric_2.clone(),
                    },
                    sample: Sample {
                        epoch: 1435781451.781,
                        value: 0 as f64,
                    },
                },
            )))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_matrix() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status" : "success",
            "data" : {
                "resultType" : "matrix",
                "result" : [
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "prometheus",
                            "instance" : "localhost:9090"
                        },
                        "values" : [
                           [ 1435781430.781, "1" ],
                           [ 1435781445.781, "1" ],
                           [ 1435781460.781, "1" ]
                        ]
                    },
                    {
                        "metric" : {
                            "__name__" : "up",
                            "job" : "node",
                            "instance" : "localhost:9091"
                        },
                        "values" : [
                           [ 1435781430.781, "0" ],
                           [ 1435781445.781, "0" ],
                           [ 1435781460.781, "1" ]
                        ]
                    }
                ]
            }
        }
        "#;

    let mut metric_1: HashMap<String, String> = HashMap::new();
    metric_1.insert("__name__".to_owned(), "up".to_owned());
    metric_1.insert("job".to_owned(), "prometheus".to_owned());
    metric_1.insert("instance".to_owned(), "localhost:9090".to_owned());

    let mut metric_2: HashMap<String, String> = HashMap::new();
    metric_2.insert("__name__".to_owned(), "up".to_owned());
    metric_2.insert("job".to_owned(), "node".to_owned());
    metric_2.insert("instance".to_owned(), "localhost:9091".to_owned());

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Expression(Expression::Range(vec!(
                Range {
                    metric: Metric {
                        labels: metric_1.clone(),
                    },
                    samples: vec!(
                        Sample {
                            epoch: 1435781430.781,
                            value: 1 as f64,
                        },
                        Sample {
                            epoch: 1435781445.781,
                            value: 1 as f64,
                        },
                        Sample {
                            epoch: 1435781460.781,
                            value: 1 as f64,
                        },
                    ),
                },
                Range {
                    metric: Metric {
                        labels: metric_2.clone(),
                    },
                    samples: vec!(
                        Sample {
                            epoch: 1435781430.781,
                            value: 0 as f64,
                        },
                        Sample {
                            epoch: 1435781445.781,
                            value: 0 as f64,
                        },
                        Sample {
                            epoch: 1435781460.781,
                            value: 1 as f64,
                        },
                    ),
                },
            )))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_labels() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status" : "success",
            "data" :[
                "__name__",
                "call",
                "code",
                "config",
                "dialer_name",
                "endpoint",
                "event",
                "goversion",
                "handler",
                "instance",
                "interval",
                "job",
                "le",
                "listener_name",
                "name",
                "quantile",
                "reason",
                "role",
                "scrape_job",
                "slice",
                "version"
            ]
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::LabelsOrValues(LabelsOrValues(vec![
                "__name__".to_owned(),
                "call".to_owned(),
                "code".to_owned(),
                "config".to_owned(),
                "dialer_name".to_owned(),
                "endpoint".to_owned(),
                "event".to_owned(),
                "goversion".to_owned(),
                "handler".to_owned(),
                "instance".to_owned(),
                "interval".to_owned(),
                "job".to_owned(),
                "le".to_owned(),
                "listener_name".to_owned(),
                "name".to_owned(),
                "quantile".to_owned(),
                "reason".to_owned(),
                "role".to_owned(),
                "scrape_job".to_owned(),
                "slice".to_owned(),
                "version".to_owned(),
            ]))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_label_values() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status" : "success",
            "data" :[
                "node",
                "prometheus"
            ]
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::LabelsOrValues(LabelsOrValues(vec![
                "node".to_owned(),
                "prometheus".to_owned(),
            ]))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_series() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status" : "success",
            "data" : [
                {
                    "__name__" : "up",
                    "job" : "prometheus",
                    "instance" : "localhost:9090"
                },
                {
                    "__name__" : "up",
                    "job" : "node",
                    "instance" : "localhost:9091"
                },
                {
                    "__name__" : "process_start_time_seconds",
                    "job" : "prometheus",
                    "instance" : "localhost:9090"
                }
            ]
        }
        "#;

    let mut metric_1: HashMap<String, String> = HashMap::new();
    metric_1.insert("__name__".to_owned(), "up".to_owned());
    metric_1.insert("job".to_owned(), "prometheus".to_owned());
    metric_1.insert("instance".to_owned(), "localhost:9090".to_owned());

    let mut metric_2: HashMap<String, String> = HashMap::new();
    metric_2.insert("__name__".to_owned(), "up".to_owned());
    metric_2.insert("job".to_owned(), "node".to_owned());
    metric_2.insert("instance".to_owned(), "localhost:9091".to_owned());

    let mut metric_3: HashMap<String, String> = HashMap::new();
    metric_3.insert(
        "__name__".to_owned(),
        "process_start_time_seconds".to_owned(),
    );
    metric_3.insert("job".to_owned(), "prometheus".to_owned());
    metric_3.insert("instance".to_owned(), "localhost:9090".to_owned());

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Series(Series(vec![
                Metric { labels: metric_1 },
                Metric { labels: metric_2 },
                Metric { labels: metric_3 },
            ]))),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_targets() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "activeTargets": [
                    {
                        "discoveredLabels": {
                            "__address__": "127.0.0.1:9090",
                            "__metrics_path__": "/metrics",
                            "__scheme__": "http",
                            "job": "prometheus"
                        },
                        "labels": {
                            "instance": "127.0.0.1:9090",
                            "job": "prometheus"
                        },
                        "scrapeUrl": "http://127.0.0.1:9090/metrics",
                        "lastError": "",
                        "lastScrape": "2017-01-17T15:07:44.723715405+01:00",
                        "health": "up"
                    }
                ],
                "droppedTargets": [
                    {
                        "discoveredLabels": {
                            "__address__": "127.0.0.1:9100",
                            "__metrics_path__": "/metrics",
                            "__scheme__": "http",
                            "job": "node"
                        }
                    }
                ]
            }
        }
        "#;

    let mut active_discovered_labels: HashMap<String, String> = HashMap::new();
    active_discovered_labels.insert("__address__".to_owned(), "127.0.0.1:9090".to_owned());
    active_discovered_labels.insert("__metrics_path__".to_owned(), "/metrics".to_owned());
    active_discovered_labels.insert("__scheme__".to_owned(), "http".to_owned());
    active_discovered_labels.insert("job".to_owned(), "prometheus".to_owned());
    let active_discovered_labels = active_discovered_labels;

    let mut active_labels: HashMap<String, String> = HashMap::new();
    active_labels.insert("instance".to_owned(), "127.0.0.1:9090".to_owned());
    active_labels.insert("job".to_owned(), "prometheus".to_owned());
    let active_labels = active_labels;

    let mut dropped_discovered_labels: HashMap<String, String> = HashMap::new();
    dropped_discovered_labels.insert("__address__".to_owned(), "127.0.0.1:9100".to_owned());
    dropped_discovered_labels.insert("__metrics_path__".to_owned(), "/metrics".to_owned());
    dropped_discovered_labels.insert("__scheme__".to_owned(), "http".to_owned());
    dropped_discovered_labels.insert("job".to_owned(), "node".to_owned());
    let dropped_discovered_labels = dropped_discovered_labels;

    let last_scrape: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2017-01-17T15:07:44.723715405+01:00").unwrap();

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        res,
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Targets(Targets {
                active: vec![ActiveTarget {
                    discovered_labels: active_discovered_labels,
                    labels: active_labels,
                    scrape_url: Url::parse("http://127.0.0.1:9090/metrics").unwrap(),
                    last_error: None,
                    last_scrape,
                    health: TargetHealth::Up,
                },],
                dropped: vec![DroppedTarget {
                    discovered_labels: dropped_discovered_labels
                },],
            })),
            warnings: Vec::new(),
        })
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_alert_managers() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "activeAlertmanagers": [
                    {
                        "url": "http://127.0.0.1:9090/api/v1/alerts"
                    }
                ],
                "droppedAlertmanagers": [
                    {
                        "url": "http://127.0.0.1:9093/api/v1/alerts"
                    }
                ]
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::AlertManagers(AlertManagers {
                active: vec![AlertManager {
                    url: Url::parse("http://127.0.0.1:9090/api/v1/alerts").unwrap(),
                },],
                dropped: vec![AlertManager {
                    url: Url::parse("http://127.0.0.1:9093/api/v1/alerts").unwrap(),
                },],
            })),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_flags() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "alertmanager.notification-queue-capacity": "10000",
                "alertmanager.timeout": "10s",
                "log.level": "info",
                "query.lookback-delta": "5m",
                "query.max-concurrency": "20"
            }
        }
        "#;

    let mut flags: HashMap<String, String> = HashMap::new();
    flags.insert(
        "alertmanager.notification-queue-capacity".to_owned(),
        "10000".to_owned(),
    );
    flags.insert("alertmanager.timeout".to_owned(), "10s".to_owned());
    flags.insert("log.level".to_owned(), "info".to_owned());
    flags.insert("query.lookback-delta".to_owned(), "5m".to_owned());
    flags.insert("query.max-concurrency".to_owned(), "20".to_owned());
    let flags = flags;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Flags(flags)),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

#[test]
fn should_deserialize_json_prom_snapshot() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "name": "20171210T211224Z-2be650b6d019eb54"
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Snapshot(Snapshot {
                name: "20171210T211224Z-2be650b6d019eb54".to_owned()
            })),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

// FIXME: make this an actual tests
#[test]
fn should_serialize_rust_prom_snapshot() -> StdResult<(), std::io::Error> {
    let s = serde_json::to_string_pretty(&ApiResult::ApiOk(ApiOk {
        data: Some(Data::Snapshot(Snapshot {
            name: "20171210T211224Z-2be650b6d019eb54".to_owned(),
        })),
        warnings: Vec::new(),
    }))?;

    dbg!(s);

    Ok(())
}

#[test]
fn should_deserialize_json_prom_config() -> StdResult<(), std::io::Error> {
    let j = r#"
        {
            "status": "success",
            "data": {
                "yaml": "CONTENT"
            }
        }
        "#;

    let res = serde_json::from_str::<ApiResult>(j)?;
    assert_eq!(
        ApiResult::ApiOk(ApiOk {
            data: Some(Data::Config(Config {
                yaml: "CONTENT".to_owned()
            })),
            warnings: Vec::new(),
        }),
        res
    );

    Ok(())
}

// FIXME: make this an actual tests
#[test]
fn should_serialize_rust_prom_config() -> StdResult<(), std::io::Error> {
    let s = serde_json::to_string_pretty(&ApiResult::ApiOk(ApiOk {
        data: Some(Data::Config(Config {
            yaml: "CONTENT".to_owned(),
        })),
        warnings: Vec::new(),
    }))?;

    dbg!(s);

    Ok(())
}
