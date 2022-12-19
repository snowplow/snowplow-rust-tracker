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

use std::fmt::{Display, Formatter, Result};

/// The errors that can occur when using the Snowplow Tracker
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred when trying to build an event or payload
    BuilderError(String),
    /// An error occurred in the emitter
    EmitterError(String),
    /// An error occurred in the event store
    EventStoreError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::BuilderError(builder_err) => write!(f, "{}", builder_err),
            Error::EmitterError(emitter_err) => write!(f, "{}", emitter_err),
            Error::EventStoreError(event_store_err) => write!(f, "{}", event_store_err),
        }
    }
}

impl std::error::Error for Error {}

// This allows us to use `#[builder(build_fn(error = "Error"))]` on builders
// to return `Error` instead of `UninitializedFieldError`
impl From<derive_builder::UninitializedFieldError> for Error {
    fn from(e: derive_builder::UninitializedFieldError) -> Error {
        Error::BuilderError(e.to_string())
    }
}
