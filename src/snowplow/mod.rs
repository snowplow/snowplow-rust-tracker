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

    pub fn remove_tracker(self: &mut Snowplow, namespace: String, app_id: String) {
        let mut index = 0;
        for (i, t) in self.trackers.iter().enumerate() {
            if t.app_id == app_id && t.namespace == namespace {
                index = i;
                break;
            }
        }

        self.trackers.remove(index);
    }

    pub fn get_tracker(
        self: &mut Snowplow,
        namespace: String,
        app_id: String,
    ) -> Option<&mut Tracker> {
        let mut index = 0;
        for (i, t) in self.trackers.iter().enumerate() {
            if t.app_id == app_id && t.namespace == namespace {
                index = i;
                break;
            }
        }

        Some(self.trackers.get_mut(index).unwrap())
    }
}
