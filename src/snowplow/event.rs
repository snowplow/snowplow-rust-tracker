use serde_json::Value;

pub struct Event {
    true_timestamp: i64,
    context: Vec<SelfDescribingJson>,
}

impl Event {
    pub fn set_context(&mut self, contexts: Vec<SelfDescribingJson>) -> &mut Event {
        self.context = contexts;
        self
    }

    pub fn set_true_timestamp(&mut self, timestamps: i64) -> &mut Event {
        self.true_timestamp = timestamps;
        self
    }
}


struct SelfDescribingEvent {
    event: Event,
    json: SelfDescribingJson,
}


pub struct SelfDescribingJson {
    schema: String,
    data: Value,
}


struct StructuredEvent {
    event: Event,
    category: String,
    action: String,
    property: String,
    label: String,
    value: u128,
}

struct ScreenViewEvent {
    event: Event,
    name: String,
    id: String,
}