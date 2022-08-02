use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU64;
use serde_json::json;
use snowplow_rust_tracker::emitter::Emitter;
use snowplow_rust_tracker::event::ScreenViewEvent;
use snowplow_rust_tracker::event::StructuredEvent;
use snowplow_rust_tracker::tracker::Tracker;
use snowplow_rust_tracker::event_store::InMemoryEventStore;

#[tokio::main]
async fn main() {
    let _batch_id = Arc::new(AtomicU64::new(1));
    let event_store = InMemoryEventStore {
        store: Arc::new(Mutex::new(Vec::new()))
    };
    let cache = event_store.store.clone();
    let emitter = Emitter::new("http://localhost:9090", event_store);
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

    println!("--- DEBUGGING ---");
    println!("Self Describing Event: {:?}", self_desc_event);
    println!("Structured Event: {:?}", struct_event);
    println!("Screen View: {:?}", screen_view);
    println!("Event Store: {:?}", cache.lock().unwrap());
}
