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

use crate::emitter::BatchEmitter;
use crate::subject::Subject;
use crate::tracker::Tracker;

/// Main interface for the package, used to initialize trackers.
pub struct Snowplow;

impl Snowplow {
    /// Creates a new [Tracker] instance
    pub fn create_tracker(
        namespace: &str,
        app_id: &str,
        collector_url: &str,
        subject: Option<Subject>,
    ) -> Tracker {
        let emitter = BatchEmitter::new(collector_url);
        Tracker::new(namespace, app_id, emitter, subject)
    }
}
