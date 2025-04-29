// Copyright 2021-Present Datadog, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// CloudEvent spec: https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md
/// 
/// This implementation supports CloudEvents in both JSON and HTTP header formats:
/// 
/// 1. JSON format with `application/cloudevents+json` content type
/// 2. Binary HTTP binding with headers starting with `ce-`
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CloudEvent<T> {
    /// Identifies the event. Producers MUST ensure that source + id is unique for each distinct event.
    #[serde(rename = "id")]
    pub id: String,

    /// Identifies the context in which an event happened.
    #[serde(rename = "source")]
    pub source: String,

    /// The version of the CloudEvents specification which the event uses.
    #[serde(rename = "specversion")]
    pub spec_version: String,

    /// Describes the type of event related to the originating occurrence.
    #[serde(rename = "type")]
    pub event_type: String,

    /// Describes the subject of the event in the context of the event producer.
    #[serde(rename = "subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    
    /// The payload or data associated with the CloudEvent.
    #[serde(rename = "data")]
    pub data: T,
}

impl<T> CloudEvent<T> 
where 
    T: for<'de> Deserialize<'de> + Serialize,
{
    /// Create a CloudEvent from HTTP headers.
    /// Headers should include "ce-id", "ce-source", "ce-specversion", "ce-type", and optionally "ce-subject".
    pub fn from_headers(headers: &HashMap<String, String>, data: T) -> Option<Self> {
        let id = headers.get("ce-id")?.to_string();
        let source = headers.get("ce-source")?.to_string();
        let spec_version = headers.get("ce-specversion")?.to_string();
        let event_type = headers.get("ce-type")?.to_string();
        let subject = headers.get("ce-subject").map(|s| s.to_string());

        debug!("Creating CloudEvent from headers: id={}, source={}, type={}", id, source, event_type);

        Some(CloudEvent {
            id,
            source,
            spec_version,
            event_type,
            subject,
            data,
        })
    }

    /// Checks if the CloudEvent contains the required attributes according to the CloudEvents spec.
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty() 
            && !self.source.is_empty() 
            && !self.spec_version.is_empty() 
            && !self.event_type.is_empty()
    }
} 