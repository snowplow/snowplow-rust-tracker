use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub true_timestamp: i64,
    pub context: Vec<SelfDescribingJson>,
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

#[derive(Serialize, Deserialize)]
struct SelfDescribingEvent {
    pub event: Event,
    pub json: SelfDescribingJson,
}

#[derive(Serialize, Deserialize)]
pub struct SelfDescribingJson {
    pub schema: String,
    pub data: Value,
}

#[derive(Serialize, Deserialize)]
struct StructuredEvent {
    pub event: Event,
    pub category: String,
    pub action: String,
    pub property: String,
    pub label: String,
    pub value: u128,
}

#[derive(Serialize, Deserialize)]
pub struct ScreenViewEvent {
    pub event: Event,
    pub name: String,
    pub id: String,
}
