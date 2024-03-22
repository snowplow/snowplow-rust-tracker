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

//! # Snowplow Tracker
//!
//! The [Snowplow](https://snowplow.io) Rust Tracker allows you to track Snowplow events in your Rust applications.
//! For information on how to effectively design tracking using Snowplow, visit our [guide on tracking design.](https://docs.snowplow.io/docs/understanding-tracking-design/)
//!
//! ## Example usage
//!
//! This simple example shows the process of creating a Tracker, and then building and tracking a [Self-Describing Event](crate::SelfDescribingEvent), using the [`link_click`](https://github.com/snowplow/iglu-central/blob/master/schemas/com.snowplowanalytics.snowplow/link_click/jsonschema/1-0-1)
//! Iglu schema URI, and schema-confirming JSON data.
//!
//! ```
//! use snowplow_tracker::{SelfDescribingEvent, Snowplow, Subject};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Build a Subject that will be attached to all events sent by this tracker
//!     let tracker_subject = match Subject::builder().language("en-gb").build() {
//!         Ok(subject) => subject,
//!         Err(e) => panic!("Subject could not be built: {e}"), // your error handling here
//!     };
//!
//!     // Create a tracker
//!     let mut tracker = Snowplow::create_tracker("ns", "app_id", "https://example.com", Some(tracker_subject));
//!
//!     // Build a Self-Describing Event, with the schema of the event we want to track, along
//!     // with relevent, schema-conforming, data
//!     let self_describing_event = match SelfDescribingEvent::builder()
//!         .schema("iglu:com.snowplowanalytics.snowplow/link_click/jsonschema/1-0-1")
//!         .data(json!({"targetUrl": "http://example.com/some-page"}))
//!         .ttm("1701147392697")
//!         .build()
//!     {
//!         Ok(event) => event,
//!         Err(e) => panic!("SelfDescribingEvent could not be built: {e}"), // your error handling here
//!     };
//!
//!     // Track our Self-Describing Event
//!     let self_desc_event_uuid = match tracker.track(self_describing_event, None) {
//!         Ok(uuid) => uuid,
//!         Err(e) => panic!("Failed to emit event: {e}"), // your error handling here
//!     };
//!
//!      // Close the tracker emitter thread
//!      match tracker.close_emitter() {
//!          Ok(_) => (),
//!          Err(e) => panic!("Emitter could not be closed: {e}"), // your error handling here
//!      };
//! }
//! ```

mod emitter;
mod error;
mod event;
mod event_batch;
mod event_store;
mod http_client;
mod payload;
mod snowplow;
mod subject;
mod tracker;

pub use emitter::{BatchEmitter, Emitter, RetryPolicy};
pub use error::Error;
pub use event::{
    PayloadAddable, ScreenViewEvent, SelfDescribingEvent, SelfDescribingEventBuilder,
    StructuredEvent, TimingEvent,
};
pub use event_store::{EventStore, InMemoryEventStore};
pub use http_client::{HttpClient, ReqwestClient};
pub use payload::{Payload, PayloadBuilder, SelfDescribingJson};
pub use snowplow::Snowplow;
pub use subject::Subject;
pub use tracker::Tracker;
