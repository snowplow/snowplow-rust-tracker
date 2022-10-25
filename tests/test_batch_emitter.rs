use testcontainers::clients::Cli;
use uuid::Uuid;

use snowplow_tracker::{BatchEmitter, InMemoryEventStore, ScreenViewEvent, Tracker};

mod common;
use common::{micro_endpoint, setup, wait_for_events};

#[tokio::test]
async fn send_batches() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let event_store = InMemoryEventStore::new(800, 50);

    let emitter = BatchEmitter::builder()
        .collector_url(&micro_url)
        .event_store(event_store)
        .build()
        .unwrap();

    let mut tracker = Tracker::new("ns", "app_id", emitter, None);

    for _ in 0..800 {
        let screenview_event = ScreenViewEvent::builder()
            .id(Uuid::new_v4())
            .name("a screen view")
            .previous_name("previous screen")
            .build()
            .unwrap();

        tracker.track(screenview_event, None).unwrap();
    }

    wait_for_events(&micro_url, "good", 800).await;
    tracker.close_emitter().unwrap();

    let all_events = micro_endpoint(&micro_url, "all").await;

    assert_eq!(800, all_events["good"]);
}

#[tokio::test]
async fn flush_emitter() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let event_store = InMemoryEventStore::new(500, 400);

    let emitter = BatchEmitter::builder()
        .collector_url(&micro_url)
        .event_store(event_store)
        .build()
        .unwrap();

    let mut tracker = Tracker::new("ns", "app_id", emitter, None);

    for _ in 0..350 {
        let screenview_event = ScreenViewEvent::builder()
            .id(Uuid::new_v4())
            .name("a screen view")
            .previous_name("previous screen")
            .build()
            .unwrap();

        tracker.track(screenview_event, None).unwrap();
    }

    tracker.flush().unwrap();
    wait_for_events(&micro_url, "good", 350).await;
    tracker.close_emitter().unwrap();

    let all_events = micro_endpoint(&micro_url, "all").await;

    assert_eq!(350, all_events["good"]);
}
