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
use quickwit_cloudrun::searcher::setup_searcher_api;
use std::env;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::setup_cloudrun_tracer(tracing::Level::INFO)?;
    let routes = setup_searcher_api().await?;

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting searcher on {}", addr);

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
        // Wait for CTRL+C or SIGTERM
        signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl_c signal");
        info!("CTRL+C received, starting graceful shutdown");
    });

    server.await;

    Ok(())
}
