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
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::Error;
use crate::payload::{EventType, PayloadBuilder, SelfDescribingEventData, SelfDescribingJson};
use crate::subject::Subject;

/// Trait implemented by event types that enables the event to add itself to a PayloadBuilder.
pub trait PayloadAddable {
    fn add_to_payload(self, payload_builder: PayloadBuilder) -> PayloadBuilder;
    fn subject(&self) -> &Option<Subject>;
}

/// Event to track custom information that does not fit into the out-of-the box events.
///
/// Self-describing events are a [data structure based on JSON Schemas](https://docs.snowplow.io/docs/understanding-tracking-design/understanding-schemas-and-validation/) and can have arbitrarily many fields.
/// Snowplow uses the schema to validate that the JSON containing the event properties is well-formed.
#[derive(Serialize, Deserialize, Builder)]
#[builder(setter(into))]
#[builder(build_fn(error = "Error"))]
pub struct SelfDescribingEvent {
    /// A valid Iglu schema path.
    ///
    /// This must point to the location of the custom event’s schema, of the format: `iglu:{vendor}/{name}/{format}/{version}`.
    pub schema: String,

    /// The custom data for the event.
    ///
    /// This data must conform to the schema specified in the schema argument, or the event will fail validation and land in bad rows.
    pub data: Value,

    /// The [Subject] of the event.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub subject: Option<Subject>,
}

impl SelfDescribingEvent {
    pub fn builder() -> SelfDescribingEventBuilder {
        SelfDescribingEventBuilder::default()
    }
}

impl PayloadAddable for SelfDescribingEvent {
    fn add_to_payload(self, payload_builder: PayloadBuilder) -> PayloadBuilder {
        payload_builder
            .e(EventType::SelfDescribingEvent)
            .ue_pr(SelfDescribingEventData::new(SelfDescribingJson::new(
                &self.schema,
                self.data,
            )))
    }

    fn subject(&self) -> &Option<Subject> {
        &self.subject
    }
}

/// Event to capture custom consumer interactions without the need to define a custom schema.
#[derive(Serialize, Deserialize, Builder, Debug, Clone)]
#[builder(setter(into, strip_option))]
#[builder(build_fn(error = "Error"))]
pub struct StructuredEvent {
    /// Name you for the group of objects you want to track e.g. "media", "ecomm".
    #[serde(rename(serialize = "se_ca"))]
    pub category: String,

    /// Defines the type of user interaction for the web object.
    ///
    /// E.g., "play-video", "add-to-basket".
    #[serde(rename(serialize = "se_ac"))]
    pub action: String,

    /// Describes the object or the action performed on it.
    ///
    /// This might be the quantity of an item added to basket
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "se_pr"))]
    pub property: Option<String>,

    /// Identifies the specific object being actioned.
    ///
    /// E.g., ID of the video being played, or the SKU or the product added-to-basket.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "se_la"))]
    pub label: Option<String>,

    /// Identifies the specific object being actioned.
    ///
    /// E.g., ID of the video being played, or the SKU or the product added-to-basket.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "se_va"))]
    #[serde(serialize_with = "optional_f64_to_string")]
    pub value: Option<f64>,

    /// The [Subject] of the event.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub subject: Option<Subject>,
}

// Serializer to convert the optional f64 to the JSON `String` type
// expected by the collector, rather than the default JSON `Number`
fn optional_f64_to_string<S>(num: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(num) = num {
        serializer.serialize_str(&num.to_string())
    } else {
        serializer.serialize_none()
    }
}

impl StructuredEvent {
    pub fn builder() -> StructuredEventBuilder {
        StructuredEventBuilder::default()
    }
}

impl PayloadAddable for StructuredEvent {
    fn add_to_payload(self, payload_builder: PayloadBuilder) -> PayloadBuilder {
        payload_builder
            .e(EventType::StructuredEvent)
            .structured_event(self)
    }

    fn subject(&self) -> &Option<Subject> {
        &self.subject
    }
}

/// Event to track user viewing a screen within the application.
///
/// It is a self-describing event with the schema "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0"
#[derive(Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option))]
#[builder(build_fn(error = "Error"))]
pub struct ScreenViewEvent {
    /// The name of the screen viewed.
    pub name: String,

    /// The id (UUID v4) of screen that was viewed.
    pub id: Uuid,

    /// The type of screen that was viewed.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "type"))]
    pub screen_type: Option<String>,

    /// The name of the previous screen that was viewed.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_name: Option<String>,

    /// The type of screen that was viewed.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_type: Option<String>,

    /// The id (UUID v4) of the previous screen that was viewed.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_id: Option<Uuid>,

    /// The type of transition that led to the screen being viewed.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_type: Option<String>,

    /// The [Subject] of the event.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub subject: Option<Subject>,
}

impl ScreenViewEvent {
    pub fn builder() -> ScreenViewEventBuilder {
        ScreenViewEventBuilder::default()
    }
}

impl PayloadAddable for ScreenViewEvent {
    fn add_to_payload(self, payload_builder: PayloadBuilder) -> PayloadBuilder {
        let event = SelfDescribingEvent {
            schema: "iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0".to_string(),
            data: json!(self),
            subject: self.subject,
        };

        event.add_to_payload(payload_builder)
    }

    fn subject(&self) -> &Option<Subject> {
        &self.subject
    }
}

/// Event to track user timing events, such as how long resources take to load.
///
/// It is a self-describing event with the schema "iglu:com.snowplowanalytics.snowplow/timing/jsonschema/1-0-0"
#[derive(Serialize, Deserialize, Builder, Default)]
#[builder(setter(into, strip_option), default)]
#[builder(build_fn(error = "Error"))]
pub struct TimingEvent {
    /// The category of the timed event
    pub category: String,

    /// What is being measured e.g. "load resource"
    pub variable: String,

    /// The number of milliseconds in elapsed time
    pub timing: i64,

    /// An optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// The [Subject] of the event.
    #[builder(default)]
    #[serde(skip_serializing)]
    pub subject: Option<Subject>,
}

impl TimingEvent {
    pub fn builder() -> TimingEventBuilder {
        TimingEventBuilder::default()
    }
}

impl PayloadAddable for TimingEvent {
    fn add_to_payload(self, payload_builder: PayloadBuilder) -> PayloadBuilder {
        let event = SelfDescribingEvent {
            schema: "iglu:com.snowplowanalytics.snowplow/timing/jsonschema/1-0-0".to_string(),
            data: json!(self),
            subject: self.subject,
        };

        event.add_to_payload(payload_builder)
    }

    fn subject(&self) -> &Option<Subject> {
        &self.subject
    }
}

#[cfg(test)]
mod tests {
    use crate::payload::Payload;

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
        let event = SelfDescribingEvent::builder()
            .schema("schema.com")
            .data(json!({}))
            .subject(Subject {
                user_id: Some("user_1".to_string()),
                ..Subject::default()
            })
            .build()
            .unwrap();

        let payload_builder = payload_builder();

        assert_eq!(&event.subject().clone().unwrap().user_id.unwrap(), "user_1");

        let payload = event.add_to_payload(payload_builder).build().unwrap();
        let ue_pr = payload.ue_pr.unwrap();

        assert_eq!(
            ue_pr.schema,
            "iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0"
        );
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
            .subject(Subject {
                user_id: Some("user_1".to_string()),
                ..Subject::default()
            })
            .build()
            .unwrap();
        let payload_builder = payload_builder();

        assert_eq!(&event.subject().clone().unwrap().user_id.unwrap(), "user_1");

        let payload = event.add_to_payload(payload_builder).build().unwrap();
        let event = payload.structured_event.unwrap();

        assert_eq!(event.category, "shop");
        assert_eq!(event.action, "add-to-basket");
        assert_eq!(event.label.unwrap(), "Add To Basket");
        assert_eq!(event.property.unwrap(), "pcs");
        assert_eq!(event.value.unwrap(), 2_f64);
    }

    #[test]
    fn builds_payload_for_screen_view() {
        let event = ScreenViewEvent::builder()
            .id(Uuid::new_v4())
            .name("a screen view")
            .subject(Subject {
                user_id: Some("user_1".to_string()),
                ..Subject::default()
            })
            .build()
            .unwrap();
        let payload_builder = payload_builder();

        assert_eq!(&event.subject().clone().unwrap().user_id.unwrap(), "user_1");

        let payload = event.add_to_payload(payload_builder).build().unwrap();
        let ue_pr = payload.ue_pr.unwrap();
        assert_eq!(
            ue_pr.data.schema,
            "iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0"
        );
    }

    #[test]
    fn builds_payload_for_timing_event() {
        let event = TimingEvent::builder()
            .category("fetch_resource")
            .variable("map_loaded")
            .timing(1423)
            .label("Time to fetch map resource")
            .subject(Subject {
                user_id: Some("user_1".to_string()),
                ..Subject::default()
            })
            .build()
            .unwrap();
        let payload_builder = payload_builder();

        assert_eq!(&event.subject().clone().unwrap().user_id.unwrap(), "user_1");

        let payload = event.add_to_payload(payload_builder).build().unwrap();
        let expected = SelfDescribingJson {
            schema: "iglu:com.snowplowanalytics.snowplow/timing/jsonschema/1-0-0".to_string(),
            data: json!({
                "category": "fetch_resource",
                "variable": "map_loaded",
                "timing": 1423_i64,
                "label": "Time to fetch map resource"
            }),
        };
        let data = payload.ue_pr.unwrap().data;
        assert_eq!(data.schema, expected.schema);
        assert_eq!(data.data, expected.data);
    }

    fn payload_builder() -> PayloadBuilder {
        Payload::builder()
            .p("platform".to_string())
            .tv(format!("rust-{}", env!("CARGO_PKG_VERSION")))
            .eid(Uuid::new_v4())
            .dtm("1".to_string())
            .stm("1".to_string())
            .aid("test".to_string())
    }

    #[test]
    fn builder_error_shows_first_encountered_missing_field() {
        let event = StructuredEvent::builder().build().unwrap_err();
        assert_eq!(event.to_string(), "Field not initialized: category");

        let event = StructuredEvent::builder()
            .category("category")
            .build()
            .unwrap_err();
        assert_eq!(event.to_string(), "Field not initialized: action");
    }
}
