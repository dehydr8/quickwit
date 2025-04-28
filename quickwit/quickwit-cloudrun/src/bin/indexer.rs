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

use quickwit_cloudrun::logger;
use serde_json::Value;
use std::net::SocketAddr;
use std::env;
use warp;
use warp::Filter;
use quickwit_cloudrun::indexer::{handler, CloudEvent};
use quickwit_proto::bytes::Bytes;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::setup_cloudrun_tracer(tracing::Level::INFO)?;
    
    // Define the handler for the indexer
    let indexer = warp::post()
        .and(warp::path::end())
        .and(warp::header::header("content-type"))
        .and(warp::body::bytes())
        .and_then(|content_type: String, body: Bytes| async move {
            if !content_type.eq_ignore_ascii_case("application/cloudevents+json") {
                return Err(warp::reject::custom(InvalidContentType));
            }

            let payload: CloudEvent<Value> = serde_json::from_slice(&body)
            .map_err(|_| warp::reject::custom(InvalidJson))?;
            
            let result = handler(payload).await;
            match result {
                Ok(value) => Ok(warp::reply::json(&value)),
                Err(e) => {
                    eprintln!("Error handling request: {:?}", e);
                    Err(warp::reject::custom(InternalError))
                }
            }
        });

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting indexer on {}", addr);

    warp::serve(indexer)
        .bind(addr)
        .await;
    
    Ok(())
}


#[derive(Debug)]
struct InvalidContentType;
impl warp::reject::Reject for InvalidContentType {}

#[derive(Debug)]
struct InvalidJson;
impl warp::reject::Reject for InvalidJson {}

#[derive(Debug)]
struct InternalError;
impl warp::reject::Reject for InternalError {}