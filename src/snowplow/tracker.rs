use crate::snowplow::Emitter;

pub struct Tracker {
    pub namespace: String,
    pub app_id: String,
    pub emitter: Emitter,
}

impl Tracker {
    pub fn new(namespace: String, app_id: String, emitter: Emitter) -> Tracker {
        Tracker {
            namespace,
            app_id,
            emitter,
        }
    }

    pub async fn track<T>(&self, event: T)
    where
     T: serde::Serialize
    {
        match self
            .emitter
            .post(event, &self.emitter.collector_url).await
        {
            Ok(resp) => println!("Got response: {resp:?}"),
            Err(e) => println!("Error: {e}"),
        }
    }
}
