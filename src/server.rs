use axum::{Extension, Router, routing::get};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::{api, config, error, types::PkceToken};

pub async fn start_api_server(state: Arc<Mutex<Option<PkceToken>>>) {
    let app = Router::new()
        .route("/health", get(api::health))
        .route("/callback", get(api::callback).layer(Extension(state)));

    let addr = match SocketAddr::from_str(&config::server_addr()) {
        Ok(addr) => addr,
        Err(e) => error!("Failed to parse server address: {}", e),
    };

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
