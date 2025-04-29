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

use serde_json::Value;
use tracing::{Instrument, debug_span, error, info, info_span};

use super::cloudevent::CloudEvent;
use super::environment::{DISABLE_JANITOR, DISABLE_MERGE};
use super::ingest::{IngestArgs, ingest};
use super::model::IndexerEvent;
use crate::logger;
use crate::utils::CloudRunContainerContext;

async fn indexer_handler(event: CloudEvent<Value>) -> Result<Value, anyhow::Error> {
    let container_ctx = CloudRunContainerContext::load();
    let payload = serde_json::from_value::<IndexerEvent>(event.data)?;
    let index_id = payload.index_id()?;
    let index_config_uri = payload.index_config_uri()?;
    let ingest_res = ingest(IngestArgs {
        index_id: index_id.clone(),
        index_config_uri: index_config_uri.clone(),
        input_path: payload.uri()?,
        input_format: quickwit_config::SourceInputFormat::Json,
        vrl_script: None,
        // TODO: instead of clearing the cache, we use a cache and set its max
        // size with indexer_config.split_store_max_num_bytes
        clear_cache: true,
    })
    .instrument(debug_span!(
        "ingest",
        env.INDEX_CONFIG_URI = index_config_uri,
        env.INDEX_ID = index_id,
        env.DISABLE_MERGE = *DISABLE_MERGE,
        env.DISABLE_JANITOR = *DISABLE_JANITOR,
        cold = container_ctx.cold,
        container_id = container_ctx.container_id,
    ))
    .await;

    match ingest_res {
        Ok(stats) => {
            info!(stats=?stats, "Indexing succeeded");
            Ok(serde_json::to_value(stats)?)
        }
        Err(e) => {
            error!(err=?e, "Indexing failed");
            Err(anyhow::anyhow!("Indexing failed").into())
        }
    }
}

pub async fn handler(event: CloudEvent<Value>) -> Result<Value, anyhow::Error> {
    let request_id = event.id.clone();
    let mut response = indexer_handler(event)
        .instrument(info_span!("indexer_handler", request_id))
        .await;
    if let Err(e) = &response {
        error!(err=?e, "Handler failed");
    }
    if let Ok(Value::Object(ref mut map)) = response {
        map.insert("request_id".to_string(), Value::String(request_id));
    }
    logger::flush_tracer();
    response
}
