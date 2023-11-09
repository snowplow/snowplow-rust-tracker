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

use crate::event_batch::EventBatch;
use crate::event_store::EventStore;
use crate::payload::{Payload, PayloadBuilder};
use crate::Error;

// This is pub(crate) as it is used in BatchEmitter
pub(crate) const DEFAULT_EVENT_STORE_CAPACITY: usize = 10_000;
const DEFAULT_BATCH_SIZE: usize = 50;

struct InMemoryEventStoreQueue {
    queue: Vec<PayloadBuilder>,
    capacity: usize,
}

// A slightly extended Vec to store maximum capacity,
// along with returning an error on add if the maximum capacity is reached.
impl InMemoryEventStoreQueue {
    fn new(capacity: usize) -> Self {
        InMemoryEventStoreQueue {
            // `with_capacity` allocates `capacity` elements, to avoid later reallocation
            queue: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add a payload to the queue
    /// Returns an error if the queue is full
    fn push(&mut self, payload: PayloadBuilder) -> Result<(), Error> {
        if self.queue.len() == self.queue.capacity() {
            return Err(Error::EventStoreError("Event store is full".to_string()));
        }
        self.queue.push(payload);
        Ok(())
    }
}

/// An implementation of the [EventStore] trait, that queues events in a Vec
pub struct InMemoryEventStore {
    event_queue: InMemoryEventStoreQueue,
    batch_size: usize,
}

/// Provides an instance of [InMemoryEventStore], with the default batch size of 50, and a queue capacity of 10,000
impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self {
            event_queue: InMemoryEventStoreQueue::new(DEFAULT_EVENT_STORE_CAPACITY),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
}

impl InMemoryEventStore {
    pub fn new(queue_capacity: usize, batch_size: usize) -> Self {
        Self {
            event_queue: InMemoryEventStoreQueue::new(queue_capacity),
            batch_size,
        }
    }

    fn event_batch(&mut self, size: usize) -> Result<EventBatch, Error> {
        if self.event_queue.queue.is_empty() {
            return Err(Error::EventStoreError("Event store is empty".to_string()));
        }

        if size > self.batch_size {
            return Err(Error::EventStoreError(
                "Not enough events to create batch".to_string(),
            ));
        }

        // Move `size` events from the event queue and set `stm` for each
        let events_to_send: Vec<Payload> = self
            .event_queue
            .queue
            .drain(0..size)
            .map(|e| e.finalise_payload())
            .collect::<Result<Vec<Payload>, Error>>()?;

        // Take the first event's `eid` and use it for the batch id
        let first_event_id = match events_to_send.first() {
            Some(payload) => payload.eid.clone(),
            None => return Err(Error::EventStoreError("No events to send".to_string())),
        };

        Ok(EventBatch::new(first_event_id, events_to_send))
    }
}

impl EventStore for InMemoryEventStore {
    fn add(&mut self, event: PayloadBuilder) -> Result<(), Error> {
        self.event_queue.push(event)
    }

    fn len(&self) -> usize {
        self.event_queue.queue.len()
    }

    fn capacity(&self) -> usize {
        self.event_queue.capacity
    }

    fn full_batch(&mut self) -> Result<EventBatch, Error> {
        if self.event_queue.queue.len() < self.batch_size {
            return Err(Error::EventStoreError(
                "Failed to get batch: Not enough events in the event store for a full batch"
                    .to_string(),
            ));
        }
        self.event_batch(self.batch_size)
    }

    fn batch_of(&mut self, size: usize) -> Result<EventBatch, Error> {
        if size > self.event_queue.queue.len() {
            return Err(Error::EventStoreError(
                "Requested batch size is greater than queue length".to_string(),
            ));
        }
        self.event_batch(size)
    }

    fn batch_size(&self) -> usize {
        self.batch_size
    }

    // InMemoryEventStore doesn't need to do anything to clean up after a send attempt
    fn cleanup_after_send_attempt(&mut self, batch_id: Uuid) -> Result<(), Error> {
        Ok(drop(batch_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_payloads(n: usize) -> Vec<PayloadBuilder> {
        (0..n)
            .map(|_| {
                Payload::builder()
                    .p("p".to_string())
                    .tv("tv".to_string())
                    .eid(uuid::Uuid::new_v4())
                    .dtm("dtm".to_string())
                    .stm("stm".to_string())
                    .ttm("ttm".to_string())
                    .aid("aid".to_string())
            })
            .collect()
    }

    #[test]
    fn adds_event_to_store() {
        let mut event_store = InMemoryEventStore::default();
        let mut payloads = create_payloads(1);
        let payload = payloads.drain(..1).next().unwrap();
        let expected_eid = payload.eid.clone();

        event_store.add(payload).unwrap();

        assert_eq!(event_store.len(), 1);
        assert_eq!(
            event_store
                .event_queue
                .queue
                .drain(0..1)
                .collect::<Vec<_>>()
                .first()
                .unwrap()
                .eid,
            expected_eid
        );
    }

    #[test]
    fn store_length() {
        let mut event_store = InMemoryEventStore::new(4, 2);
        let payloads = create_payloads(4);

        for payload in payloads {
            event_store.add(payload).unwrap();
        }

        assert_eq!(event_store.len(), 4);
    }

    #[test]
    fn get_batch() {
        let mut event_store = InMemoryEventStore::new(4, 2);
        let payloads = create_payloads(4);

        for payload in payloads {
            event_store.add(payload).unwrap();
        }

        assert_eq!(event_store.len(), 4);
        assert_eq!(event_store.full_batch().unwrap().events.len(), 2);
        assert_eq!(event_store.len(), 2);
    }

    #[test]
    fn get_batch_without_enough_events_in_queue() {
        let mut event_store = InMemoryEventStore::new(2, 2);
        let payloads = create_payloads(1);

        for payload in payloads {
            event_store.add(payload).unwrap();
        }

        assert_eq!(event_store.len(), 1);
        assert!(event_store.full_batch().is_err());
    }
}
