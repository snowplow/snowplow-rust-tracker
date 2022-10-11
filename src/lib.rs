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
//! ```
//! use snowplow_tracker::{Snowplow, SelfDescribingJson, SelfDescribingEvent, Subject};
//! use serde_json::json;
//!
//! // Initialize a tracker instance given a namespace, application ID, Snowplow collector URL, and
//! // Subject
//!
//! let subject = Subject::builder().language("en-gb").build().unwrap();
//! let tracker = Snowplow::create_tracker("ns", "app_id", "https://...", Some(subject));
//!
//! // Tracking a self-describing event with a context entity
//! tracker.track(
//!     SelfDescribingEvent::builder()
//!         .schema("iglu:com.snowplowanalytics.snowplow/link_click/jsonschema/1-0-1")
//!         .data(json!({"targetUrl": "http://a-target-url.com"}))
//!         .subject(
//!             Subject::builder()
//!             .user_id("user_1")
//!             .build()
//!             .unwrap()
//!         )
//!         .build()
//!         .unwrap(),
//!     Some(vec![
//!         SelfDescribingJson::new("iglu:org.schema/WebPage/jsonschema/1-0-0", json!({"keywords": ["tester"]}))
//!     ]),
//! );
//! ```

mod emitter;
mod event;
mod payload;
mod snowplow;
mod subject;
mod tracker;

pub use emitter::Emitter;
pub use event::ScreenViewEvent;
pub use event::SelfDescribingEvent;
pub use event::StructuredEvent;
pub use event::TimingEvent;
pub use payload::SelfDescribingJson;
pub use snowplow::Snowplow;
pub use subject::Subject;
pub use tracker::Tracker;
