use chrono::Utc;
use once_cell::sync::OnceCell;
use proq::api::{ProqClient, ProqProtocol};
use proq::result_types::ApiResult::ApiOk;
use std::sync::Once;
use std::time::Duration;

static CLIENT: OnceCell<ProqClient> = OnceCell::new();
static BARRIER: Once = Once::new();

fn client() -> &'static ProqClient {
    BARRIER.call_once(|| {
        let c = ProqClient::new_with_proto(
            "localhost:9090",
            ProqProtocol::HTTP,
            Some(Duration::from_secs(5)),
        )
        .unwrap();
        let _ = CLIENT.set(c);
    });

    CLIENT.get().unwrap()
}

#[test]
fn proq_instant_query() {
    futures::executor::block_on(async {
        let x = match client().instant_query("up", None).await.unwrap() {
            ApiOk(r) => {
                dbg!(r);
                true
            }
            e => {
                dbg!(e);
                false
            }
        };

        assert!(x)
    });
}

#[test]
fn proq_range_query() {
    futures::executor::block_on(async {
        let end = Utc::now();
        let start = Some(end - chrono::Duration::hours(1));
        let step = Some(Duration::from_secs_f64(1.5));

        let x = match client()
            .range_query("up", start, Some(end), step)
            .await
            .unwrap()
        {
            ApiOk(r) => {
                dbg!(r);
                true
            }
            e => {
                dbg!(e);
                false
            },
        };

        assert!(x)
    });
}
