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

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::payload::{EventType, SelfDescribingEventData, PayloadBuilder, Payload, SelfDescribingJson};

/// Trait implemented by event types that enables the tracker to build their payload to be sent to the Collector.
pub trait EventBuildable {
    fn build_payload(self, payload_builder: PayloadBuilder) -> Payload;
}

/// Event to track custom information that does not fit into the out-of-the box events.
///
/// Self-describing events are a [data structure based on JSON Schemas](https://docs.snowplowanalytics.com/docs/understanding-tracking-design/understanding-schemas-and-validation/) and can have arbitrarily many fields.
/// To define your own custom self-describing event, you must create a JSON schema for that event and upload it to an [Iglu Schema Repository](https://github.com/snowplow/iglu) using [igluctl](https://docs.snowplowanalytics.com/docs/open-source-components-and-applications/iglu/) (or if a Snowplow BDP customer, you can use the [Snowplow BDP Console UI](https://docs.snowplowanalytics.com/docs/understanding-tracking-design/managing-data-structures/) or [Data Structures API](https://docs.snowplowanalytics.com/docs/understanding-tracking-design/managing-data-structures-via-the-api-2/)).
/// Snowplow uses the schema to validate that the JSON containing the event properties is well-formed.
#[derive(Serialize, Deserialize, Builder)]
pub struct SelfDescribingEvent {
    /// A valid Iglu schema path.
    ///
    /// This must point to the location of the custom event’s schema, of the format: `iglu:{vendor}/{name}/{format}/{version}`.
    #[builder(setter(into))]
    pub schema: String,

    /// The custom data for the event.
    ///
    /// This data must conform to the schema specified in the schema argument, or the event will fail validation and land in bad rows.
    #[builder(setter(into))]
    pub data: Value,
}

impl SelfDescribingEvent {
    pub fn builder() -> SelfDescribingEventBuilder {
        SelfDescribingEventBuilder::default()
    }
}

impl EventBuildable for SelfDescribingEvent {
    fn build_payload(self, payload_builder: PayloadBuilder) -> Payload {
        payload_builder
            .e(EventType::SelfDescribingEvent)
            .ue_pr(SelfDescribingEventData::new(SelfDescribingJson::new(
                &self.schema,
                self.data,
            )))
            .build()
            .unwrap()
    }
}

/// Event to capture custom consumer interactions without the need to define a custom schema.
#[derive(Serialize, Deserialize, Builder)]
pub struct StructuredEvent {
    /// Name you for the group of objects you want to track e.g. "media", "ecomm".
    #[builder(setter(into))]
    pub category: String,

    /// Defines the type of user interaction for the web object.
    ///
    /// E.g., "play-video", "add-to-basket".
    #[builder(setter(into))]
    pub action: String,

    /// Describes the object or the action performed on it.
    ///
    /// This might be the quantity of an item added to basket
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,

    /// Identifies the specific object being actioned.
    ///
    /// E.g., ID of the video being played, or the SKU or the product added-to-basket.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Identifies the specific object being actioned.
    ///
    /// E.g., ID of the video being played, or the SKU or the product added-to-basket.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    // serde isn't happy with u128 and I'm not sure why
    pub value: Option<f64>,
}

impl StructuredEvent {
    pub fn builder() -> StructuredEventBuilder {
        StructuredEventBuilder::default()
    }
}

impl EventBuildable for StructuredEvent {
    fn build_payload(self, payload_builder: PayloadBuilder) -> Payload {
        payload_builder
            .e(EventType::StructuredEvent)
            .se_ca(self.category)
            .se_ac(self.action)
            .se_pr(self.property)
            .se_la(self.label)
            .se_va(if let Some(value) = self.value { Some(value.to_string()) } else { None })
            .build()
            .unwrap()
    }
}

/// Event to track user viewing a screen within the application.
/// 
/// It is a self-describing event with the schema "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0"
#[derive(Serialize, Deserialize, Builder)]
pub struct ScreenViewEvent {
    /// The name of the screen viewed.
    #[builder(setter(into))]
    pub name: String,

    /// The id (UUID v4) of screen that was viewed.
    #[builder(setter(into))]
    pub id: Uuid,

    /// The type of screen that was viewed.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "type"))]
    pub screen_type: Option<String>,

    /// The name of the previous screen that was viewed.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "previousName"))]
    pub previous_name: Option<String>,

    /// The type of screen that was viewed.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "previousType"))]
    pub previous_type: Option<String>,

    /// The id (UUID v4) of the previous screen that was viewed.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "previousId"))]
    pub previous_id: Option<Uuid>,

    /// The type of transition that led to the screen being viewed.
    #[builder(setter(into), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "transitionType"))]
    pub transition_type: Option<String>,
}

impl ScreenViewEvent {
    pub fn builder() -> ScreenViewEventBuilder {
        ScreenViewEventBuilder::default()
    }
}

impl EventBuildable for ScreenViewEvent {
    fn build_payload(self, payload_builder: PayloadBuilder) -> Payload {
        let event = SelfDescribingEvent::builder()
            .schema("iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0")
            .data(json!(self))
            .build()
            .unwrap();
        event.build_payload(payload_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_structured_event() {
        let event = StructuredEvent::builder()
            .category("test")
            .action("test_action")
            .build()
            .unwrap();

        assert_eq!("test", event.category);
        assert_eq!("test_action", event.action);
    }

    #[test]
    fn builds_payload_for_self_describing_event() {
        let event = SelfDescribingEvent {
            schema: "schema.com".to_string(),
            data: json!({"targetUrl": "http://a-target-url.com"})
        };
        let payload_builder = payload_builder();
        let payload = event.build_payload(payload_builder);
        let ue_pr = payload.ue_pr.unwrap();
        assert_eq!(ue_pr.schema, "iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0");
        assert_eq!(ue_pr.data.schema, "schema.com");
    }

    #[test]
    fn builds_payload_for_structured_event() {
        let event = StructuredEvent::builder()
            .category("shop")
            .action("add-to-basket")
            .label("Add To Basket".to_string())
            .property("pcs".to_string())
            .value(2.0)
            .build()
            .unwrap();
        let payload_builder = payload_builder();
        let payload = event.build_payload(payload_builder);
        assert_eq!(payload.se_ca.unwrap(), "shop");
        assert_eq!(payload.se_ac.unwrap(), "add-to-basket");
        assert_eq!(payload.se_la.unwrap(), "Add To Basket");
        assert_eq!(payload.se_pr.unwrap(), "pcs");
        assert_eq!(payload.se_va.unwrap(), "2");
    }

    #[test]
    fn builds_payload_for_screen_view() {
        let event = ScreenViewEvent::builder()
            .id(Uuid::new_v4())
            .name("a screen view")
            .build()
            .unwrap();
        let payload_builder = payload_builder();
        let payload = event.build_payload(payload_builder);
        let ue_pr = payload.ue_pr.unwrap();
        assert_eq!(ue_pr.data.schema, "iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0");
    }

    fn payload_builder() -> PayloadBuilder {
        Payload::builder()
            .p("platform".to_string())
            .tv("0.1.9".to_string())
            .eid(Uuid::new_v4())
            .dtm("1".to_string())
            .stm("1".to_string())
            .aid("test".to_string())
    }
}
