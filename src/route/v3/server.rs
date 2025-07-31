use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use api_derive::get;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::byond::status::status;

#[get("/server")]
pub(super) async fn server() -> String {
    get_server_status().await
}

static LAST_SERVER_STATUS: LazyLock<RwLock<Option<(Instant, String)>>> =
    LazyLock::new(|| RwLock::new(None));

async fn get_server_status() -> String {
    {
        let last_server_status = LAST_SERVER_STATUS.read().await;
        if let Some((last_update, server_status)) = &*last_server_status {
            if last_update.elapsed() < Duration::from_secs(30) {
                return server_status.clone();
            }
        }
    }

    let mut should_cache = false;
    let mut response = Vec::new();

    // remove
    let servers = [Server {
        name: "Server 1".to_string(),
        address: "10.253.0.1:3131".to_string(),
        connection_address: "play.ss13.tr:3131".to_string(),
        error_message: "No error".to_string(),
    }];

    #[derive(Serialize)]
    struct Server {
        name: String,
        address: String,
        connection_address: String,
        error_message: String,
    }
    //

    for server in servers.iter() {
        let status = status(&server.address).await.ok();

        if !should_cache && status.is_some() {
            should_cache = true;
        }

        response.push(status); // remove unwrap
    }

    let response = serde_json::to_string(&response).unwrap_or_else(|_| "[]".to_string());

    if should_cache {
        let mut last_server_status = LAST_SERVER_STATUS.write().await;
        *last_server_status = Some((Instant::now(), response.clone()));
    }

    response
}
