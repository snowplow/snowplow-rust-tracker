use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
pub struct SelfDescribingEvent {
    pub event: Event,
    pub json: SelfDescribingJson,
}

#[derive(Serialize, Deserialize)]
pub struct SelfDescribingData {
    pub schema: String,
    pub data: Value,
}

#[derive(Deserialize)]
pub struct SelfDescribingJson {
    pub schema: String,
    pub data: SelfDescribingData,
}

impl SelfDescribingJson {
    pub fn from_schema_and_data(schema: String, data: String) -> Result<Self, serde_json::Error> {
        let data: Value = serde_json::from_str(&data)?;
        Ok(SelfDescribingJson {
            schema: "iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0"
                .to_string(),
            data: SelfDescribingData { schema, data },
        })
    }
}

// The collector expects the `data` field of the `SelfDescribingJson` to be an object,
// but the SelfDescribingJson to be a string, so we have to manually serialize SelfDescribingJson
impl Serialize for SelfDescribingJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &json!({
                "schema": self.schema,
                "data": self.data,
            })
            .to_string(),
        )
    }
}

#[derive(Serialize, Deserialize, Builder)]
pub struct StructuredEvent {
    pub category: String,
    pub action: String,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    // serde isn't happy with u128 and I'm not sure why
    pub value: Option<u64>,
}

impl StructuredEvent {
    pub fn builder() -> StructuredEventBuilder {
        StructuredEventBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Builder)]
pub struct ScreenViewEvent {
    pub name: String,
    pub id: String,
}

impl ScreenViewEvent {
    pub fn builder() -> ScreenViewEventBuilder {
        ScreenViewEventBuilder::default()
    }
}
