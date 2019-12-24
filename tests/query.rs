use proq::api::{ProqClient, ProqProtocol};
use proq::result_types::ApiResult::ApiOk;
use std::time::Duration;

#[test]
fn proq_instant_query() {
    let client = ProqClient::new_with_proto(
        "localhost:9090",
        ProqProtocol::HTTP,
        Some(Duration::from_secs(5)),
    )
    .unwrap();

    futures::executor::block_on(async {
        let x = match client.instant_query("up", None).await.unwrap() {
            ApiOk(r) => {
                dbg!(r);
                true
            }
            _ => false,
        };

        assert!(x)
    });
}
