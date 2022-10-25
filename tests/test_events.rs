use serde_json::json;
use testcontainers::clients::Cli;
use uuid::Uuid;

use snowplow_tracker::{
    BatchEmitter, InMemoryEventStore, ScreenViewEvent, SelfDescribingEvent, SelfDescribingJson,
    StructuredEvent, Subject, TimingEvent, Tracker,
};

mod common;
use common::{micro_endpoint, setup, wait_for_events};

// A tracker with batch/queue size of 1, so it sends every event immediately
fn test_tracker(
    micro_endpoint: &str,
    subject: Option<Subject>,
    queue_capacity: Option<usize>,
    batch_size: Option<usize>,
) -> Tracker {
    let event_store = InMemoryEventStore::new(queue_capacity.unwrap_or(1), batch_size.unwrap_or(1));
    let emitter = BatchEmitter::builder()
        .collector_url(micro_endpoint)
        .event_store(event_store)
        .build()
        .unwrap();

    Tracker::new("test-namespace", "test-app-id", emitter, subject)
}

#[tokio::test]
async fn track_valid_event_to_good() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen")
        .build()
        .unwrap();

    tracker.track(screenview_event, None).unwrap();

    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let req = micro_endpoint(&micro_url, "good").await;
    let good_events = req.as_array().unwrap();
    assert_eq!(1, good_events.len());
}

#[tokio::test]
async fn track_event_with_subject() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);
    let domain_user_id = Uuid::new_v4();
    let network_user_id = Uuid::new_v4();
    let session_user_id = Uuid::new_v4();
    let subject = Subject::builder()
        .user_id("user_1")
        .timezone("Europe/London")
        .language("en-gb")
        .ip_address("0.0.0.0")
        .user_agent("Mozilla/Firefox")
        .domain_user_id(domain_user_id)
        .network_user_id(network_user_id)
        .session_user_id(session_user_id)
        .build()
        .unwrap();

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen")
        .subject(subject)
        .build()
        .unwrap();

    tracker.track(screenview_event, None).unwrap();

    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let event = &good_events.as_array().unwrap().last().unwrap()["event"];

    assert_eq!("user_1", event["user_id"]);
    assert_eq!("Europe/London", event["os_timezone"]);
    assert_eq!("en-gb", event["br_lang"]);
    assert_eq!("0.0.0.0", event["user_ipaddress"]);
    assert_eq!("Mozilla/Firefox", event["useragent"]);
    assert_eq!(domain_user_id.to_string(), event["domain_userid"]);
    assert_eq!(network_user_id.to_string(), event["network_userid"]);
    assert_eq!(session_user_id.to_string(), event["domain_sessionid"]);
}

#[tokio::test]
async fn track_event_with_partial_subject() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);

    let subject = Subject::builder()
        .user_id("user_1")
        .timezone("Europe/London")
        .language("en-gb")
        .build()
        .unwrap();

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen")
        .subject(subject)
        .build()
        .unwrap();

    tracker.track(screenview_event, None).unwrap();
    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let event = &good_events.as_array().unwrap().last().unwrap()["event"];

    // Fields sent in Subject
    assert_eq!("user_1".to_string(), event["user_id"]);
    assert_eq!("Europe/London".to_string(), event["os_timezone"]);
    assert_eq!("en-gb".to_string(), event["br_lang"]);

    // Fields not sent in Subject, not set by Enrich
    assert_eq!(serde_json::Value::Null, event["useragent"]);
    assert_eq!(serde_json::Value::Null, event["domain_userid"]);

    // Fields not sent in subject, set by Enrich
    assert_ne!(serde_json::Value::Null, event["user_ipaddress"]);
    assert_ne!(serde_json::Value::Null, event["network_userid"]);
}

#[tokio::test]
async fn event_subject_overrides_tracker_subject() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker_subject = Subject::builder().user_id("user_1").build().unwrap();
    let mut tracker = test_tracker(&micro_url, Some(tracker_subject), None, None);
    let subject = Subject::builder().user_id("user_2").build().unwrap();

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .subject(subject)
        .build()
        .unwrap();

    tracker.track(screenview_event, None).unwrap();
    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let event = &good_events.as_array().unwrap().last().unwrap()["event"];

    assert_eq!("user_2", event["user_id"]);
}

#[tokio::test]
async fn track_screen_view_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .screen_type("feed")
        .previous_name("previous screen")
        .previous_type("carousel")
        .previous_id(Uuid::new_v4())
        .transition_type("navigation")
        .build()
        .unwrap();

    let expected_event = json!({
        "schema": "iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0".to_string(),
        "data" : serde_json::to_value(&screenview_event).unwrap(),
    });

    tracker.track(screenview_event, None).unwrap();
    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;

    assert_eq!(
        expected_event,
        good_events.as_array().unwrap().last().unwrap()["event"]["unstruct_event"]["data"]
    );
}

#[tokio::test]
async fn track_structured_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);

    let structured_event = StructuredEvent::builder()
        .category("shop")
        .action("add-to-basket")
        .label("Add To Basket")
        .property("pcs")
        .value(2.0)
        .build()
        .unwrap();

    tracker.track(structured_event, None).unwrap();
    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let event = &good_events.as_array().unwrap().last().unwrap()["event"];

    assert_eq!(event["se_category"], "shop");
    assert_eq!(event["se_action"], "add-to-basket");
    assert_eq!(event["se_label"], "Add To Basket");
    assert_eq!(event["se_property"], "pcs");
    assert_eq!(event["se_value"], 2.0);
}

#[tokio::test]
async fn track_self_describing_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);
    tracker
        .track(
            SelfDescribingEvent::builder()
                .schema("iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0")
                .data(json!({"name": "test", "id": "something else"}))
                .build()
                .unwrap(),
            Some(vec![SelfDescribingJson::new(
                "iglu:org.schema/WebPage/jsonschema/1-0-0",
                json!({"keywords": ["tester"]}),
            )]),
        )
        .unwrap();

    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let received_event = good_events.as_array().unwrap().last().unwrap();

    let expected_unstruct_event = json!({
        "data": {
          "id": "something else",
          "name": "test"
        },
        "schema": "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0"
    });

    assert_eq!(
        received_event["event"]["unstruct_event"]["data"],
        expected_unstruct_event
    );

    let expected_context = json!({
        "data": {
            "keywords": [
                "tester"
            ]
        },
        "schema": "iglu:org.schema/WebPage/jsonschema/1-0-0",
    });

    assert_eq!(
        received_event["event"]["contexts"]["data"]
            .as_array()
            .unwrap()
            .first()
            .unwrap(),
        &expected_context
    );
}

#[tokio::test]
async fn track_timing_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, None, None);

    let timing_event = TimingEvent::builder()
        .category("fetch_resource")
        .variable("map_loaded")
        .timing(1423)
        .label("Time to fetch map resource")
        .build()
        .unwrap();

    let expected_event = json!({
        "schema": "iglu:com.snowplowanalytics.snowplow/timing/jsonschema/1-0-0".to_string(),
        "data" : serde_json::to_value(&timing_event).unwrap(),
    });

    tracker.track(timing_event, None).unwrap();
    wait_for_events(&micro_url, "good", 1).await;
    tracker.close_emitter().unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;

    assert_eq!(
        expected_event,
        good_events.as_array().unwrap().last().unwrap()["event"]["unstruct_event"]["data"]
    );
}

#[tokio::test]
async fn track_many_events() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let mut tracker = test_tracker(&micro_url, None, Some(10), Some(2));
    let mut expected: Vec<serde_json::Value> = vec![];

    for i in 0..10 {
        let event = TimingEvent::builder()
            .category("fetch_resource")
            .variable("map_loaded")
            .timing(i)
            .label("Time to fetch map resource")
            .build()
            .unwrap();

        let expected_event = json!({
            "schema": "iglu:com.snowplowanalytics.snowplow/timing/jsonschema/1-0-0".to_string(),
            "data" : serde_json::to_value(&event).unwrap(),
        });

        tracker.track(event, None).unwrap();
        expected.push(expected_event);
    }

    wait_for_events(&micro_url, "good", 10).await;
    tracker.close_emitter().unwrap();
    let good_events = micro_endpoint(&micro_url, "good").await;

    expected.iter().for_each(|expected_event| {
        assert!(good_events
            .as_array()
            .unwrap()
            .iter()
            .any(
                |received_event| received_event["event"]["unstruct_event"]["data"]
                    == *expected_event
            ))
    })
}
