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
use snowplow_tracker::{Snowplow, Event, ScreenViewEvent, StructuredEvent, SelfDescribingJson, SelfDescribingEvent};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let tracker = Snowplow::create_tracker("ns", "app_id", "http://localhost:9090");

    let self_desc_event_id = tracker.track(
        Event::SelfDescribing(
            SelfDescribingEvent {
                schema: "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0".to_string(),
                data: json!({"name": "test", "id": "something else"})
            }
        ), 
Some(vec![
            SelfDescribingJson::new("iglu:org.schema/WebPage/jsonschema/1-0-0", json!({"keywords": ["tester"]}))
        ])
    ).await.unwrap();

    let struct_event_id = tracker
        .track(
            Event::Structured(
                StructuredEvent::builder()
                    .category("shop")
                    .action("add-to-basket")
                    .label("Add To Basket".to_string())
                    .property("pcs".to_string())
                    .value(2.0)
                    .build()
                    .unwrap(),
            ),
            None
        ).await
        .unwrap();

    let screen_view_event_id = tracker
        .track(
            Event::ScreenView(
                ScreenViewEvent::builder()
                    .id(Uuid::new_v4())
                    .name("a screen view")
                    .previous_name("previous screen".to_string())
                    .build()
                    .unwrap()
            ),
            None
        ).await
        .unwrap();

    println!("--- DEBUGGING ---");
    println!("Self Describing Event: {}", self_desc_event_id);
    println!("Structured Event: {}", struct_event_id);
    println!("Screen View: {}", screen_view_event_id);
}
