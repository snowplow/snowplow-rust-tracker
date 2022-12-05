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

use super::RetryPolicy;

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
    http_client: Option<Box<dyn HttpClient + Send + Sync>>,
    retry_policy: RetryPolicy,
}

impl BatchEmitterBuilder {
    pub fn default() -> Self {
        Self {
            collector_url: None,
            event_store: Arc::new(Mutex::new(InMemoryEventStore::default())),
            http_client: None,
            retry_policy: RetryPolicy::MaxRetries(10),
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

    /// Set the [HttpClient] implementation
    pub fn http_client(mut self, http_client: impl HttpClient + Send + Sync + 'static) -> Self {
        self.http_client = Some(Box::new(http_client));
        self
    }

    /// Set the retry policy
    pub fn retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
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
                    self.http_client
                        .unwrap_or(ReqwestClient::new(&collector_url)),
                    self.retry_policy,
                ))
            }
            None => Err(Error::EmitterError("Collector URL is required".to_string())),
        }
    }
}

// HTTP status codes that should not be retried
const DONT_RETRY_STATUS_CODES: [u16; 5] = [400, 401, 403, 410, 422];

/// The batch sent to the Snowplow Collector and the response code
pub struct SentBatchResponse {
    pub batch: EventBatch,
    pub code: u16,
}

impl BatchEmitter {
    pub fn builder() -> BatchEmitterBuilder {
        BatchEmitterBuilder::default()
    }

    fn create_emitter(
        collector_url: &str,
        event_store_capacity: usize,
        event_store: Arc<Mutex<dyn EventStore + Send + Sync>>,
        http_client: Box<dyn HttpClient + Send + Sync>,
        retry_policy: RetryPolicy,
    ) -> BatchEmitter {
        let (tx, rx) = tokio::sync::mpsc::channel(event_store_capacity);
        let mut emitter = BatchEmitter {
            collector_url: collector_url.to_string(),
            http_client,
            event_store,
            executor_handle: None,
            tx,
        };

        // Clone http client to be used in the spawned thread
        let client = emitter.http_client.clone();
        let store = emitter.event_store.clone();

        // Spawn the tokio runtime in a separate thread
        emitter.executor_handle = Some(std::thread::spawn(move || {
            BatchEmitter::start_tokio(client, rx, store, retry_policy);
        }));

        emitter
    }

    /// Create a new [BatchEmitter] with an [InMemoryEventStore]
    pub fn new(collector_url: &str) -> BatchEmitter {
        BatchEmitter::create_emitter(
            collector_url,
            DEFAULT_EVENT_STORE_CAPACITY,
            Arc::new(Mutex::new(InMemoryEventStore::default())),
            ReqwestClient::new(collector_url),
            RetryPolicy::MaxRetries(10),
        )
    }

    // Static Methods

    fn is_successful_response(code: u16) -> bool {
        code >= 200 && code < 300
    }

    // True if the code is outside 200-299 and not in DONT_RETRY_STATUS_CODES
    fn should_retry(code: u16) -> bool {
        match Self::is_successful_response(code) {
            true => false,
            false => !DONT_RETRY_STATUS_CODES.contains(&code),
        }
    }

    fn retry_batch(
        mut batch: EventBatch,
        retry_tx: tokio::sync::mpsc::UnboundedSender<EmitterMessage>,
    ) {
        batch.update_for_retry();

        let batch_id = batch.id;
        match retry_tx.send(EmitterMessage::Send(batch)) {
            Ok(_) => log::debug!("Batch {batch_id} re-queued"),
            Err(e) => {
                log::warn!("Failed to re-queue batch {batch_id}: {e}")
            }
        }
    }

    fn run_cleanup(
        store: Arc<Mutex<dyn EventStore + Send + Sync>>,
        batch: EventBatch,
    ) -> Result<(), Error> {
        let mut store_guard = match store.lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(Error::EmitterError(format!(
                    "Failed to acquire event store lock: {e}"
                )))
            }
        };

        match store_guard.cleanup_after_send_attempt(batch.id) {
            Ok(_) => log::debug!("Cleanup run for batch: {}", batch.id),
            Err(e) => return Err(Error::EmitterError(format!("Failed to cleanup: {e}"))),
        };

        Ok(())
    }

    async fn batch_send_task(
        mut batch: EventBatch,
        client: Box<dyn HttpClient + Send + Sync>,
        retry_tx: tokio::sync::mpsc::UnboundedSender<EmitterMessage>,
        store: Arc<Mutex<dyn EventStore + Send + Sync>>,
        retry_policy: RetryPolicy,
    ) {
        if let Some(delay) = batch.delay {
            log::debug!("Delaying batch {} for {:?}", batch.id, delay);
            tokio::time::sleep(delay).await;

            if let Err(e) = batch.update_event_stm() {
                // If the update fails, we just re-send the batch as-is
                // Not ideal, but it's better than losing events
                log::warn!(
                    "Failed to update stm of events in batch {} for retry: {e}",
                    batch.id
                )
            };
        };

        let batch_length = batch.events.len();
        match Self::send_batch(batch, client).await {
            Ok(resp) => {
                // We got a response from the collector, but need to check if
                // it was successful

                match (
                    Self::should_retry(resp.code),
                    resp.batch.has_retry(retry_policy),
                ) {
                    // An unsuccessful response with retry attempts remaining
                    (true, true) => Self::retry_batch(resp.batch, retry_tx),

                    // An unsuccessful response with no retry attempts remaining
                    (true, false) => {
                        log::warn!("Batch {} failed to send, no retry available", resp.batch.id);
                        match Self::run_cleanup(store, resp.batch) {
                            Ok(_) => (),
                            Err(e) => log::error!("{e}"),
                        }
                    }

                    // A successful response
                    (false, _) => {
                        log::info!("Sent batch {} of {batch_length} events", resp.batch.id);
                        match Self::run_cleanup(store, resp.batch) {
                            Ok(_) => (),
                            Err(e) => log::error!("{e}"),
                        }
                    }
                }
            }

            // The request to the collector failed - no response
            Err(failed_batch) => {
                if failed_batch.has_retry(retry_policy) {
                    Self::retry_batch(failed_batch, retry_tx)
                } else {
                    log::warn!(
                        "Batch {} failed to send, no retry available",
                        failed_batch.id
                    );
                    match Self::run_cleanup(store, failed_batch) {
                        Ok(_) => (),
                        Err(e) => log::error!("{e}"),
                    }
                }
            }
        }
    }

    // Sends an EventBatch to the collector
    async fn send_batch(
        batch: EventBatch,
        http_client: Box<dyn HttpClient + Send + Sync>,
    ) -> Result<SentBatchResponse, EventBatch> {
        match http_client.post(batch.as_payload()).await {
            Ok(code) => {
                log::debug!("Batch {} sent with status code {}", batch.id, code);
                Ok(SentBatchResponse { batch, code })
            }
            Err(e) => {
                log::warn!("Failed to send batch {}: {e}, re-queueing...", batch.id);
                Err(batch)
            }
        }
    }

    // Starts a tokio runtime and runs the emitter loop
    fn start_tokio(
        http_client: Box<dyn HttpClient + Send + Sync>,
        mut rx: tokio::sync::mpsc::Receiver<EmitterMessage>,
        event_store: Arc<Mutex<dyn EventStore + Send + Sync>>,
        retry_policy: RetryPolicy,
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
            let (retry_tx, mut retry_rx) = tokio::sync::mpsc::unbounded_channel();

            loop {
                // `rx.recv().await` will not resolve until either a message is received,
                // or the channel is closed and there are no more messages, in which case we exit the loop

                // select! is used to check both the `retry_rx` channel and the `rx` channel for new messages
                let message = match tokio::select! {
                    // `biased;` is used to ensure that the `retry_rx` channel is checked first, so retries get priority
                    biased;

                    retry = retry_rx.recv() => retry,
                    event = rx.recv() => event,
                } {
                    Some(message) => message,
                    None => break,
                };

                match message {
                    EmitterMessage::Send(batch) => {
                        // Clone to move into the task
                        let client = http_client.clone();
                        let retry_transmitter = retry_tx.clone();
                        let store = event_store.clone();

                        // Spawn a new task to send the batch
                        tokio_tasks.push(tokio::spawn(async move {
                            Self::batch_send_task(
                                batch,
                                client,
                                retry_transmitter,
                                store,
                                retry_policy,
                            )
                            .await
                        }));
                    }

                    // On break, the emitter and runtime will be dropped
                    //
                    // Tokio will cancel any running tasks once the runtime is dropped, meaning any queued or retry batches will be lost,
                    // so we attempt to send any remaining batches before exiting
                    EmitterMessage::Close => {
                        let remaining = tokio_tasks.len();
                        for (i, task) in tokio_tasks.iter_mut().enumerate() {
                            log::debug!("Waiting for task {}/{remaining} to complete", i + 1);
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
        log::debug!("Flushing event store");

        // Get a lock on the event store
        let mut store_lock = match self.event_store.lock() {
            Ok(store) => store,
            Err(e) => return Err(Error::EmitterError(e.to_string())),
        };

        // Send batches until the event store doesn't have enough events to fill a batch
        while let Ok(batch) = store_lock.full_batch() {
            if let Err(e) = self.tx.try_send(EmitterMessage::Send(batch)) {
                return Err(Error::EmitterError(e.to_string()));
            }
        }

        // Create a batch of the remaining events and send it
        let remaining_events = store_lock.len();
        let final_batch = store_lock.batch_of(remaining_events)?;
        if let Err(e) = self.tx.try_send(EmitterMessage::Send(final_batch)) {
            return Err(Error::EmitterError(e.to_string()));
        };

        log::debug!("Finished flushing event store");

        Ok(())
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

    #[test]
    fn should_retry() {
        let below_200 = (0..=199).collect::<Vec<_>>();
        let between_300_and_599 = (300..=599)
            .into_iter()
            .filter(|code| !DONT_RETRY_STATUS_CODES.contains(code))
            .collect::<Vec<_>>();

        let should_retry_codes = [below_200, between_300_and_599].concat();

        for code in 0..=599 {
            assert_eq!(
                BatchEmitter::should_retry(code),
                should_retry_codes.contains(&code)
            )
        }
    }
}
