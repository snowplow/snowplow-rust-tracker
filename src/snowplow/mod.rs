mod emitter;
mod tracker;
mod event;

use self::tracker::Tracker;

pub struct Snowplow {
    trackers: Vec<Tracker>,
}

impl Snowplow {
    pub fn new() -> Snowplow {
        Snowplow {
            trackers: Vec::new(),
        }
    }

    pub fn create_tracker(self: &mut Snowplow, namespace: String, app_id: String) -> &mut Tracker {
        let tracker = Tracker::new(namespace, app_id);
        self.trackers.push(tracker);
        self.trackers.last_mut().unwrap()
    }
}
