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
use crate::payload::{Payload, ContextData, SelfDescribingJson};
use crate::event::EventBuildable;
use uuid::Uuid;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
    pub async fn track(&self, event: impl EventBuildable, context: Option<Vec<SelfDescribingJson>>) -> Option<Uuid> {
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

        let payload = event.build_payload(payload_builder);
        if self.emitter.add(payload).await.is_ok() {
            Some(event_id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

        #[test]
        fn create_new_tracker() {
            let tracker = Tracker::new(
                "test namespace",
                "test app id",
                Emitter::new("http://example.com/")
            );

            assert_eq!(tracker.namespace, "test namespace");
            assert_eq!(tracker.app_id, "test app id");
            assert_eq!(tracker.emitter.collector_url, "http://example.com/");
            assert_eq!(tracker.config.platform, "pc".to_string());
            assert_eq!(tracker.config.version, "rust-0.1.0".to_string());
            assert_eq!(tracker.config.encode_base_64, false);
        }
}
