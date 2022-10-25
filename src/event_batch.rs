// Copyright (c) 2022 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0,
// and you may not use this file except in compliance with the Apache License Version 2.0.
// You may obtain a copy of the Apache License Version 2.0 at http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the Apache License Version 2.0 is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the Apache License Version 2.0 for the specific language governing permissions and limitations there under.

use serde_json::json;

use crate::{payload::Payload, SelfDescribingJson};

const PAYLOAD_DATA_SCHEMA: &str =
    "iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4";

/// A batch of events to be sent to the collector.
#[derive(Debug)]
pub struct EventBatch {
    pub events: Vec<Payload>,
}

impl EventBatch {
    /// Creates a sendable payload from the batch.
    pub fn as_payload(&self) -> SelfDescribingJson {
        SelfDescribingJson {
            schema: PAYLOAD_DATA_SCHEMA.to_string(),
            data: json!(self.events),
        }
    }
}
