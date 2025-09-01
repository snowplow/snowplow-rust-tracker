# Tests Module - Testing Patterns Documentation

## Testing Overview

The test suite uses Snowplow Micro (a lightweight Snowplow collector) in Docker containers for integration testing. Tests validate event tracking, batching, retry logic, and error handling using real HTTP communication rather than mocks for comprehensive validation.

## Test Infrastructure

### Snowplow Micro Setup
```rust
// ✅ Correct: Container lifecycle management
let docker = Cli::default();
let (_container, micro_url) = setup(&docker);
// _container kept alive for test duration

// ❌ Wrong: Manual container management
let container = docker.run(micro_image);
// Forgot to stop container
```

### Test Tracker Configuration
```rust
// ✅ Correct: Single-event batching for deterministic tests
fn test_tracker(url: &str) -> Tracker {
    let store = InMemoryEventStore::new(
        capacity: 1,    // Send immediately
        batch_size: 1   // One event per batch
    );
    // ...
}

// ❌ Wrong: Production batching in tests
InMemoryEventStore::new(1000, 100) // Non-deterministic timing
```

## Integration Test Patterns

### Event Verification Pattern
```rust
// ✅ Correct: Wait for events then verify
tracker.track(event, None)?;
wait_for_events(&micro_url, "good", 1).await;
let events = micro_endpoint(&micro_url, "good").await;
assert_eq!(events.len(), 1);

// ❌ Wrong: Check immediately
tracker.track(event, None)?;
let events = micro_endpoint(&micro_url, "good").await; // Race condition
```

### Async Test Structure
```rust
// ✅ Correct: Use tokio::test macro
#[tokio::test]
async fn test_async_operation() {
    // Async test body
}

// ❌ Wrong: Block on runtime manually
#[test]
fn test_async_operation() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async { ... });
}
```

## Common Test Utilities

### wait_for_events Helper
```rust
// ✅ Correct: Polling with timeout
async fn wait_for_events(url: &str, endpoint: &str, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let count = get_event_count(url, endpoint).await;
        if count >= expected { return; }
        if Instant::now() > deadline { panic!("Timeout"); }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ❌ Wrong: Fixed sleep
tokio::time::sleep(Duration::from_secs(2)).await; // Flaky
```

### Micro Endpoint Helpers
```rust
// ✅ Correct: Structured endpoint access
async fn micro_endpoint(base_url: &str, endpoint: &str) -> Value {
    reqwest::get(format!("{}/micro/{}", base_url, endpoint))
        .await?.json().await?
}

// ❌ Wrong: Hardcoded URLs
reqwest::get("http://localhost:9090/micro/good").await?
```

## Test Categories

### Good Events Testing
Tests that events are correctly formatted and accepted:
```rust
// ✅ Correct: Verify in "good" endpoint
wait_for_events(&url, "good", 1).await;
assert!(micro_endpoint(&url, "bad").await.is_empty());

// ❌ Wrong: Only check event was sent
tracker.track(event, None)?; // No verification
```

### Bad Events Testing
Tests that malformed events are rejected:
```rust
// ✅ Correct: Verify in "bad" endpoint
let malformed_event = create_malformed_event();
tracker.track(malformed_event, None)?;
wait_for_events(&url, "bad", 1).await;

// ❌ Wrong: Expect error on track
tracker.track(malformed_event, None).unwrap_err(); // Track succeeds, validation is async
```

### Retry Logic Testing
```rust
// ✅ Correct: Use FlakeyHttpClient
let client = FlakeyHttpClient::new(vec![false, false, true]);
// Fails twice, succeeds on third attempt

// ❌ Wrong: Real network failures
// Unpredictable and environment-dependent
```

## Event Builder Testing

### Complete Event Testing
```rust
// ✅ Correct: Test all fields
let event = ScreenViewEvent::builder()
    .id(Uuid::new_v4())
    .name("test_screen")
    .previous_name("prev_screen")
    .build()?;
assert_all_fields_serialized(&event);

// ❌ Wrong: Partial testing
let event = ScreenViewEvent::builder()
    .name("test").build()?; // Missing required fields
```

### Subject Merging Tests
```rust
// ✅ Correct: Test priority order
let tracker_subject = Subject::builder().user_id("tracker").build()?;
let event_subject = Subject::builder().user_id("event").build()?;
// Event subject should take priority
assert_eq!(merged.user_id, Some("event".into()));

// ❌ Wrong: Assume no merging
// Test only one subject source
```

## Performance Testing

### Batch Performance Tests
```rust
// ✅ Correct: Measure with realistic batches
let store = InMemoryEventStore::new(1000, 100);
let start = Instant::now();
for _ in 0..1000 { tracker.track(event.clone(), None)?; }
tracker.flush()?;
assert!(start.elapsed() < Duration::from_secs(5));

// ❌ Wrong: Micro-benchmarks
let start = Instant::now();
tracker.track(event, None)?; // Single event timing meaningless
```

### Memory Usage Tests
```rust
// ✅ Correct: Test store limits
let store = InMemoryEventStore::new(100, 10);
for _ in 0..200 {
    tracker.track(event.clone(), None)?;
}
// Should not exceed capacity

// ❌ Wrong: Unbounded testing
let events = vec![event; 1_000_000]; // OOM risk
```

## Error Simulation

### Network Error Simulation
```rust
// ✅ Correct: Deterministic failure
impl HttpClient for FailingClient {
    async fn send_post(&self, _: &str, _: String) -> Result<Response> {
        Err(Error::EmitterError("Network error".into()))
    }
}

// ❌ Wrong: Random failures
if rand::random::<f32>() > 0.5 { Err(...) } // Non-deterministic
```

### Timeout Simulation
```rust
// ✅ Correct: Controlled delays
impl HttpClient for SlowClient {
    async fn send_post(&self, _: &str, _: String) -> Result<Response> {
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok(Response { code: 200 })
    }
}

// ❌ Wrong: Actual network timeouts
// Depends on network conditions
```

## Test Data Patterns

### UUID Generation
```rust
// ✅ Correct: Unique IDs per test
let event_id = Uuid::new_v4();

// ❌ Wrong: Hardcoded IDs
let event_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
// Can cause conflicts
```

### JSON Test Data
```rust
// ✅ Correct: Use serde_json::json! macro
let data = json!({
    "key": "value",
    "nested": { "field": 123 }
});

// ❌ Wrong: String concatenation
let data = format!(r#"{{"key": "{}"}}"#, value);
```

## Container Management

### Testcontainers Best Practices
```rust
// ✅ Correct: Let testcontainers handle lifecycle
let docker = Cli::default();
let container = docker.run(GenericImage::new("snowplow/micro", "latest"));
let port = container.get_host_port_ipv4(9090);

// ❌ Wrong: Manual Docker commands
std::process::Command::new("docker")
    .args(&["run", "-d", "snowplow/micro"]);
```

### Port Management
```rust
// ✅ Correct: Dynamic port allocation
let port = container.get_host_port_ipv4(9090);
let url = format!("http://localhost:{}", port);

// ❌ Wrong: Hardcoded ports
let url = "http://localhost:9090"; // Port conflicts
```

## Test Cleanup

### Emitter Cleanup
```rust
// ✅ Correct: Always close emitter
let mut tracker = test_tracker(&url);
// ... test logic ...
tracker.close_emitter()?; // Essential

// ❌ Wrong: Rely on Drop
// Emitter thread may outlive test
```

### Resource Cleanup Pattern
```rust
// ✅ Correct: RAII with defer pattern
let _guard = defer(|| cleanup_resources());
// Test logic here
// Cleanup runs on scope exit

// ❌ Wrong: Manual cleanup
cleanup_resources(); // May not run if test panics
```

## Debugging Failed Tests

### Enable Logging
```bash
# ✅ Correct: Use RUST_LOG
RUST_LOG=debug cargo test test_name

# ❌ Wrong: println! debugging
println!("Event: {:?}", event); // Not captured in tests
```

### Inspect Micro State
```rust
// ✅ Correct: Check all endpoints
dbg!(micro_endpoint(&url, "good").await);
dbg!(micro_endpoint(&url, "bad").await);
dbg!(micro_endpoint(&url, "all").await);

// ❌ Wrong: Assume endpoint
// Only checking "good" misses validation failures
```

## Quick Reference

### New Integration Test Checklist
- [ ] Set up Snowplow Micro container
- [ ] Create test tracker with batch_size=1
- [ ] Track events
- [ ] Wait for events with timeout
- [ ] Verify in correct endpoint (good/bad)
- [ ] Close emitter
- [ ] Container cleanup automatic via testcontainers

### Test Flakiness Checklist
- [ ] Use `wait_for_events` not sleep
- [ ] Set batch_size=1 for deterministic timing
- [ ] Use unique IDs per test
- [ ] Check for port conflicts
- [ ] Verify container started successfully
- [ ] Add timeout to async operations

### Performance Test Checklist
- [ ] Use realistic batch sizes
- [ ] Measure end-to-end time including flush
- [ ] Test memory limits
- [ ] Test concurrent operations
- [ ] Clean up resources properly

## Contributing to tests/CLAUDE.md

This file documents test-specific patterns. When updating:

1. Focus on integration test patterns with Snowplow Micro
2. Document deterministic test strategies
3. Include container management patterns
4. Test helper functions should be reusable
5. Ensure tests are reproducible and not flaky