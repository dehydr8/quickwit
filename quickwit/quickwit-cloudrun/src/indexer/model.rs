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
    Custom { source_uri: String },
    GCS(GCSObjectData),
}

impl IndexerEvent {
    pub fn uri(&self) -> anyhow::Result<Uri> {
        let path: String = match self {
            IndexerEvent::Custom { source_uri } => source_uri.clone(),
            IndexerEvent::GCS(event) => [
                "gs://",
                &event.bucket,
                "/",
                &event.name,
            ]
            .join(""),
        };
        Uri::from_str(&path)
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
