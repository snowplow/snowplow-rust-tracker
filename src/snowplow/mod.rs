pub mod emitter;
pub mod event;
pub mod tracker;

use emitter::Emitter;
use std::fmt::{self, Formatter};
use tracker::Tracker;

#[derive(Debug, Clone)]
pub struct NoSuchTracker {
    pub app_id: String,
    pub namespace: String,
}

impl fmt::Display for NoSuchTracker {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Tracker not found: {:?}", self)
    }
}
pub struct Snowplow {
    trackers: Vec<Tracker>,
}

// The "main object that manages trackers" concept will be difficult to implement for rust
impl Snowplow {
    pub fn new() -> Snowplow {
        Snowplow {
            trackers: Vec::new(),
        }
    }

    /*
    When you create a tracker, you require a mutable reference to the Snowplow object you
    have created using Snowplow::new(). This mutable reference will live until the returned value,
    the new tracker, is out of scope.

    You are only allowed a _single_ mutable reference to a variable, otherwise the compiler will complain
    We will likely (for now) just have the user manage it all, meaning:

    Remove the `main Snowplow object` concept entirely, and provide direct access to:
        - Tracker
        - Emitter

    2.
    */
    pub fn create_tracker(
        &mut self,
        namespace: String,
        app_id: String,
        emitter: Emitter,
    ) -> &Tracker {
        let trackers = &mut self.trackers;
        let tracker = Tracker::new(&namespace, &app_id, emitter);
        trackers.push(tracker);
        trackers.last().unwrap()
    }

    pub fn remove_tracker(
        &mut self,
        namespace: String,
        app_id: String,
    ) -> Result<(), NoSuchTracker> {
        let index = self
            .trackers
            .iter()
            .position(|t| t.app_id == app_id && t.namespace == namespace);

        match index {
            Some(i) => {
                self.trackers.remove(i);
                Ok(())
            }
            None => Err(NoSuchTracker { namespace, app_id }),
        }
    }

    pub fn get_tracker(&self, namespace: String, app_id: String) -> Option<&Tracker> {
        match self
            .trackers
            .iter()
            .position(|t| t.app_id == app_id && t.namespace == namespace)
        {
            Some(pos) => self.trackers.get(pos),
            _ => None,
        }
    }
}
