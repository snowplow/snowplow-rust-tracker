use std::sync::{atomic::AtomicUsize, Arc};

use snowplow_tracker::{BatchEmitter, InMemoryEventStore, ScreenViewEvent, Tracker};
use testcontainers::clients::Cli;
use uuid::Uuid;

mod common;
use common::{micro_endpoint, setup, wait_for_events, FlakeyHttpClient};

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

    let mut events = Vec::new();
    for _ in 0..800 {
        events.push(
            ScreenViewEvent::builder()
                .id(Uuid::new_v4())
                .name("a screen view")
                .previous_name("previous screen")
                .ttm("1701147392697")
                .build()
                .unwrap(),
        );
    }

    for event in events {
        tracker.track(event, None).unwrap();
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
            .ttm("1701147392697".to_string())
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

#[tokio::test]
async fn successful_send_after_retry() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let event_store = InMemoryEventStore::new(2, 1);

    let counter = Arc::new(AtomicUsize::new(0));

    let http_client = FlakeyHttpClient {
        micro_url: micro_url.clone(),
        count: counter.clone(),
        number_of_events_to_block: 2,
    };

    let emitter = BatchEmitter::builder()
        .collector_url(&micro_url)
        .event_store(event_store)
        .http_client(http_client)
        .build()
        .unwrap();

    let mut tracker = Tracker::new("ns", "app_id", emitter, None);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen")
        .ttm("1701147392697".to_string())
        .build()
        .unwrap();

    tracker.track(screenview_event, None).unwrap();

    wait_for_events(&micro_url, "good", 1).await;

    tracker.close_emitter().unwrap();

    let all_events = micro_endpoint(&micro_url, "all").await;

    assert!(counter.load(std::sync::atomic::Ordering::SeqCst) == 2);
    assert_eq!(1, all_events["good"]);
}
