use crate::snowplow::emitter::Emitter;

pub struct Tracker {
    namespace: String,
    app_id: String,
    emitter: Emitter,
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
