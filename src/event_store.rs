use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::payload::{BatchPayload, Payload};


/**
TODO
- change trait so it does not take mutex arc hardcoded -> generic type
- EventStore trait is reachable outside and injectable in Emitter
- instatiate InMemory in main
- InMemory is reachable outside
- Create add and remove
**/
pub trait EventStore {  // TODO - use generic instead of store explicit
    fn add_event(&self, payload: Payload) -> bool;
    fn get_event_batch(&self, batch_id: &Arc<AtomicU64>, amount: u32) -> Option<BatchPayload>;
    fn delete_by_ids(&self, ids: Vec<uuid::Uuid>) -> bool;
}

#[derive(Debug)]
pub struct InMemoryEventStore {
    pub store: Arc<Mutex<Vec<Payload>>>
}

impl EventStore for InMemoryEventStore {

    fn add_event(&self, payload: Payload) -> bool {
        match self.store.lock() {
            Ok(mut guard) => {
                guard.push(payload);
                drop(guard);
                true
            }
            _ => false,
        }
    }

    fn get_event_batch(&self, batch_id: &Arc<AtomicU64>, amount: u32) -> Option<BatchPayload> {
        let bid = batch_id
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| Some(v + 1))
            .unwrap_or(0);

        self.store.lock()
            .map_or(None, | guard |
                Some(if guard.iter().count() < amount as usize {
                    None // TODO - return error and handle at above layer
                } else {
                    let slice = &guard[..amount as usize];
                    Some(
                        BatchPayload {
                            id: bid,
                            payloads: slice.to_vec(),
                        }
                    )
                })
            )
            .flatten()
    }

    fn delete_by_ids(&self, ids: Vec<uuid::Uuid>) -> bool {
        match self.store.lock() {
            Ok(mut guard) => {
                guard.retain(| payload | !ids.contains(&payload.eid) );
                drop(guard);
                true
            }
            _ => false,
        }
    }
}
