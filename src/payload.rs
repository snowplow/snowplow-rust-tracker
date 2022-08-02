use super::event::SelfDescribingJson;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventType {
    #[serde(rename(serialize = "pv"))]
    PageView,
    #[serde(rename(serialize = "pp"))]
    PagePing,
    #[serde(rename(serialize = "ue"))]
    LinkClick,
    #[serde(rename(serialize = "ue"))]
    AdImpression,
    #[serde(rename(serialize = "tr"))]
    EcommerceTransactionTr,
    #[serde(rename(serialize = "ti"))]
    EcommerceTransactionTi,
    #[serde(rename(serialize = "se"))]
    StructuredEvent,
    #[serde(rename(serialize = "ue"))]
    SelfDescribingEvent,
}

pub struct BatchPayload {
    pub id: u64,
    pub payloads: Vec<Payload>,
}

#[derive(Builder, Serialize, Deserialize, Default, Clone, Debug)]
#[builder(field(public))]
#[builder(pattern = "owned")]
pub struct Payload {
    p: String,
    tv: String,
    pub eid: uuid::Uuid,
    dtm: String,
    stm: String,
    #[builder(setter(strip_option))]
    e: Option<EventType>,
    aid: String,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    ue_pr: Option<SelfDescribingJson>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    ue_px: Option<String>,
    // Stuctured Event
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    se_ca: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    se_ac: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    se_la: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    se_pr: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    se_va: Option<u64>,
}

impl Payload {
    pub fn builder() -> PayloadBuilder {
        PayloadBuilder::default()
    }
}
