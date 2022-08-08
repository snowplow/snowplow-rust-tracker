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

use crate::emitter::Emitter;
use crate::payload::{Payload, PayloadBuilder, SelfDescribingEventData, ContextData, EventType, SelfDescribingJson};
use crate::event::{SelfDescribingEvent, Event, StructuredEvent, ScreenViewEvent};
use uuid::Uuid;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use serde_json::json;

pub struct TrackerConfig {
    pub platform: String,
    pub version: String,
    pub encode_base_64: bool,
}

/// Snowplow tracker instance used to track events to the Snowplow Collector
pub struct Tracker {
    /// Tracker namespace that identifies the tracker within the app
    pub namespace: String,
    /// Application ID
    pub app_id: String,
    /// Emitter used to send events to the Collector
    pub emitter: Emitter,
    /// Additional tracker config
    pub config: TrackerConfig,
}

impl Tracker {
    pub fn new(namespace: &str, app_id: &str, emitter: Emitter) -> Tracker {
        Tracker {
            namespace: namespace.to_string(),
            app_id: app_id.to_string(),
            emitter,
            config: TrackerConfig {
                platform: "pc".to_string(),
                version: "rust-0.1.0".to_string(),
                encode_base_64: false,
            }
        }
    }

    /// Tracks a Snowplow event with optional context entities and sends it to the Snowplow collector.
    pub async fn track(&self, event: Event, context: Option<Vec<SelfDescribingJson>>) -> Option<Uuid> {
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let event_id = Uuid::new_v4();

        let mut payload_builder = Payload::builder()
            .p(self.config.platform.clone())
            .tv(self.config.version.clone())
            .eid(event_id.clone())
            .dtm(since_the_epoch.as_millis().to_string())
            .stm(since_the_epoch.as_millis().to_string())
            .aid(self.app_id.clone());

        if let Some(context) = context {
            payload_builder = payload_builder.co(ContextData::new(context.to_vec()));
        }

        let payload = self.build_event_payload(event, payload_builder);
        if self.emitter.add(payload).await.is_ok() {
            Some(event_id)
        } else {
            None
        }
    }

    fn build_event_payload(&self, event: Event, payload_builder: PayloadBuilder) -> Payload {
        match event {
            Event::SelfDescribing(self_describing) => self.build_self_describing_event_payload(self_describing, payload_builder),
            Event::Structured(structured) => self.build_structured_event_payload(structured, payload_builder),
            Event::ScreenView(screen_view) => self.build_screen_view_payload(screen_view, payload_builder)
        }.build().unwrap()
    }

    fn build_self_describing_event_payload(&self, event: SelfDescribingEvent, payload_builder: PayloadBuilder) -> PayloadBuilder {
        payload_builder
            .e(EventType::SelfDescribingEvent)
            .ue_pr(SelfDescribingEventData::new(SelfDescribingJson { schema: event.schema, data: event.data }))
    }

    fn build_structured_event_payload(&self, struct_event: StructuredEvent, payload_builder: PayloadBuilder) -> PayloadBuilder {
        payload_builder
            .e(EventType::StructuredEvent)
            .se_ca(struct_event.category)
            .se_ac(struct_event.action)
            .se_pr(struct_event.property)
            .se_la(struct_event.label)
            .se_va(if let Some(value) = struct_event.value { Some(value.to_string()) } else { None })
    }

    fn build_screen_view_payload(&self, screen_view: ScreenViewEvent, payload_builder: PayloadBuilder) -> PayloadBuilder {
        let event = SelfDescribingEvent::builder()
            .schema("iglu:com.snowplowanalytics.mobile/screen_view/jsonschema/1-0-0")
            .data(json!(screen_view))
            .build()
            .unwrap();
        self.build_self_describing_event_payload(event, payload_builder)
    }
}
