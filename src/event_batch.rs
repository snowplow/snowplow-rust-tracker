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

use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use rand::Rng;
use serde_json::json;
use uuid::Uuid;

use crate::{emitter::RetryPolicy, payload::Payload, Error, SelfDescribingJson};

const PAYLOAD_DATA_SCHEMA: &str =
    "iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4";

/// A batch of events to be sent to the collector.
#[derive(Debug)]
pub struct EventBatch {
    pub id: Uuid,
    pub events: Vec<Payload>,
    pub delay: Option<Duration>,
    pub retry_attempts: u32,
}

impl EventBatch {
    pub fn new(id: Uuid, events: Vec<Payload>) -> Self {
        Self {
            id,
            events,
            delay: None,
            retry_attempts: 0,
        }
    }

    /// Creates a sendable payload from the batch.
    pub fn as_payload(&self) -> SelfDescribingJson {
        SelfDescribingJson {
            schema: PAYLOAD_DATA_SCHEMA.to_string(),
            data: json!(self.events),
        }
    }

    /// Whether the batch has any retries remaining.
    pub fn has_retry(&self, retry_policy: RetryPolicy) -> bool {
        match retry_policy {
            RetryPolicy::NoRetry => false,
            RetryPolicy::MaxRetries(n) => self.retry_attempts < n,
            RetryPolicy::RetryForever => true,
        }
    }

    /// Updates the events `stm` field in batch with the current time.
    pub fn update_event_stm(&mut self) -> Result<(), Error> {
        let since_the_epoch =
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e: SystemTimeError| {
                    Error::BuilderError(format!("Failed to get current time: {}", e.to_string()))
                })?;

        for event in self.events.iter_mut() {
            event.stm = since_the_epoch.as_millis().to_string();
        }

        Ok(())
    }

    /// Updates the delay until another sending attempt is made.
    pub fn update_for_retry(&mut self) {
        let max_event_delay_time = Duration::from_secs(600_000);

        self.retry_attempts += 1;

        self.delay = match self.delay {
            Some(delay) => {
                // 2 +- random number between 0 and 1
                let delay_mul = rand::thread_rng().gen_range(1.0..=3.0);

                Some(delay.mul_f32(delay_mul).min(max_event_delay_time))
            }
            None => Some(Duration::from_secs(1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uuid::Uuid;

    use crate::emitter::RetryPolicy;
    use crate::PayloadBuilder;
    use crate::{event_batch::EventBatch, payload::Payload};

    fn create_payloads(n: usize) -> Vec<PayloadBuilder> {
        (0..n)
            .map(|_| {
                Payload::builder()
                    .p("p".to_string())
                    .tv("tv".to_string())
                    .eid(Uuid::new_v4())
                    .dtm("dtm".to_string())
                    .stm("stm".to_string())
                    .aid("aid".to_string())
            })
            .collect()
    }

    #[test]
    fn update_event_stm() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        let mut batch = EventBatch::new(
            Uuid::new_v4(),
            create_payloads(5)
                .drain(..)
                .map(|p| p.finalise_payload().unwrap())
                .collect(),
        );

        std::thread::sleep(Duration::from_secs(1));

        batch.update_event_stm().unwrap();

        for event in batch.events.iter() {
            let event_stm = Duration::from_millis(event.stm.parse::<u64>().unwrap());
            assert!(event_stm > now);
        }
    }

    #[test]
    fn update_batch_delay() {
        let mut batch = EventBatch::new(
            Uuid::new_v4(),
            create_payloads(5)
                .drain(..)
                .map(|p| p.finalise_payload().unwrap())
                .collect(),
        );

        std::thread::sleep(Duration::from_secs(1));

        batch.update_for_retry();

        assert!(batch.delay.unwrap() > Duration::from_secs(0));
    }

    #[test]
    fn no_retry_policy() {
        let batch = EventBatch::new(
            Uuid::new_v4(),
            create_payloads(5)
                .drain(..)
                .map(|p| p.finalise_payload().unwrap())
                .collect(),
        );

        assert!(!batch.has_retry(RetryPolicy::NoRetry));
    }

    #[test]
    fn limited_retry_policy() {
        let mut batch = EventBatch::new(
            Uuid::new_v4(),
            create_payloads(5)
                .drain(..)
                .map(|p| p.finalise_payload().unwrap())
                .collect(),
        );
        let policy = RetryPolicy::MaxRetries(5);

        assert!(batch.has_retry(policy));

        for _ in 0..5 {
            batch.update_for_retry();
        }

        assert!(!batch.has_retry(policy));
    }
}
