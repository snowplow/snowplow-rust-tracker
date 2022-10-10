use serde_json::json;
use testcontainers::core::WaitFor;
use testcontainers::images::generic::GenericImage;
use testcontainers::{clients::Cli, Container};
use uuid::Uuid;

use snowplow_tracker::{
    ScreenViewEvent, SelfDescribingEvent, SelfDescribingJson, Snowplow, StructuredEvent, Subject,
    TimingEvent,
};

fn micro() -> GenericImage {
    let running_message = WaitFor::message_on_stderr("REST interface bound to /0.0.0.0:9090");

    GenericImage::new("snowplow/snowplow-micro", "latest")
        .with_exposed_port(9090)
        .with_wait_for(running_message.clone())
}

async fn micro_endpoint(micro_url: &str, page: &str) -> serde_json::Value {
    let resp = reqwest::get(micro_url.to_string() + "/micro/" + page)
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap()
}

fn setup(docker: &'_ Cli) -> (Container<'_, GenericImage>, String) {
    let container = docker.run(micro());
    let host_port = container.get_host_port_ipv4(9090);
    let micro_url = format!("http://0.0.0.0:{host_port}");
    (container, micro_url)
}

#[tokio::test]
async fn track_valid_event_to_good() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen")
        .build()
        .unwrap();

    tracker.track(screenview_event, None).await.unwrap();

    let all_events = micro_endpoint(&micro_url, "all").await;
    assert_eq!(1, all_events["good"]);
}

#[tokio::test]
async fn track_event_with_subject() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);
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

    tracker.track(screenview_event, None).await.unwrap();

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

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);

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

    tracker.track(screenview_event, None).await.unwrap();

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
    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, Some(tracker_subject));
    let subject = Subject::builder().user_id("user_2").build().unwrap();

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .subject(subject)
        .build()
        .unwrap();

    tracker.track(screenview_event, None).await.unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;
    let event = &good_events.as_array().unwrap().last().unwrap()["event"];

    assert_eq!("user_2", event["user_id"]);
}

#[tokio::test]
async fn track_screen_view_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);

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

    tracker.track(screenview_event, None).await.unwrap();

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

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);

    let structured_event = StructuredEvent::builder()
        .category("shop")
        .action("add-to-basket")
        .label("Add To Basket")
        .property("pcs")
        .value(2.0)
        .build()
        .unwrap();

    tracker.track(structured_event, None).await.unwrap();

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

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);
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
        .await
        .unwrap();

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

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url, None);

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

    tracker.track(timing_event, None).await.unwrap();

    let good_events = micro_endpoint(&micro_url, "good").await;

    assert_eq!(
        expected_event,
        good_events.as_array().unwrap().last().unwrap()["event"]["unstruct_event"]["data"]
    );
}
