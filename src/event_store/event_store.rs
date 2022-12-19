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

use uuid::Uuid;

use crate::error::Error;
use crate::event_batch::EventBatch;
use crate::payload::PayloadBuilder;

/// An EventStore is responsible for storing events until they are sent to the collector.
///
/// Implement this trait to use your own EventStore implementation on an [Emitter](crate::Emitter).
pub trait EventStore {
    /// Add a [PayloadBuilder] to the EventStore
    fn add(&mut self, payload: PayloadBuilder) -> Result<(), Error>;
    /// The number of events currently in the EventStore
    fn len(&self) -> usize;
    /// The set size of the batches that will be sent to the collector
    fn batch_size(&self) -> usize;
    /// The maximum number of events that can be stored in the EventStore
    fn capacity(&self) -> usize;
    /// Removes and returns a batch of events from the event store
    /// The batch size is determined by the `batch_size` field
    fn full_batch(&mut self) -> Result<EventBatch, Error>;
    /// Removes and returns the provided number of events from the EventStore as an [EventBatch]
    fn batch_of(&mut self, size: usize) -> Result<EventBatch, Error>;
    // A method to be called after attempts to send are finished, either successfully or unsuccessfully
    fn cleanup_after_send_attempt(&mut self, batch_id: Uuid) -> Result<(), Error>;
}
