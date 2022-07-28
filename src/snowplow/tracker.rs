use crate::snowplow::emitter::Emitter;

pub struct Tracker {
    pub namespace: String,
    pub app_id: String,
    pub emitter: Emitter,
}

impl Tracker {
    pub fn new(namespace: String, app_id: String) -> Tracker {
        Tracker {
            namespace,
            app_id,
            emitter: Emitter::new(),
        }
    }
}
