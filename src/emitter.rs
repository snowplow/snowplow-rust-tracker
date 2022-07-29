use reqwest::Client;
use crate::payload::Payload;
use serde_json::json;

pub struct Emitter {
    pub collector_url: String,
    http_client: Client,
}

impl Emitter {
    pub fn new(collector_url: &str) -> Emitter {
        Emitter {
            collector_url: collector_url.to_string(),
            http_client: Client::new(),
        }
    }

    pub async fn post(&self, payload: Payload, url: &str) -> Result<String, reqwest::Error> {
        let collector_url = url.to_string() + "/com.snowplowanalytics.snowplow/tp2";

        let payload = json!({
            "schema": "iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4",
            "data": vec![payload]
        });

        let resp = self
            .http_client
            .post(collector_url)
            .json(&payload)
            .send()
            .await?;

        resp.text().await
    }
}
