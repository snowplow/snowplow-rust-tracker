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

use crate::payload::PayloadBuilder;
use crate::Error;

/// An Emitter is responsible for handling events in an [EventStore](crate::EventStore),
/// which are sent to the collector using a [HttpClient](crate::HttpClient).
///
/// Implement this trait to use your own Emitter implementation on a tracker.
pub trait Emitter: Send + Sync {
    /// Add a [PayloadBuilder] to the Emitter
    fn add(&mut self, payload: PayloadBuilder) -> Result<(), Error>;
    /// Try to send all events in the Emitter's queue
    fn flush(&mut self) -> Result<(), Error>;
    /// Safely shuts down the Emitter.
    fn close(&mut self) -> Result<(), Error>;
    /// The provided URL of the Snowplow collector
    fn collector_url(&self) -> &str;
}
