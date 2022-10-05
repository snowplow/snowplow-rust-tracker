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

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subject allows you to attach additional information about your application's environment
///
/// A Subject can be (attached to either a Tracker) where it will be sent with every Event, and/or
/// attached to an Event, with the Event-level subject taking priority over Tracker-level
#[derive(Serialize, Deserialize, Builder, Default, Clone, Debug)]
#[builder(setter(into, strip_option), default)]
pub struct Subject {
    /// Unique identifier for user
    #[serde(rename(serialize = "uid"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// The timezone label.
    ///
    /// Populates the `os_timezone` field.
    #[serde(rename(serialize = "tz"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// The language set on the device.
    ///
    /// Populates the `lang` field.
    #[serde(rename(serialize = "lang"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Custom IP address. It overrides the IP address used by default.
    ///
    /// Populates the `user_ipaddress` field.
    #[serde(rename(serialize = "ip"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Custom user-agent. It overrides the user-agent used by default.
    ///
    /// Populates the `useragent` field.
    #[serde(rename(serialize = "ua"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Domain user ID (UUIDv4).
    ///
    /// Populates the `domain_userid` field.
    /// Typically used to link native tracking to in-app browser events tracked using the JavaScript Tracker.
    #[serde(rename(serialize = "duid"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_user_id: Option<Uuid>,

    /// Network user ID (UUIDv4).
    ///
    /// Populates the `network_userid` field.
    /// Typically used to link native tracking to in-app browser events tracked using the JavaScript Tracker.
    /// Normally one would retrieve the network userid from the browser and pass it to the app.
    #[serde(rename(serialize = "tnuid"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_user_id: Option<Uuid>,

    /// Session user ID (UUIDv4)
    ///
    /// Unique identifier (UUID) for this visit of this user_id to this domain
    #[serde(rename(serialize = "sid"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_user_id: Option<Uuid>,
}

impl Subject {
    pub fn builder() -> SubjectBuilder {
        SubjectBuilder::default()
    }

    /// Merges another instance of [Subject], with self taking priority
    pub fn merge(self, other: Subject) -> Self {
        Self {
            user_id: self.user_id.or(other.user_id),
            timezone: self.timezone.or(other.timezone),
            language: self.language.or(other.language),
            ip_address: self.ip_address.or(other.ip_address),
            user_agent: self.user_agent.or(other.user_agent),
            domain_user_id: self.domain_user_id.or(other.domain_user_id),
            network_user_id: self.network_user_id.or(other.network_user_id),
            session_user_id: self.session_user_id.or(other.session_user_id),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_subject_all_fields() {
        let domain_user_id = Uuid::new_v4();
        let network_user_id = Uuid::new_v4();
        let session_user_id = Uuid::new_v4();
        let subject = Subject::builder()
            .user_id("user_1")
            .timezone("Europe/London")
            .language("en")
            .ip_address("0.0.0.0")
            .user_agent("Mozilla/Firefox")
            .domain_user_id(domain_user_id)
            .network_user_id(network_user_id)
            .session_user_id(session_user_id)
            .build()
            .unwrap();

        assert_eq!("user_1".to_string(), subject.user_id.unwrap());
        assert_eq!("Europe/London".to_string(), subject.timezone.unwrap());
        assert_eq!("en".to_string(), subject.language.unwrap());
        assert_eq!("0.0.0.0".to_string(), subject.ip_address.unwrap());
        assert_eq!("Mozilla/Firefox".to_string(), subject.user_agent.unwrap());
        assert_eq!(domain_user_id, subject.domain_user_id.unwrap());
        assert_eq!(network_user_id, subject.network_user_id.unwrap());
        assert_eq!(session_user_id, subject.session_user_id.unwrap());
    }

    #[test]
    fn test_build_subject_partial() {
        let subject = Subject::builder()
            .user_id("user_1")
            .timezone("Europe/London")
            .build()
            .unwrap();

        assert_eq!("user_1".to_string(), subject.user_id.unwrap());
        assert_eq!("Europe/London".to_string(), subject.timezone.unwrap());
        assert!(subject.language.is_none());
        assert!(subject.ip_address.is_none());
        assert!(subject.user_agent.is_none());
        assert!(subject.domain_user_id.is_none());
        assert!(subject.network_user_id.is_none());
        assert!(subject.session_user_id.is_none());
    }

    #[test]
    fn test_merge_subjects() {
        let sub_with_priority = Subject::builder().user_id("user_1").build().unwrap();
        let sub_to_merge = Subject::builder()
            .user_id("user_2")
            .ip_address("999.999.999.999")
            .build()
            .unwrap();

        let merged = sub_with_priority.merge(sub_to_merge);

        assert_eq!(merged.user_id.unwrap(), "user_1");
        assert_eq!(merged.ip_address.unwrap(), "999.999.999.999");
    }
}
