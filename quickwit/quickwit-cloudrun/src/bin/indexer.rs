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

use quickwit_cloudrun::indexer::{CloudEvent, handler};
use quickwit_cloudrun::logger;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, warn};
use warp;
use warp::Filter;
use warp::http::HeaderMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::setup_cloudrun_tracer(tracing::Level::INFO)?;

    // Define the handler for the indexer
    let indexer = warp::post()
        .and(warp::path::end())
        .and(warp::header::headers_cloned())
        .and(warp::body::json())
        .and_then(|headers: HeaderMap, body: Value| async move {
            // Check if we have CloudEvent headers
            let ce_headers = extract_cloudevent_headers(&headers);

            match CloudEvent::from_headers(&ce_headers, body) {
                Some(cloudevent) => {
                    if !cloudevent.is_valid() {
                        warn!("Invalid CloudEvent headers: {:?}", ce_headers);
                        return Err(warp::reject::custom(InternalError));
                    }

                    let result = handler(cloudevent).await;
                    return match result {
                        Ok(value) => Ok(warp::reply::json(&value)),
                        Err(e) => {
                            warn!("Error handling request: {:?}", e);
                            Err(warp::reject::custom(InternalError))
                        }
                    };
                }
                None => {
                    warn!("Invalid headers or body: {:?}", ce_headers);
                    return Err(warp::reject::custom(InternalError));
                }
            }
        });

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting indexer on {}", addr);

    let (_, server) = warp::serve(indexer).bind_with_graceful_shutdown(addr, async {
        // Wait for CTRL+C or SIGTERM
        signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl_c signal");
        info!("CTRL+C received, starting graceful shutdown");
    });

    server.await;

    Ok(())
}

/// Extract CloudEvent headers from the request headers
/// CloudEvent headers are prefixed with "ce-" in HTTP binding
fn extract_cloudevent_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut ce_headers = HashMap::new();
    
    for (key, value) in headers.iter() {
        let key_str = key.to_string().to_lowercase();
        if key_str.starts_with("ce-") {
            if let Ok(value_str) = value.to_str() {
                ce_headers.insert(key_str, value_str.to_string());
            }
        }
    }
    
    ce_headers
}

#[derive(Debug)]
struct InternalError;
impl warp::reject::Reject for InternalError {}
