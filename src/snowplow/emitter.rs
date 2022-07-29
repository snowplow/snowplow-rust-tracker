use reqwest::Client;

pub struct Emitter {
    pub collector_url: String,
    http_client: Client,
}

impl Emitter {
    pub fn new(collector_url: String) -> Emitter {
        Emitter {
            collector_url,
            http_client: Client::new(),
        }
    }

    pub async fn post<T>(&self, payload: T, url: &str) -> Result<String, reqwest::Error>
    where
        T: serde::Serialize,
    {
        let collector_url = url.to_string() + "/com.snowplowanalytics.snowplow/tp2";
        let resp = self
            .http_client
            .post(collector_url)
            .json(&payload)
            .send()
            .await?;

        resp.text().await
    }
}
