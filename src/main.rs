use serde_json::json;
use snowplow_rust_tracker::snowplow::emitter::Emitter;
use snowplow_rust_tracker::snowplow::event::ScreenViewEvent;
use snowplow_rust_tracker::snowplow::event::StructuredEvent;
use snowplow_rust_tracker::snowplow::tracker::Tracker;

#[tokio::main]
async fn main() {
    let emitter = Emitter::new("http://localhost:9090");
    let tracker = Tracker::new("namespace", "id", emitter);

    let self_desc_event = tracker
        .track_self_describing_event(
            "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0",
            &json!({"name": "test", "id": "something else"}).to_string(),
        )
        .await;

    let struct_event = tracker
        .track_struct_event(
            StructuredEvent::builder()
                .category("test")
                .action("test_action")
                .build()
                .unwrap(),
        )
        .await;

    let screen_view = tracker
        .track_screen_view(
            ScreenViewEvent::builder()
                .id("this is")
                .name("a screen view")
                .build()
                .unwrap(),
        )
        .await;

    println!("Self Describing Event: {:?}", self_desc_event);
    println!("Structured Event: {:?}", struct_event);
    println!("Screen View: {:?}", screen_view);
}
