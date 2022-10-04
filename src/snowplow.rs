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

use crate::emitter::Emitter;
use crate::tracker::Tracker;

pub struct Snowplow;

/// Main interface for the package used to initialize trackers.
impl Snowplow {
    /// Creates a new Tracker instance that can be used to track events
    pub fn create_tracker(namespace: &str, app_id: &str, collector_url: &str) -> Tracker {
        let emitter = Emitter::new(collector_url);
        let tracker = Tracker::new(namespace, app_id, emitter);
        tracker
    }
}
