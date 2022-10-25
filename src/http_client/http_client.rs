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

use crate::payload::SelfDescribingJson;
use crate::Error;

/// A HttpClient is responsible for sending events to the collector.
///
/// This is an async trait, using the [async_trait crate](https://crates.io/crates/async-trait).
///
/// Implement this trait to use your own HttpClient implementation on an [Emitter](crate::Emitter).
#[async_trait]
pub trait HttpClient {
    /// Send a [SelfDescribingJson] to the collector via POST
    async fn post(&self, payload: SelfDescribingJson) -> Result<(), Error>;
    /// Duplicate the HttpClient
    fn clone(&self) -> Box<dyn HttpClient + Send + Sync>;
}
