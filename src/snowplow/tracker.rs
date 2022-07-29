
use crate::snowplow::Emitter;
use crate::snowplow::payload::{Payload, PayloadBuilder};
use crate::snowplow::payload::EventType;
use uuid::Uuid;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::snowplow::event::SelfDescribingJson;
use crate::snowplow::event::StructuredEvent;
use crate::snowplow::event::ScreenViewEvent;
use serde_json::json;

pub struct TrackerConfig {
    pub platform: String,
    pub version: String,
    pub encode_base_64: bool,
}

pub struct Tracker {
    pub namespace: String,
    pub app_id: String,
    pub emitter: Emitter,
    pub config: TrackerConfig,
}

#[derive(Debug)]
pub enum TrackingError {
    EmitterError(reqwest::Error),
    PayloadError(serde_json::Error),
}

impl Tracker {
    pub fn new(namespace: String, app_id: String, emitter: Emitter) -> Tracker {
        Tracker {
            namespace,
            app_id,
            emitter,
            config: TrackerConfig {
                platform: "pc".to_string(),
                version: "rust-0.1.0".to_string(),
                encode_base_64: false,
            }
        }
    }

    pub async fn track(&self, pb: PayloadBuilder) -> Result<String, reqwest::Error> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let payload = pb
            .p(self.config.platform.clone())
            .tv(self.config.version.clone())
            .eid(Uuid::new_v4())
            .dtm(since_the_epoch.as_millis().to_string())
            .stm(since_the_epoch.as_millis().to_string())
            .build()
            .unwrap();

        self.emitter.post(payload, &self.emitter.collector_url).await
    }

    pub async fn track_self_describing_event(&self, schema: String, data: String) -> Result<String, TrackingError> {
        let evnt = SelfDescribingJson::from_schema_and_data(schema, data).unwrap();

        let payload_builder = Payload::builder()
            .e(EventType::SelfDescribingEvent)
            .ue_pr(evnt)
            .aid(self.app_id.clone());

        match self.track(payload_builder).await {
            Ok(res) => Ok(res),
            Err(err) => Err(TrackingError::EmitterError(err)),
        }
    }

    pub async fn track_struct_event(&self, struct_event: StructuredEvent ) -> Result<String, TrackingError> {
        let payload_builder = Payload::builder()
            .e(EventType::StructuredEvent)
            .aid(self.app_id.clone())
            .se_ca(struct_event.category)
            .se_ac(struct_event.action)
            .se_pr(struct_event.property)
            .se_la(struct_event.label)
            .se_va(struct_event.value);

        match self.track(payload_builder).await {
            Ok(res) => Ok(res),
            Err(err) => Err(TrackingError::EmitterError(err)),
        }
    }

    pub async fn track_screen_view(&self, screen_view: ScreenViewEvent) -> Result<String, TrackingError> {
        match self.track_self_describing_event(
            "iglu:com.snowplowanalytics.snowplow/screen_view/jsonschema/1-0-0".to_string(),
            json!(screen_view).to_string(),
        ).await {
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        }
    }
}
