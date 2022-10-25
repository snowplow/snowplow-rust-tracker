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

use std::time::UNIX_EPOCH;
use std::time::{SystemTime, SystemTimeError};
use uuid::Uuid;

use crate::emitter::Emitter;
use crate::error::Error;
use crate::event::PayloadAddable;
use crate::payload::{ContextData, Payload, SelfDescribingJson};
use crate::subject::Subject;

pub struct TrackerConfig {
    pub platform: String,
    pub version: String,
    pub encode_base_64: bool,
}

/// The Snowplow tracker, used to track events
pub struct Tracker {
    /// Tracker namespace that identifies the tracker within the app
    namespace: String,
    /// Application ID
    app_id: String,
    /// Emitter used to send events to the Collector
    emitter: Box<dyn Emitter>,
    /// Additional tracker config
    config: TrackerConfig,
    /// The [Subject] that will be applied to all events
    /// An event-level subject will take priority over this
    subject: Subject,
}

impl Tracker {
    /// Creates a new Tracker instance
    pub fn new(
        namespace: &str,
        app_id: &str,
        emitter: impl Emitter + 'static,
        subject: Option<Subject>,
    ) -> Tracker {
        Tracker {
            namespace: namespace.to_string(),
            app_id: app_id.to_string(),
            emitter: Box::new(emitter),
            // By providing a default subject, we can avoid having to unwrap the subject
            //
            // The default for Subject provides `None` for all fields, so will be skipped
            // when serializing
            subject: subject.unwrap_or(Subject::default()),
            config: TrackerConfig {
                platform: "pc".to_string(),
                version: "rust-0.1.0".to_string(),
                encode_base_64: false,
            },
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn emitter(&self) -> &Box<dyn Emitter> {
        &self.emitter
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    /// Attempts to send all events in the event store to the collector
    pub fn flush(&mut self) -> Result<(), Error> {
        self.emitter.flush()
    }

    /// Safely shuts down the Emitter
    pub fn close_emitter(&mut self) -> Result<(), Error> {
        self.emitter.close()
    }

    /// Provides mutable access to the `subject` field
    ///
    /// ## Example
    /// ```
    /// use snowplow_tracker::{Snowplow, Subject};
    ///
    /// // Build a Subject that will be attached to this tracker
    /// let tracker_subject = match Subject::builder().user_id("user_1").language("en-gb").build() {
    ///     Ok(subject) => subject,
    ///     Err(e) => panic!("Subject could not be built: {e}"), // your error handling here
    /// };
    ///
    /// // Create a tracker with attached Subject
    /// let mut tracker = Snowplow::create_tracker("ns", "app_id", "https://...", Some(tracker_subject));
    ///
    /// assert_eq!(tracker.subject().user_id, Some("user_1".to_string()));
    /// assert_eq!(tracker.subject().language, Some("en-gb".to_string()));
    ///
    /// // Bulild a new Subject to replace the instance in `tracker`
    /// let new_tracker_subject = match Subject::builder().user_id("user_2").build() {
    ///     Ok(subject) => subject,
    ///     Err(e) => panic!("Subject could not be built: {e}"), // your error handling here
    /// };
    ///
    /// // We must dereference here to assign to the mutably borrowed value
    /// *tracker.subject_mut() = new_tracker_subject;
    ///
    /// // Close the tracker emitter
    /// match tracker.close_emitter() {
    ///     Ok(_) => (),
    ///     Err(e) => panic!("Emitter could not be closed: {e}"), // your error handling here
    /// };
    ///
    /// assert_eq!(tracker.subject().user_id, Some("user_2".to_string()));
    /// assert_eq!(tracker.subject().language, None);
    /// ```
    pub fn subject_mut(&mut self) -> &mut Subject {
        &mut self.subject
    }

    /// Tracks a Snowplow event with optional context entities and sends it to the Snowplow collector.
    pub fn track(
        &mut self,
        event: impl PayloadAddable,
        context: Option<Vec<SelfDescribingJson>>,
    ) -> Result<Uuid, Error> {
        let since_the_epoch =
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e: SystemTimeError| {
                    Error::BuilderError(format!("Failed to get current time: {}", e.to_string()))
                })?;

        let event_id = Uuid::new_v4();

        let mut payload_builder = Payload::builder()
            .p(self.config.platform.clone())
            .tv(self.config.version.clone())
            .eid(event_id.clone())
            .dtm(since_the_epoch.as_millis().to_string())
            .aid(self.app_id.clone());

        if let Some(context) = context {
            payload_builder = payload_builder.co(ContextData::new(context));
        }

        // Event Subject gets priority over Tracker Subject
        if let Some(event_subject) = event.subject() {
            payload_builder =
                payload_builder.subject(event_subject.clone().merge(self.subject.clone()));
        }

        payload_builder = event.add_to_payload(payload_builder);

        let event_id = match payload_builder.eid {
            Some(eid) => eid,
            None => return Err(Error::BuilderError("Event ID not set".to_string())),
        };

        self.emitter.add(payload_builder)?;
        Ok(event_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::BatchEmitter;

    use super::*;

    #[test]
    fn create_new_tracker() {
        let mut tracker = Tracker::new(
            "test namespace",
            "test app id",
            BatchEmitter::builder()
                .collector_url("http://example.com/")
                .build()
                .unwrap(),
            Some(Subject {
                user_id: Some("user_1".to_string()),
                ..Subject::default()
            }),
        );

        assert_eq!(tracker.namespace, "test namespace");
        assert_eq!(tracker.app_id, "test app id");
        assert_eq!(tracker.emitter.collector_url(), "http://example.com/");
        assert_eq!(tracker.subject.user_id, Some("user_1".to_string()));
        assert_eq!(tracker.config.platform, "pc".to_string());
        assert_eq!(tracker.config.version, "rust-0.1.0".to_string());
        assert_eq!(tracker.config.encode_base_64, false);

        tracker.close_emitter().unwrap();
    }

    #[test]
    fn replace_tracker_subject() {
        let mut tracker = Tracker::new(
            "test namespace",
            "test app id",
            BatchEmitter::builder()
                .collector_url("http://example.com/")
                .build()
                .unwrap(),
            Some(Subject::builder().user_id("user_1").build().unwrap()),
        );
        assert_eq!(tracker.subject.user_id, Some("user_1".to_string()));

        *tracker.subject_mut() = Subject::builder().user_id("user_2").build().unwrap();

        assert_eq!(tracker.subject.user_id, Some("user_2".to_string()));

        tracker.close_emitter().unwrap();
    }

    #[test]
    fn update_tracker_subject() {
        let mut tracker = Tracker::new(
            "test namespace",
            "test app id",
            BatchEmitter::builder()
                .collector_url("http://example.com/")
                .build()
                .unwrap(),
            Some(
                Subject::builder()
                    .user_id("user_1")
                    .ip_address("999.999.999.999")
                    .build()
                    .unwrap(),
            ),
        );
        assert_eq!(tracker.subject.user_id, Some("user_1".to_string()));
        assert_eq!(
            tracker.subject.ip_address,
            Some("999.999.999.999".to_string())
        );

        let updated_subject = Subject::builder().user_id("user_2").build().unwrap();

        *tracker.subject_mut() = updated_subject.merge(tracker.subject.clone());

        assert_eq!(tracker.subject.user_id, Some("user_2".to_string()));
        assert_eq!(
            tracker.subject.ip_address,
            Some("999.999.999.999".to_string())
        );

        tracker.close_emitter().unwrap();
    }
}
