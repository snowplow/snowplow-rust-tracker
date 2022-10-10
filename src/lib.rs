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

//! # Snowplow Rust Tracker
//!
//! The Snowplow Rust Tracker allows you to track Snowplow events in your Rust applications.
//!
//! ## Example usage
//!
//! use snowplow_tracker::{Snowplow, SelfDescribingJson, SelfDescribingEvent, Subject};
//! use serde_json::json;
//!
//! // Initialize a tracker instance given a namespace, application ID, Snowplow collector URL, and
//! // Subject
//!
//! let tracker_subject = match Subject::builder().language("en-gb").build() {
//!     Ok(subject) => subject,
//!     Err(e) => panic!("{e}"), // your error handling here
//! };
//!
//! let tracker = Snowplow::create_tracker("ns", "app_id", "https://...", Some(tracker_subject));
//!
//!
//! // Tracking a self-describing event with a context entity and subject
//! let event_subject = match Subject::builder().language("en-gb").build() {
//!     Ok(subject) => subject,
//!     Err(e) => panic!("{e}"), // your error handling here
//! };
//!
//! let self_describing_event_build = match SelfDescribingEvent::builder()
//!     .schema("iglu:com.snowplowanalytics.snowplow/link_click/jsonschema/1-0-1")
//!     .data(json!({"targetUrl": "http://a-target-url.com"}))
//!     .subject(event_subject)
//! {
//!     Ok(event) => event,
//!     Err(e) => panic!("{e}"), // your error handling here
//! };
//!
//! let context = Some(vec![SelfDescribingJson::new(
//!     "iglu:org.schema/WebPage/jsonschema/1-0-0",
//!     json!({"keywords": ["tester"]}),
//! ]));
//!
//! let self_desc_event_id = match tracker.track(
//!     self_describing_event,
//!     context,
//! ) {
//!     Ok(id) => id,
//!     Err(e) => panic!("{e}"), // your error handling here
//! }
//!

mod emitter;
mod error;
mod event;
mod payload;
mod snowplow;
mod subject;
mod tracker;

pub use emitter::Emitter;
pub use error::Error;
pub use event::ScreenViewEvent;
pub use event::SelfDescribingEvent;
pub use event::StructuredEvent;
pub use event::TimingEvent;
pub use payload::SelfDescribingJson;
pub use snowplow::Snowplow;
pub use subject::Subject;
pub use tracker::Tracker;
