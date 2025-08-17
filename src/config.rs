//! Configuration management for the Spotify Release Tracker.
//!
//! This module handles loading and accessing configuration values from environment
//! variables and `.env` files. It provides a centralized way to manage application
//! configuration including Spotify API credentials, server settings, and other
//! runtime parameters.
//!
//! The configuration system follows a hierarchical approach:
//! 1. Environment variables (highest priority)
//! 2. `.env` file in the local data directory
//! 3. Application defaults (where applicable)

use dotenv;
use std::{env, path::PathBuf};

/// Loads environment variables from a `.env` file in the local data directory.
///
/// Creates the necessary directory structure if it doesn't exist and loads
/// environment variables from a `.env` file located in the platform-specific
/// local data directory under `sporlcli/.env`. This allows users to store
/// configuration securely without hardcoding sensitive values.
///
/// # Directory Structure
///
/// The function looks for the `.env` file in:
/// - Linux: `~/.local/share/sporlcli/.env`
/// - macOS: `~/Library/Application Support/sporlcli/.env`
/// - Windows: `%LOCALAPPDATA%/sporlcli/.env`
///
/// # Returns
///
/// Returns `Ok(())` if the environment file is successfully loaded, or an error
/// string if directory creation or file loading fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The parent directory cannot be created
/// - The `.env` file cannot be read or parsed
///
/// # Example
///
/// ```
/// use sporlcli::config;
///
/// #[tokio::main]
/// async fn main() {
///     if let Err(e) = config::load_env().await {
///         eprintln!("Configuration error: {}", e);
///     }
/// }
/// ```
pub async fn load_env() -> Result<(), String> {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("sporlcli/.env");
    if let Some(parent) = path.parent() {
        async_fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }

    dotenv::from_path(path).expect("Failed to load .env file");
    Ok(())
}

/// Returns the server address for the local OAuth callback server.
///
/// Retrieves the `SERVER_ADDRESS` environment variable which specifies
/// the address and port where the local HTTP server should bind for
/// handling OAuth callbacks during the authentication flow.
///
/// # Panics
///
/// Panics if the `SERVER_ADDRESS` environment variable is not set.
///
/// # Example
///
/// ```
/// let addr = server_addr(); // e.g., "127.0.0.1:8080"
/// ```
pub fn server_addr() -> String {
    env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set")
}

/// Returns the Spotify user ID for API operations.
///
/// Retrieves the `SPOTIFY_USER_ID` environment variable which identifies
/// the Spotify user account for playlist creation and other user-specific
/// operations.
///
/// # Panics
///
/// Panics if the `SPOTIFY_USER_ID` environment variable is not set.
///
/// # Example
///
/// ```
/// let user_id = spotify_user(); // e.g., "username"
/// ```
pub fn spotify_user() -> String {
    env::var("SPOTIFY_USER_ID").expect("SPOTIFY_USER_ID must be set")
}

/// Returns the Spotify API client ID for authentication.
///
/// Retrieves the `SPOTIFY_API_AUTH_CLIENT_ID` environment variable which
/// contains the client ID obtained when registering the application with
/// Spotify's developer platform.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_AUTH_CLIENT_ID` environment variable is not set.
///
/// # Example
///
/// ```
/// let client_id = spotify_client_id(); // e.g., "abc123..."
/// ```
pub fn spotify_client_id() -> String {
    env::var("SPOTIFY_API_AUTH_CLIENT_ID").expect("SPOTIFY_API_AUTH_CLIENT_ID must be set")
}

/// Returns the Spotify API client secret for authentication.
///
/// Retrieves the `SPOTIFY_API_AUTH_CLIENT_SECRET` environment variable which
/// contains the client secret obtained when registering the application with
/// Spotify's developer platform. This is used for secure authentication.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_AUTH_CLIENT_SECRET` environment variable is not set.
///
/// # Security Note
///
/// The client secret should be kept confidential and never exposed in logs
/// or version control.
///
/// # Example
///
/// ```
/// let client_secret = spotify_client_secret(); // e.g., "def456..."
/// ```
pub fn spotify_client_secret() -> String {
    env::var("SPOTIFY_API_AUTH_CLIENT_SECRET").expect("SPOTIFY_API_AUTH_CLIENT_SECRET must be set")
}

/// Returns the Spotify OAuth redirect URI.
///
/// Retrieves the `SPOTIFY_API_REDIRECT_URI` environment variable which specifies
/// the callback URL that Spotify should redirect to after user authorization.
/// This must match the redirect URI registered in the Spotify application settings.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_REDIRECT_URI` environment variable is not set.
///
/// # Example
///
/// ```
/// let redirect_uri = spotify_redirect_uri(); // e.g., "http://localhost:8080/callback"
/// ```
pub fn spotify_redirect_uri() -> String {
    env::var("SPOTIFY_API_REDIRECT_URI").expect("SPOTIFY_API_REDIRECT_URI must be set")
}

/// Returns the Spotify API scope permissions.
///
/// Retrieves the `SPOTIFY_API_AUTH_SCOPE` environment variable which defines
/// the scope of permissions requested during OAuth authentication. The scope
/// determines what API operations the application can perform on behalf of the user.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_AUTH_SCOPE` environment variable is not set.
///
/// # Example
///
/// ```
/// let scope = spotify_scope(); // e.g., "user-follow-read playlist-modify-public"
/// ```
pub fn spotify_scope() -> String {
    env::var("SPOTIFY_API_AUTH_SCOPE").expect("SPOTIFY_API_AUTH_SCOPE must be set")
}

/// Returns the Spotify OAuth authorization URL.
///
/// Retrieves the `SPOTIFY_API_AUTH_URL` environment variable which contains
/// the base URL for Spotify's OAuth authorization endpoint. This is where
/// users are redirected to grant permissions to the application.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_AUTH_URL` environment variable is not set.
///
/// # Example
///
/// ```
/// let auth_url = spotify_apiauth_url(); // e.g., "https://accounts.spotify.com/authorize"
/// ```
pub fn spotify_apiauth_url() -> String {
    env::var("SPOTIFY_API_AUTH_URL").expect("SPOTIFY_API_AUTH_URL must be set")
}

/// Returns the Spotify Web API base URL.
///
/// Retrieves the `SPOTIFY_API_URL` environment variable which contains the
/// base URL for Spotify's Web API endpoints. This is used for all API
/// operations after authentication.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_URL` environment variable is not set.
///
/// # Example
///
/// ```
/// let api_url = spotify_apiurl(); // e.g., "https://api.spotify.com/v1"
/// ```
pub fn spotify_apiurl() -> String {
    env::var("SPOTIFY_API_URL").expect("SPOTIFY_API_URL must be set")
}

/// Returns the Spotify OAuth token exchange URL.
///
/// Retrieves the `SPOTIFY_API_TOKEN_URL` environment variable which contains
/// the URL for exchanging authorization codes for access tokens during the
/// OAuth flow. This is used in the final step of authentication.
///
/// # Panics
///
/// Panics if the `SPOTIFY_API_TOKEN_URL` environment variable is not set.
///
/// # Example
///
/// ```
/// let token_url = spotify_apitoken_url(); // e.g., "https://accounts.spotify.com/api/token"
/// ```
pub fn spotify_apitoken_url() -> String {
    env::var("SPOTIFY_API_TOKEN_URL").expect("SPOTIFY_API_TOKEN_URL must be set")
}
