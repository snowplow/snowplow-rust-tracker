# Emitter Module - Specialized Documentation

## Module Overview

The emitter module is responsible for the async event delivery pipeline in the Snowplow Rust Tracker. It manages event batching, retry logic, thread coordination, and HTTP communication with Snowplow collectors. This module implements a producer-consumer pattern with a dedicated Tokio runtime thread.

## Architectural Design

### Thread Architecture

```
Main Thread                    Emitter Thread (Tokio Runtime)
    │                                    │
    ├─ add(payload) ──────►             │
    │                      Channel       │
    │                  (mpsc::Sender)    │
    │                          │         │
    │                          ▼         │
    │                    ┌──────────┐   │
    │                    │ Receiver │   │
    │                    └────┬─────┘   │
    │                         │         │
    │                         ▼         │
    │                  ┌─────────────┐  │
    │                  │ Event Loop  │  │
    │                  │  - Batch    │  │
    │                  │  - Send     │  │
    │                  │  - Retry     │  │
    │                  └─────────────┘  │
    └────────────────────────────────────┘
```

## Core Patterns

### Emitter Trait Implementation
```rust
// ✅ Correct: Thread-safe trait object
emitter: Box<dyn Emitter + 'static>

// ❌ Wrong: Concrete type coupling
emitter: BatchEmitter
```

### Channel Communication Pattern
```rust
// ✅ Correct: Bounded channel with backpressure
let (tx, rx) = tokio::sync::mpsc::channel(100);

// ❌ Wrong: Unbounded channel (memory issues)
let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
```

### Thread Spawning Pattern
```rust
// ✅ Correct: Store handle for cleanup
executor_handle: Option<std::thread::JoinHandle<()>>

// ❌ Wrong: Detached thread
std::thread::spawn(|| { ... }); // No way to join
```

## BatchEmitter Implementation

### Builder Configuration
```rust
// ✅ Correct: Fluent builder with defaults
BatchEmitter::builder()
    .collector_url("https://collector.snowplow.io")
    .retry_policy(RetryPolicy::MaxRetries(10))
    .event_store(InMemoryEventStore::default())
    .build()?

// ❌ Wrong: Missing required fields
BatchEmitter::builder().build()? // No collector_url
```

### Retry Policy Patterns

#### Exponential Backoff
```rust
// ✅ Correct: Jittered exponential backoff
let delay = Duration::from_millis(
    (100 * 2_u64.pow(attempt)).min(60000) + rand::random::<u64>() % 1000
);

// ❌ Wrong: Fixed delay
let delay = Duration::from_secs(5);
```

#### Retry Decision Logic
```rust
// ✅ Correct: Check status codes properly
fn should_retry(code: u16) -> bool {
    !Self::is_successful_response(code) && 
    !DONT_RETRY_STATUS_CODES.contains(&code)
}

// ❌ Wrong: Retry on all non-200
fn should_retry(code: u16) -> bool { code != 200 }
```

### Event Batching Strategy

```rust
// ✅ Correct: Respect batch size limits
if batch.events.len() >= batch_size {
    send_batch(batch).await?;
}

// ❌ Wrong: Unbounded batch growth
batch.events.push(event); // No size check
```

## Async Patterns

### Tokio Runtime Management
```rust
// ✅ Correct: Dedicated runtime thread
std::thread::spawn(move || {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async { ... });
});

// ❌ Wrong: Block main thread
tokio::runtime::Runtime::new()?.block_on(async { ... });
```

### Graceful Shutdown
```rust
// ✅ Correct: Send close message and join thread
tx.send(EmitterMessage::Close).await?;
if let Some(handle) = executor_handle.take() {
    handle.join().map_err(|_| Error::EmitterError(...))?;
}

// ❌ Wrong: Drop without cleanup
drop(emitter); // May lose events
```

## Error Handling Patterns

### Network Error Recovery
```rust
// ✅ Correct: Differentiate error types
match client.send(batch).await {
    Ok(resp) if should_retry(resp.code) => retry_batch(batch),
    Ok(_) => cleanup_batch(batch),
    Err(e) if is_network_error(&e) => retry_batch(batch),
    Err(e) => log_and_drop(batch, e),
}

// ❌ Wrong: Treat all errors the same
client.send(batch).await.unwrap_or_else(|_| retry_batch(batch))
```

### Lock Acquisition
```rust
// ✅ Correct: Handle poisoned mutex
let store = match event_store.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
};

// ❌ Wrong: Panic on poisoned mutex
let store = event_store.lock().unwrap();
```

## Performance Optimizations

### Batch Processing
```rust
// ✅ Correct: Process multiple messages per iteration
while let Ok(msg) = rx.try_recv() {
    process_message(msg);
    if processed >= MAX_BATCH { break; }
}

// ❌ Wrong: One message per iteration
if let Some(msg) = rx.recv().await {
    process_message(msg);
}
```

### Event Store Access
```rust
// ✅ Correct: Minimize lock duration
let batch = {
    let mut store = event_store.lock()?;
    store.take_batch()?
}; // Lock released here
send_batch(batch).await?;

// ❌ Wrong: Hold lock during I/O
let mut store = event_store.lock()?;
send_batch(store.take_batch()?).await?; // Lock held during network call
```

## Testing Patterns

### Flaky HTTP Client for Tests
```rust
// ✅ Correct: Deterministic failure simulation
pub struct FlakeyHttpClient {
    fail_pattern: Vec<bool>, // [true, false, false] = fail first, succeed next two
}

// ❌ Wrong: Random failures in tests
if rand::random::<bool>() { return Err(...) }
```

### Async Test Helpers
```rust
// ✅ Correct: Timeout for async operations
tokio::time::timeout(Duration::from_secs(5), async {
    wait_for_events(&url, "good", expected_count).await
}).await??;

// ❌ Wrong: Unbounded wait
wait_for_events(&url, "good", expected_count).await;
```

## Common Issues & Solutions

### Issue: Events Lost on Shutdown
```rust
// ✅ Solution: Flush before closing
emitter.flush()?;
emitter.close()?;

// ❌ Problem: Direct close
emitter.close()?; // Pending events lost
```

### Issue: Memory Growth
```rust
// ✅ Solution: Bounded event store
InMemoryEventStore::new(capacity: 1000, batch_size: 100)

// ❌ Problem: Unbounded storage
events: Vec<Event> // No limit
```

### Issue: Thread Panic Handling
```rust
// ✅ Solution: Catch panics in thread
std::thread::spawn(move || {
    std::panic::catch_unwind(|| {
        runtime.block_on(event_loop)
    }).unwrap_or_else(|e| log::error!("Emitter panic: {:?}", e))
});

// ❌ Problem: Unhandled panic
std::thread::spawn(move || {
    runtime.block_on(event_loop) // Panic kills thread silently
});
```

## Module-Specific Constants

```rust
// Retry configuration
const DONT_RETRY_STATUS_CODES: [u16; 6] = [400, 401, 403, 404, 410, 422];
const MAX_BACKOFF_MS: u64 = 60000;
const INITIAL_BACKOFF_MS: u64 = 100;

// Batch configuration  
const DEFAULT_BATCH_SIZE: usize = 100;
const DEFAULT_EVENT_STORE_CAPACITY: usize = 1000;
```

## Integration Points

### With EventStore
- Thread-safe access via `Arc<Mutex<dyn EventStore>>`
- Batch retrieval with `take_events_batch()`
- Cleanup with `cleanup_after_send_attempt()`

### With HttpClient
- Async send via `send_post()`
- Response handling with status codes
- Network error differentiation

### With Tracker
- Event addition via channel
- Flush support
- Graceful shutdown

## Quick Reference

### Adding New Retry Strategy
- [ ] Implement in `RetryPolicy` enum
- [ ] Update `should_retry_batch()` logic
- [ ] Add backoff calculation
- [ ] Test with `FlakeyHttpClient`

### Implementing Custom Emitter
- [ ] Implement `Emitter` trait
- [ ] Handle thread lifecycle
- [ ] Implement flush mechanism
- [ ] Ensure graceful shutdown
- [ ] Add integration tests

### Debugging Event Loss
- [ ] Check `close_emitter()` called
- [ ] Verify `flush()` before close
- [ ] Monitor channel capacity
- [ ] Check retry policy settings
- [ ] Enable debug logging

## Contributing to emitter/CLAUDE.md

This file documents emitter-specific patterns. When updating:

1. Focus on async/threading patterns unique to this module
2. Document retry and backoff strategies
3. Keep examples specific to event emission
4. Test patterns with mock HTTP clients
5. Ensure thread-safety in all examples