use serde_json::json;
use snowplow_rust_tracker::snowplow::emitter::Emitter;
use snowplow_rust_tracker::snowplow::event::SelfDescribingJson;
use snowplow_rust_tracker::snowplow::tracker::Tracker;
use snowplow_rust_tracker::snowplow::Snowplow;
// use serde_json::json;

#[tokio::main]
async fn main() {
    let mut trackers: Vec<Tracker> = Vec::new();
    let mut sp = Snowplow::new();

    let y = SelfDescribingJson {
        schema: "namespace".to_string(),
        data: json!({
            "name": "John Doe",
            "age": 43u32,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }),
    };

    let emitter = Emitter::new("http://localhost:9090".to_string());

    let x = sp.create_tracker("namespace".to_string(), "id".to_string(), emitter);

    x.track(y).await;
}
