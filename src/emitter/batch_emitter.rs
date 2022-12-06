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

use std::sync::{Arc, Mutex};

use crate::emitter::Emitter;
use crate::error::Error;
use crate::event_batch::EventBatch;
use crate::event_store::DEFAULT_EVENT_STORE_CAPACITY;
use crate::event_store::{EventStore, InMemoryEventStore};
use crate::http_client::ReqwestClient;
use crate::payload::PayloadBuilder;
use crate::HttpClient;

/// An implementation of the [Emitter] trait that sends batched events to the Snowplow Collector.
pub struct BatchEmitter {
    /// The URL of your Snowplow [Collector](https://docs.snowplow.io/docs/pipeline-components-and-applications/stream-collector/)
    collector_url: String,
    /// A [HttpClient](crate::HttpClient) implementation to send events to the Snowplow Collector
    http_client: Box<dyn HttpClient + Send + Sync>,
    /// An [EventStore](crate::EventStore) implementation, used to queue events
    event_store: Arc<Mutex<dyn EventStore + Send + Sync>>,
    /// The thread running the tokio runtime
    executor_handle: Option<std::thread::JoinHandle<()>>,
    /// The transmitter to send an [EmitterMessage] to the [Emitter] thread
    tx: tokio::sync::mpsc::Sender<EmitterMessage>,
}

/// Possible messages to send to the Emitter, sent via the [Emitter] transmitter
#[derive(Debug)]
pub enum EmitterMessage {
    /// Sends a batch of events
    Send(EventBatch),
    /// Shuts down the [Emitter]
    /// This will also attempt to send all events currently in the [EventStore]
    Close,
}

/// A builder for the [BatchEmitter] struct
pub struct BatchEmitterBuilder {
    collector_url: Option<String>,
    event_store: Arc<Mutex<dyn EventStore + Send + Sync>>,
}

impl BatchEmitterBuilder {
    pub fn default() -> Self {
        Self {
            collector_url: None,
            event_store: Arc::new(Mutex::new(InMemoryEventStore::default())),
        }
    }

    /// Set the URL of your Snowplow [Collector](https://docs.snowplow.io/docs/pipeline-components-and-applications/stream-collector/)
    pub fn collector_url(mut self, collector_url: &str) -> Self {
        self.collector_url = Some(collector_url.to_string());
        self
    }

    /// Set the [EventStore] implementation  
    pub fn event_store(mut self, event_store: impl EventStore + Send + Sync + 'static) -> Self {
        self.event_store = Arc::new(Mutex::new(event_store));
        self
    }

    /// Build the [BatchEmitter]
    pub fn build(self) -> Result<BatchEmitter, Error> {
        match self.collector_url {
            Some(collector_url) => {
                let event_store_capacity = match self.event_store.lock() {
                    Ok(event_store) => event_store.capacity(),
                    Err(e) => {
                        return Err(Error::EventStoreError(
                            format!("Failed to lock event store: {}", e).to_string(),
                        ))
                    }
                };

                Ok(BatchEmitter::create_emitter(
                    &collector_url,
                    event_store_capacity,
                    self.event_store,
                ))
            }
            None => Err(Error::EmitterError("Collector URL is required".to_string())),
        }
    }
}

impl BatchEmitter {
    pub fn builder() -> BatchEmitterBuilder {
        BatchEmitterBuilder::default()
    }

    fn create_emitter(
        collector_url: &str,
        event_store_capacity: usize,
        event_store: Arc<Mutex<dyn EventStore + Send + Sync>>,
    ) -> BatchEmitter {
        let (tx, mut rx) = tokio::sync::mpsc::channel(event_store_capacity);
        let mut emitter = BatchEmitter {
            collector_url: collector_url.to_string(),
            http_client: ReqwestClient::new(&collector_url),
            event_store,
            executor_handle: None,
            tx: tx.clone(),
        };

        // Clone http client to be used in the spawned thread
        let client = emitter.http_client.clone();

        // Spawn the tokio runtime in a separate thread
        emitter.executor_handle = Some(std::thread::spawn(move || {
            BatchEmitter::start_tokio(client, &mut rx)
        }));

        emitter
    }

    /// Create a new [BatchEmitter] with an [InMemoryEventStore]
    pub fn new(collector_url: &str) -> BatchEmitter {
        BatchEmitter::create_emitter(
            collector_url,
            DEFAULT_EVENT_STORE_CAPACITY,
            Arc::new(Mutex::new(InMemoryEventStore::default())),
        )
    }

    // Static Methods

    // Sends an EventBatch to the collector
    async fn send_batch(
        batch: EventBatch,
        http_client: Box<dyn HttpClient + Send + Sync>,
    ) -> Result<(), Error> {
        match http_client.post(batch.as_payload()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::EmitterError(format!("{e}"))),
        }
    }

    // Starts a tokio runtime and runs the emitter loop
    fn start_tokio(
        http_client: Box<dyn HttpClient + Send + Sync>,
        rx: &mut tokio::sync::mpsc::Receiver<EmitterMessage>,
    ) {
        // Create a new runtime to handle the async tasks
        // Unwrap here as if the runtime fails to start, there is nothing we can do
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // The main emitter loop
        // This continuously loops and checks for new batches to send
        rt.block_on(async {
            // The currently running tokio tasks
            let mut tokio_tasks: Vec<_> = Vec::new();

            loop {
                // `rx.recv().await` will not resolve until either a message is recieved,
                // or the channel is closed and there are no more messages, in which case we exit the loop
                let message = match rx.recv().await {
                    Some(msg) => {
                        log::debug!("Received message: {:?}", msg);
                        msg
                    }
                    None => break,
                };

                match message {
                    EmitterMessage::Send(batch) => {
                        // Clone HttpClient to be moved into the task
                        let client = http_client.clone();

                        // Spawn a new task to send the batch
                        tokio_tasks.push(tokio::spawn(async move {
                            let batch_length = batch.events.len();
                            match Self::send_batch(batch, client).await {
                                Ok(_) => {
                                    log::info!("Sent batch of {batch_length} events")
                                }
                                Err(e) => log::warn!("Failed to send batch: {e}"),
                            }
                        }));
                    }

                    // On break, the emitter and runtime will be dropped
                    //
                    // Tokio will cancel any running tasks once the runtime is dropped, meaning any queued batches will be lost,
                    // so we attempt to send any remaining batches before exiting
                    EmitterMessage::Close => {
                        let remaining = tokio_tasks.len();
                        for (i, task) in tokio_tasks.iter_mut().enumerate() {
                            log::debug!("Waiting for task {i}/{remaining} to complete");
                            task.await.unwrap();
                        }
                        break;
                    }
                }

                // Discard any completed tasks in the task list
                tokio_tasks.retain(|t| !t.is_finished());
            }
        });
    }
}

impl Drop for BatchEmitter {
    fn drop(&mut self) {
        // Get the join handle for the thread running the tokio runtime and wait for it to finish
        //
        // It's likely that the thread has already finished once the emitter loop has exited
        if let Some(handle) = self.executor_handle.take() {
            handle.join().unwrap();
            log::debug!("BatchEmitter thread joined");
        }
        log::debug!("BatchEmitter dropped");
    }
}

impl Emitter for BatchEmitter {
    /// Adds a payload to the event store
    ///
    /// This may also trigger sending a payload to the collector if the event store has enough events to fill a batch
    fn add(&mut self, payload: PayloadBuilder) -> Result<(), Error> {
        let batch = match self.event_store.lock() {
            Ok(mut store) => {
                match store.add(payload) {
                    Ok(_) => log::debug!("Added event to event store"),
                    Err(e) => {
                        log::error!("Failed to add event to event store: {e}");
                        return Err(e);
                    }
                }
                // If the event store has enough events to fill a batch, return the batch
                store.full_batch()
            }
            Err(e) => return Err(Error::EmitterError(e.to_string())),
        };

        // We can ignore the error here, as the only error that can return is the event store being empty,
        // in which case we don't want to send a batch
        if let Ok(batch) = batch {
            return match self.tx.try_send(EmitterMessage::Send(batch)) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::EmitterError(e.to_string())),
            };
        }

        Ok(())
    }

    /// Attempt to send all events currently in the event store
    fn flush(&mut self) -> Result<(), Error> {
        // Get a batch of all events currently in the event store
        let batch = match self.event_store.lock() {
            Ok(mut store) => {
                let store_length = store.len();
                store.batch_of(store_length)
            }
            Err(e) => return Err(Error::EmitterError(e.to_string())),
        }?;

        match self.tx.try_send(EmitterMessage::Send(batch)) {
            Ok(_) => {
                log::debug!("Flushing event store");
                Ok(())
            }
            Err(e) => Err(Error::EmitterError(e.to_string())),
        }
    }

    /// Shut down and drop the emitter
    ///
    /// This will cancel any running tasks and may result in events being lost
    fn close(&mut self) -> Result<(), Error> {
        match self.tx.try_send(EmitterMessage::Close) {
            Ok(_) => {
                log::debug!("Closing emitter");
                Ok(())
            }
            Err(e) => Err(Error::EmitterError(e.to_string())),
        }
    }

    fn collector_url(&self) -> &str {
        &self.collector_url
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn add_event_to_store() {
        let mut emitter = BatchEmitter::new("http://localhost:8080");
        let payload = PayloadBuilder::default();

        emitter.add(payload).unwrap();
        assert_eq!(emitter.event_store.lock().unwrap().len(), 1);

        emitter.close().unwrap();
    }

    #[tokio::test]
    async fn send_batch() {
        let event_store = InMemoryEventStore::new(2, 2);
        let mut emitter = BatchEmitter::builder()
            .collector_url("http://localhost:8080")
            .event_store(event_store)
            .build()
            .unwrap();

        emitter.add(PayloadBuilder::default()).unwrap();
        assert_eq!(emitter.event_store.lock().unwrap().len(), 1);

        // Adding a second event should trigger a batch to be sent
        emitter.add(PayloadBuilder::default()).unwrap();
        assert_eq!(emitter.event_store.lock().unwrap().len(), 0);

        emitter.close().unwrap();
    }
}
