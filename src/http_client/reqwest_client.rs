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

use async_trait::async_trait;
use reqwest::Client;

use crate::{Error, HttpClient, SelfDescribingJson};

const POST_PATH: &str = "com.snowplowanalytics.snowplow/tp2";

/// A [HttpClient] implementation useing the reqwest crate to send events to the collector.
pub struct ReqwestClient {
    pub client: reqwest::Client,
    pub collector_url: String,
}

impl ReqwestClient {
    pub fn new(collector_url: &str) -> Box<ReqwestClient> {
        Box::new(ReqwestClient {
            client: Client::new(),
            collector_url: collector_url.to_string(),
        })
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn post(&self, payload: SelfDescribingJson) -> Result<u16, Error> {
        let collector_url = format!("{}/{}", self.collector_url, POST_PATH);

        match self.client.post(&collector_url).json(&payload).send().await {
            Ok(resp) => Ok(resp.status().as_u16()),
            Err(e) => Err(Error::EmitterError(format!("POST request failed: {e}"))),
        }
    }

    fn clone(&self) -> Box<dyn HttpClient + Send + Sync> {
        Box::new(ReqwestClient {
            client: self.client.clone(),
            collector_url: self.collector_url.clone(),
        })
    }
}
