# Snowplow Rust Tracker - Project Documentation

## Project Overview

The Snowplow Rust Tracker is a library for tracking behavioral analytics events to Snowplow collectors. It provides a robust, async-first implementation with configurable retry policies, event batching, and flexible event storage mechanisms. The tracker supports multiple event types (Self-Describing, Structured, ScreenView, Timing) and allows attaching contextual subject data to events.

**Key Technologies:**
- Rust 2021 Edition
- Tokio async runtime
- Reqwest HTTP client
- Serde for serialization
- derive_builder for ergonomic builders
- UUID v4 for event IDs

## Development Commands

```bash
# Build the project
cargo build --verbose

# Run tests
cargo test --verbose

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run security audit
cargo audit

# Generate documentation
cargo doc --no-deps --open

# Run with logging
RUST_LOG=debug cargo test
```

## Architecture

### System Design

The tracker follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────┐
│             Public API Layer                 │
│         (Snowplow, Tracker)                 │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│            Event Layer                       │
│  (SelfDescribingEvent, StructuredEvent,     │
│   ScreenViewEvent, TimingEvent)             │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│          Emitter Layer                       │
│    (BatchEmitter, RetryPolicy)              │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│     Storage & Transport Layer                │
│  (EventStore, HttpClient, EventBatch)       │
└──────────────────────────────────────────────┘
```

### Threading Model

The tracker uses a dedicated thread with Tokio runtime for async event emission:
- Main thread: Event creation and queueing
- Emitter thread: Async batch sending with retry logic
- Communication via `tokio::sync::mpsc` channels

## Core Architectural Principles

### 1. Builder Pattern Everywhere
All complex types use the builder pattern via `derive_builder`:
```rust
// ✅ Correct: Use builders for complex types
let event = SelfDescribingEvent::builder()
    .schema("iglu:com.example/event/jsonschema/1-0-0")
    .data(json!({"key": "value"}))
    .build()?;

// ❌ Wrong: Direct struct construction
let event = SelfDescribingEvent { schema: "...", data: json!({}) };
```

### 2. Error Handling via Result Type
All fallible operations return `Result<T, Error>`:
```rust
// ✅ Correct: Propagate errors properly
tracker.track(event, None)?;

// ❌ Wrong: Unwrap in library code
tracker.track(event, None).unwrap();
```

### 3. Trait-Based Extensibility
Key components use traits for flexibility:
```rust
// ✅ Correct: Accept trait implementations
pub trait Emitter {
    fn add(&mut self, payload: PayloadBuilder) -> Result<(), Error>;
}

// ❌ Wrong: Hardcode concrete types
pub struct Tracker {
    emitter: BatchEmitter, // Should be Box<dyn Emitter>
}
```

### 4. Async-First Design
Network operations are async with Tokio:
```rust
// ✅ Correct: Async network calls
async fn send_batch(batch: EventBatch) -> Result<Response, Error>

// ❌ Wrong: Blocking network calls in async context
fn send_batch(batch: EventBatch) -> Result<Response, Error>
```

## Layer Organization & Responsibilities

### Public API Layer (`src/snowplow.rs`, `src/tracker.rs`)
- **Snowplow**: Factory for creating trackers
- **Tracker**: Main tracking interface, manages emitter and subject

### Event Layer (`src/event.rs`)
- Event types implementing `PayloadAddable` trait
- Schema validation through Iglu URIs
- Subject attachment at event level

### Emitter Layer (`src/emitter/`)
- **BatchEmitter**: Batches events and handles async sending
- **RetryPolicy**: Configurable retry strategies
- Thread management and channel communication

### Storage Layer (`src/event_store/`)
- **EventStore trait**: Abstract event persistence
- **InMemoryEventStore**: Default in-memory implementation

### Transport Layer (`src/http_client/`)
- **HttpClient trait**: Abstract HTTP operations
- **ReqwestClient**: Default Reqwest implementation

## Critical Import Patterns

### Module Organization
```rust
// ✅ Correct: Use mod.rs for module exports
// src/emitter/mod.rs
mod batch_emitter;
mod emitter;
pub use batch_emitter::BatchEmitter;
pub use emitter::Emitter;

// ❌ Wrong: Export private implementations
pub mod batch_emitter; // Exposes internals
```

### Re-exports in lib.rs
```rust
// ✅ Correct: Clean public API
pub use emitter::{BatchEmitter, Emitter, RetryPolicy};
pub use event::{SelfDescribingEvent, StructuredEvent};

// ❌ Wrong: Deep module paths in public API
pub mod emitter;
// Users need: snowplow_tracker::emitter::BatchEmitter
```

## Essential Library Patterns

### Builder Pattern with derive_builder
```rust
// ✅ Correct: Consistent builder configuration
#[derive(Builder)]
#[builder(setter(into, strip_option))]
#[builder(build_fn(error = "Error"))]
pub struct Subject {
    #[builder(default)]
    pub user_id: Option<String>,
}

// ❌ Wrong: Manual builder implementation
impl SubjectBuilder {
    pub fn user_id(mut self, id: String) -> Self { ... }
}
```

### Serde Serialization
```rust
// ✅ Correct: Field renaming and conditional serialization
#[derive(Serialize)]
pub struct Subject {
    #[serde(rename = "uid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

// ❌ Wrong: Serialize None values
#[derive(Serialize)]
pub struct Subject {
    pub user_id: Option<String>, // Sends "user_id": null
}
```

### Arc<Mutex<T>> for Thread-Safe Sharing
```rust
// ✅ Correct: Thread-safe event store
event_store: Arc<Mutex<dyn EventStore + Send + Sync>>

// ❌ Wrong: Non-thread-safe sharing
event_store: Rc<RefCell<dyn EventStore>>
```

## Model Organization Pattern

### Event Hierarchy
All events implement `PayloadAddable`:
```rust
// ✅ Correct: Implement required trait
impl PayloadAddable for CustomEvent {
    fn add_to_payload(self, builder: PayloadBuilder) -> PayloadBuilder
    fn subject(&self) -> &Option<Subject>
}

// ❌ Wrong: Event without trait implementation
pub struct CustomEvent { ... } // Can't be tracked
```

### Subject Merging Pattern
```rust
// ✅ Correct: Event subject takes priority
let merged = event_subject.merge(tracker_subject);

// ❌ Wrong: Tracker subject overwrites event
let merged = tracker_subject.merge(event_subject);
```

## Common Pitfalls & Solutions

### 1. Emitter Lifecycle Management
```rust
// ✅ Correct: Always close the emitter
tracker.close_emitter()?;

// ❌ Wrong: Let emitter drop without closing
// Events may be lost
```

### 2. UUID Generation
```rust
// ✅ Correct: Use UUID v4 for event IDs
let event_id = Uuid::new_v4();

// ❌ Wrong: String-based IDs
let event_id = "event_123".to_string();
```

### 3. Timestamp Handling
```rust
// ✅ Correct: Unix epoch milliseconds
SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()

// ❌ Wrong: Seconds or formatted strings
SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
```

### 4. Test Container Usage
```rust
// ✅ Correct: Use testcontainers for integration tests
let docker = Cli::default();
let (_container, url) = setup(&docker);

// ❌ Wrong: Mock HTTP client in integration tests
let client = MockHttpClient::new();
```

## File Structure Template

```
snowplow-rust-tracker/
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── snowplow.rs               # Main factory interface
│   ├── tracker.rs                # Core tracker implementation
│   ├── event.rs                  # Event types and traits
│   ├── subject.rs                # Subject data model
│   ├── error.rs                  # Error types
│   ├── payload.rs                # Payload building
│   ├── event_batch.rs            # Batch management
│   ├── emitter/
│   │   ├── mod.rs               # Module exports
│   │   ├── emitter.rs           # Emitter trait
│   │   ├── batch_emitter.rs    # Async batch implementation
│   │   └── retry_policy.rs     # Retry configuration
│   ├── event_store/
│   │   ├── mod.rs
│   │   ├── event_store.rs      # Storage trait
│   │   └── in_memory_event_store.rs
│   └── http_client/
│       ├── mod.rs
│       ├── http_client.rs      # HTTP trait
│       └── reqwest_client.rs   # Reqwest implementation
└── tests/
    ├── common/                  # Shared test utilities
    │   ├── mod.rs
    │   ├── micro.rs            # Snowplow Micro setup
    │   └── flakey_http_client.rs
    ├── test_events.rs          # Event tracking tests
    └── test_batch_emitter.rs   # Emitter tests
```

## Quick Reference

### Event Creation Checklist
- [ ] Use builder pattern
- [ ] Validate schema URI format
- [ ] Handle builder errors with `?`
- [ ] Attach subject if needed
- [ ] Check JSON data conforms to schema

### New Event Type Checklist
- [ ] Implement `PayloadAddable` trait
- [ ] Add `#[derive(Builder)]` with proper attributes
- [ ] Configure serde field renaming
- [ ] Add `subject` field with `#[serde(skip_serializing)]`
- [ ] Export from `lib.rs`
- [ ] Add integration tests

### Testing Checklist
- [ ] Use `#[tokio::test]` for async tests
- [ ] Set up Snowplow Micro container
- [ ] Wait for events with `wait_for_events`
- [ ] Always close emitter in tests
- [ ] Test both success and failure cases

## Contributing to CLAUDE.md

When adding or updating content in this document, please follow these guidelines:

### File Size Limit
- **CLAUDE.md must not exceed 40KB** (currently ~19KB)
- Check file size after updates: `wc -c CLAUDE.md`
- Remove outdated content if approaching the limit

### Code Examples
- Keep all code examples **4 lines or fewer**
- Focus on the essential pattern, not complete implementations
- Use `// ❌` and `// ✅` to clearly show wrong vs right approaches

### Content Organization
- Add new patterns to existing sections when possible
- Create new sections sparingly to maintain structure
- Update the architectural principles section for major changes
- Ensure examples follow current codebase conventions

### Quality Standards
- Test any new patterns in actual code before documenting
- Verify imports and syntax are correct for the codebase
- Keep language concise and actionable
- Focus on "what" and "how", minimize "why" explanations

### Multiple CLAUDE.md Files
- **Directory-specific CLAUDE.md files** can be created for specialized modules
- Follow the same structure and guidelines as this root CLAUDE.md
- Keep them focused on directory-specific patterns and conventions
- Maximum 20KB per directory-specific CLAUDE.md file

### Instructions for LLMs
When editing files in this repository, **always check for CLAUDE.md guidance**:

1. **Look for CLAUDE.md in the same directory** as the file being edited
2. **If not found, check parent directories** recursively up to project root
3. **Follow the patterns and conventions** described in the applicable CLAUDE.md
4. **Prioritize directory-specific guidance** over root-level guidance when conflicts exist