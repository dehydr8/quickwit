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

use std::str::FromStr;

use quickwit_common::uri::Uri;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::indexer::environment::{DEFAULT_INDEX_CONFIG_URI, INFER_INDEX_IDS};

use super::environment::{DEFAULT_INDEX_ID, INDEX_CONFIG_URI_MAP, INFER_INDEX_MAP};

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GCSObjectData {
    pub bucket: String,
    pub name: String,
    pub size: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
/// Event types that can be used to invoke the indexer CloudRun.
pub enum IndexerEvent {
    Custom {
        source_uri: String,
        index_id: String,
        index_config_uri: String,
    },
    GCS(GCSObjectData),
}

impl IndexerEvent {
    pub fn uri(&self) -> anyhow::Result<Uri> {
        let path: String = match self {
            IndexerEvent::Custom {
                index_id: _,
                source_uri,
                index_config_uri,
            } => [source_uri, "#", index_config_uri].join(""),
            IndexerEvent::GCS(event) => ["gs://", &event.bucket, "/", &event.name].join(""),
        };
        Uri::from_str(&path)
    }

    pub fn index_id(&self) -> anyhow::Result<String> {
        match self {
            IndexerEvent::Custom { index_id, .. } => Ok(index_id.clone()),
            // if INFER_INDEX_IDS is true, use the path to infer the index ID
            // else use DEFAULT_INDEX_ID
            IndexerEvent::GCS(event) => {
                if *INFER_INDEX_IDS {
                    // use event.name
                    // it can be in the following formats:
                    // - syslog/YYYY/MM/DD/xx.json
                    // - appengine.googleapis.com/request_log/YYYY/MM/DD/xx.json
                    //
                    // The following parts will be the key:
                    // - syslog
                    // - appengine.googleapis.com/request_log
                    //
                    // The value of the index_id will be fetched from the INFER_INDEX_MAP

                    let parts: Vec<&str> = event.name.split('/').collect();

                    if parts.len() < 5 {
                        return Err(anyhow::anyhow!(
                            "Unable to infer index ID from GCS event name: {}",
                            event.name
                        ));
                    }

                    // The prefix is all parts except the last 4
                    let prefix = parts[..parts.len() - 4].join("/");

                    let index_id = INFER_INDEX_MAP.get(&prefix).ok_or(anyhow::anyhow!(
                        "Unable to infer index ID from GCS event name: {}",
                        event.name
                    ))?;
                    Ok(index_id.to_string())
                } else {
                    Ok(DEFAULT_INDEX_ID.clone())
                }
            }
        }
    }

    pub fn index_config_uri(&self) -> anyhow::Result<String> {
        match self {
            IndexerEvent::Custom {
                index_config_uri, ..
            } => Ok(index_config_uri.clone()),
            IndexerEvent::GCS(_) => {
                let index_id = self.index_id()?;
                let index_config_uri = if let Some(uri) = INDEX_CONFIG_URI_MAP.get(&index_id) {
                    uri.clone()
                } else {
                    warn!("Unable to find index config URI for index ID: {}", index_id);
                    DEFAULT_INDEX_CONFIG_URI.clone()
                };
                Ok(index_config_uri)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_custom_event_uri() {
        let cust_event = json!({
            "source_uri": "gs://quickwit-test/test.json"
        });
        let parsed_cust_event: IndexerEvent = serde_json::from_value(cust_event).unwrap();
        assert_eq!(
            parsed_cust_event.uri().unwrap(),
            Uri::from_str("gs://quickwit-test/test.json").unwrap(),
        );
    }

    #[test]
    fn test_gcs_event_uri() {
        let gcs_event = json!({
            "bucket": "sample-bucket",
            "cacheControl": "max-age=6000",
            "componentCount": 1234,
            "contentDisposition": "attachment",
            "contentEncoding": "gzip",
            "contentLanguage": "en",
            "contentType": "text/plain",
            "crc32c": "AAAAAA==",
            "customerEncryption": {
              "keySha256": "Vb/C17P2fk35hguiD/pdLoXJk2j2NlmWmUmnOPsLtfA=",
              "encryptionAlgorithm": "AES256"
            },
            "etag": "COu8mb3Dn+kCEAE=",
            "eventBasedHold": true,
            "generation": 96883251,
            "id": "sample-bucket/MyFile/1234567",
            "kind": "storage#object",
            "kmsKeyName": "nulla",
            "md5Hash": "xrX0h3SqCoeoKidv+GvlBw==",
            "mediaLink": "https://www.googleapis.com/download/storage/v1/b/projectid-sample-bucket/o/MyFile?generation=1588778055917163\\u0026alt=media",
            "metadata": {},
            "metageneration": 1,
            "name": "MyFile",
            "retentionExpirationTime": "2021-12-25T21:04:32.279744Z",
            "selfLink": "https://www.googleapis.com/storage/v1/b/projectid-sample-bucket/o/MyFile",
            "size": 0,
            "storageClass": "MULTI_REGIONAL",
            "temporaryHold": false,
            "timeCreated": "2020-05-06T15:14:15.917Z",
            "timeDeleted": "2021-11-25T21:04:32.279744Z",
            "timeStorageClassUpdated": "1988-10-25T10:29:01.558Z",
            "updated": "2021-11-25T21:04:32.279744Z"
        });
        let gcs_event: IndexerEvent = serde_json::from_value(gcs_event).unwrap();
        assert_eq!(
            gcs_event.uri().unwrap(),
            Uri::from_str("gs://sample-bucket/MyFile").unwrap(),
        );
    }
}
