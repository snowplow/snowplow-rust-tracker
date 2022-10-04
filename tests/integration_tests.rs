use serde_json::json;
use testcontainers::core::WaitFor;
use testcontainers::images::generic::GenericImage;
use testcontainers::{clients::Cli, Container};
use uuid::Uuid;

use snowplow_tracker::{
    ScreenViewEvent, SelfDescribingEvent, SelfDescribingJson, Snowplow, StructuredEvent,
};

fn get_micro() -> GenericImage {
    let running_message = WaitFor::message_on_stderr("REST interface bound to /0.0.0.0:9090");

    GenericImage::new("snowplow/snowplow-micro", "latest")
        .with_exposed_port(9090)
        .with_wait_for(running_message.clone())
}

async fn get_micro_endpoint(micro_url: &str, page: &str) -> serde_json::Value {
    let resp = reqwest::get(micro_url.to_string() + "/micro/" + page)
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap()
}

fn setup(docker: &'_ Cli) -> (Container<'_, GenericImage>, String) {
    let container = docker.run(get_micro());
    let host_port = container.get_host_port_ipv4(9090);
    let micro_url = format!("http://0.0.0.0:{host_port}");
    (container, micro_url)
}

#[tokio::test]
async fn track_valid_event_to_good() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen".to_string())
        .build()
        .unwrap();

    tracker.track(screenview_event, None).await;

    let all_events = get_micro_endpoint(&micro_url, "all").await;
    assert_eq!(1, all_events["good"]);
}

#[tokio::test]
async fn track_screen_view_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url);

    let screenview_event = ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous screen".to_string())
        .build()
        .unwrap();

    let expected_event = json!({
        "schema": "iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0".to_string(),
        "data" : serde_json::to_value(&screenview_event).unwrap(),
    });

    tracker.track(screenview_event, None).await.unwrap();

    let good_events = get_micro_endpoint(&micro_url, "good").await;

    assert_eq!(
        expected_event,
        good_events.as_array().unwrap().last().unwrap()["event"]["unstruct_event"]["data"]
    );
}

#[tokio::test]
async fn track_structured_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url);

    let structured_event = StructuredEvent::builder()
        .category("shop")
        .action("add-to-basket")
        .label("Add To Basket".to_string())
        .property("pcs".to_string())
        .value(2.0)
        .build()
        .unwrap();

    let expected_event = serde_json::to_value(&structured_event).unwrap();

    tracker.track(structured_event, None).await.unwrap();

    let good_events = get_micro_endpoint(&micro_url, "good").await;

    let properties = vec!["category", "action", "label", "property", "value"];
    for prop in properties {
        assert_eq!(
            expected_event[prop],
            good_events.as_array().unwrap().last().unwrap()["event"][format!("se_{prop}")]
        );
    }
}

#[tokio::test]
async fn track_self_describing_event() {
    let docker = Cli::default();
    let (_container, micro_url) = setup(&docker);

    let tracker = Snowplow::create_tracker("ns", "app_id", &micro_url);
    tracker
        .track(
            SelfDescribingEvent {
                schema: "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0"
                    .to_string(),
                data: json!({"name": "test", "id": "something else"}),
            },
            Some(vec![SelfDescribingJson::new(
                "iglu:org.schema/WebPage/jsonschema/1-0-0",
                json!({"keywords": ["tester"]}),
            )]),
        )
        .await;

    let good_events = get_micro_endpoint(&micro_url, "good").await;
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
