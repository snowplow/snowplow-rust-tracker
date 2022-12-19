use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use snowplow_tracker::{HttpClient, SelfDescribingJson};
use testcontainers::clients::Cli;

use crate::common::setup;

pub struct FlakeyHttpClient {
    pub micro_url: String,
    pub count: Arc<AtomicUsize>,
    pub number_of_events_to_block: usize,
}

#[async_trait::async_trait]
impl HttpClient for FlakeyHttpClient {
    async fn post(&self, payload: SelfDescribingJson) -> Result<u16, snowplow_tracker::Error> {
        if self.count.load(Ordering::SeqCst) < self.number_of_events_to_block {
            self.count.fetch_add(1, Ordering::SeqCst);
            return Ok(500);
        } else {
            let client = reqwest::Client::new();
            Ok(client
                .post(&(self.micro_url.to_string() + "/com.snowplowanalytics.snowplow/tp2"))
                .json(&payload)
                .send()
                .await
                .unwrap()
                .status()
                .as_u16())
        }
    }

    fn clone(&self) -> Box<dyn HttpClient + Send + Sync> {
        Box::new(FlakeyHttpClient {
            count: self.count.clone(),
            number_of_events_to_block: self.number_of_events_to_block,
            micro_url: self.micro_url.clone(),
        })
    }
}

#[tokio::test]
async fn flaky_http_client_returns_500_n_times() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let client = FlakeyHttpClient {
        micro_url: micro_url.to_string(),
        count: Arc::new(AtomicUsize::new(0)),
        number_of_events_to_block: 5,
    };

    let sdj = SelfDescribingJson {
        schema: String::new(),
        data: serde_json::json!({}),
    };

    for _ in 0..5 {
        assert_eq!(client.post(sdj.clone()).await.unwrap(), 500);
    }

    assert_eq!(client.post(sdj.clone()).await.unwrap(), 200);
}
