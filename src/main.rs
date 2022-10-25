// Copyright (c) 2022 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0,
// and you may not use this file except in compliance with the Apache License Version 2.0.
// You may obtain a copy of the Apache License Version 2.0 at http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the Apache License Version 2.0 is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the Apache License Version 2.0 for the specific language governing permissions and limitations there under.

use serde_json::json;
use snowplow_tracker::{
    ScreenViewEvent, SelfDescribingEvent, SelfDescribingJson, Snowplow, StructuredEvent,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let tracker = Snowplow::create_tracker("ns", "app_id", "http://localhost:9090", None);

    // Tracking a Self-Describing event with event context
    let self_describing_event = match SelfDescribingEvent::builder()
        .schema("iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0")
        .data(json!({"name": "test", "id": "something else"}))
        .build()
    {
        Ok(event) => event,
        Err(e) => panic!("{e}"), // your error handling here
    };

    let event_context = Some(vec![SelfDescribingJson::new(
        "iglu:org.schema/WebPage/jsonschema/1-0-0",
        json!({"keywords": ["tester"]}),
    )]);

    let self_desc_event_id = match tracker.track(self_describing_event, event_context).await {
        Ok(uuid) => uuid,
        Err(e) => panic!("{e}"), // your error handling here
    };

    // Tracking a Structured event
    let structured_event = match StructuredEvent::builder()
        .category("shop")
        .action("add-to-basket")
        .label("Add To Basket")
        .property("pcs")
        .value(2.0)
        .build()
    {
        Ok(event) => event,
        Err(e) => panic!("{e}"), // your error handling here
    };

    let struct_event_id = match tracker.track(structured_event, None).await {
        Ok(uuid) => uuid,
        Err(e) => panic!("{e}"), // your error handling here
    };

    // Tracking a Screen View event
    let screen_view_event = match ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .previous_name("previous name")
        .build()
    {
        Ok(event) => event,
        Err(e) => panic!("{e}"), // your error handling here
    };

    let screen_view_event_id = match tracker.track(screen_view_event, None).await {
        Ok(uuid) => uuid,
        Err(e) => panic!("{e}"), // your error handling here
    };

    println!("--- DEBUGGING ---");
    println!("Self Describing Event: {}", self_desc_event_id);
    println!("Structured Event: {}", struct_event_id);
    println!("Screen View: {}", screen_view_event_id);
}
