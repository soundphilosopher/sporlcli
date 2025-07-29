use axum::{Extension, Router, routing::get};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::{api, info, types::PkceToken};

pub async fn start_api_server(state: Arc<Mutex<Option<PkceToken>>>) {
    let app = Router::new()
        .route("/health", get(api::health))
        .route("/callback", get(api::callback).layer(Extension(state)));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9900));
    info!("Server started at {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
