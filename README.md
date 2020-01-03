<h1 align="center">
    <img src="https://github.com/vertexclique/proq/raw/master/img/proq.png" width="200" height="200"/>
</h1>
<div align="center">
 <strong>
   Proq – Idiomatic Async Prometheus Query (PromQL) Client for Rust.
 </strong>
<hr>

[![Build Status](https://github.com/vertexclique/proq/workflows/CI/badge.svg)](https://github.com/vertexclique/proq/actions)
[![Latest Version](https://img.shields.io/crates/v/proq.svg)](https://crates.io/crates/proq)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/proq/)
</div>

This crate provides async client API for Prometheus Query API.
All queries can be written with PromQL notation.
Timeout and protocol configuration can be passed at the client initiation time.

#### Adding as dependency
```toml
[dependencies]
proq = "0.1"
```

#### Basic Usage
```rust
use proq::prelude::*;
use std::time::Duration;

fn main() {
    let client = ProqClient::new(
        "localhost:9090",
        Some(Duration::from_secs(5)),
    ).unwrap();

    futures::executor::block_on(async {
        let end = Utc::now();
        let start = Some(end - chrono::Duration::minutes(1));
        let step = Some(Duration::from_secs_f64(1.5));

        let rangeq = client.range_query("up", start, Some(end), step).await;
    });
}
```

For more information please head to the [Documentation](https://docs.rs/proq/).