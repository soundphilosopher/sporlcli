use axum::{Extension, Router, routing::get};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::{api, config, error, types::PkceToken};

/// Starts the API server for handling OAuth callbacks and health checks.
///
/// Creates and configures an Axum web server with routes for OAuth authentication
/// callbacks and health monitoring. The server is designed to handle the OAuth
/// PKCE flow by providing a callback endpoint that receives authorization codes
/// from Spotify and exchanges them for access tokens.
///
/// The server includes the following routes:
/// - `/health` - Health check endpoint for monitoring server status
/// - `/callback` - OAuth callback endpoint for receiving authorization codes
///
/// The server runs indefinitely and will bind to the address specified in the
/// application configuration. It uses shared state to manage the PKCE token
/// throughout the authentication flow.
///
/// # Arguments
///
/// * `state` - Shared state containing the PKCE token information, wrapped in
///   Arc<Mutex<>> for thread-safe access across request handlers
///
/// # Panics
///
/// This function will panic if:
/// - The server address from configuration cannot be parsed
/// - The server fails to bind to the specified address
/// - The server encounters an unrecoverable error during operation
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
/// use crate::types::PkceToken;
///
/// let state = Arc::new(Mutex::new(None::<PkceToken>));
/// start_api_server(state).await;
/// ```
///
/// # Note
///
/// This function runs indefinitely and should typically be spawned in a
/// separate task or used as the main server loop. The server will continue
/// running until the process is terminated or an unrecoverable error occurs.
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
