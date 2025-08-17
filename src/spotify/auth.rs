use std::{sync::Arc, time::Duration};

use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    config, error,
    management::TokenManager,
    server::start_api_server,
    success,
    types::{PkceToken, Token},
    utils, warning,
};

/// Initiates the complete OAuth 2.0 PKCE authentication flow with Spotify.
///
/// This function orchestrates the entire authentication process including:
/// 1. Generating PKCE code verifier and challenge
/// 2. Starting a local callback server
/// 3. Opening the authorization URL in the user's browser
/// 4. Waiting for the OAuth callback
/// 5. Persisting the obtained token for future use
///
/// The PKCE (Proof Key for Code Exchange) flow provides enhanced security
/// for OAuth flows without requiring a client secret to be stored securely.
///
/// # Arguments
///
/// * `shared_state` - Thread-safe shared state for storing PKCE information
///   and the resulting token between the auth flow and callback handler
///
/// # Authentication Flow
///
/// 1. **PKCE Setup**: Generates a cryptographically secure code verifier and
///    derives the corresponding code challenge using SHA256
/// 2. **Server Start**: Launches a local HTTP server to handle the OAuth callback
/// 3. **Browser Launch**: Opens the Spotify authorization URL in the default browser
/// 4. **User Authorization**: User grants permissions in their browser
/// 5. **Callback Handling**: Local server receives the authorization code
/// 6. **Token Exchange**: Authorization code is exchanged for an access token
/// 7. **Token Persistence**: Token is saved for future API requests
///
/// # Error Handling
///
/// - Browser launch failures result in a warning with manual URL instructions
/// - Token persistence failures terminate the program with an error
/// - Authentication timeouts or failures terminate with an error message
///
/// # Security Features
///
/// - Uses PKCE flow to avoid storing client secrets
/// - Code verifier is generated with cryptographic randomness
/// - Authorization code is single-use and time-limited
/// - Tokens are stored securely in the local data directory
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// let shared_state = Arc::new(Mutex::new(None));
/// auth(shared_state).await;
/// ```
///
/// # User Experience
///
/// The function provides clear feedback throughout the process and handles
/// common failure scenarios gracefully. Users receive success confirmation
/// or clear error messages with next steps.
pub async fn auth(shared_state: Arc<Mutex<Option<PkceToken>>>) {
    // generate PKCE verifier and challenge
    let code_verifier = utils::generate_code_verifier();
    let code_challenge = utils::generate_code_challenge(&code_verifier);

    // start API server
    let server_state = Arc::clone(&shared_state);
    tokio::spawn(async move {
        start_api_server(server_state).await;
    });

    // Construct the authorization URL
    let auth_url = format!(
        "{spotify_auth_url}?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&code_challenge={code_challenge}&code_challenge_method=S256&scope={scope}",
        spotify_auth_url = &config::spotify_apiauth_url(),
        client_id = &config::spotify_client_id(),
        redirect_uri = &config::spotify_redirect_uri(),
        code_challenge = code_challenge,
        scope = &config::spotify_scope()
    );

    // Store verifier in shared state before redirect
    {
        let mut lock = shared_state.lock().await;
        *lock = Some(PkceToken {
            code_verifier: code_verifier.clone(),
            token: None,
        });
    }

    // Open the authorization URL in the default browser
    if webbrowser::open(&auth_url).is_err() {
        warning!(
            "Failed to open browser. Please navigate to the following URL manually:\n{}",
            auth_url
        )
    }

    // wait for callback to be hit
    let token = wait_for_token(shared_state).await;

    match token {
        Some(t) => {
            // initialize token manager with token
            let token_manager = TokenManager::new(t.clone());
            if let Err(e) = token_manager.persist().await {
                error!("Failed to save token to cache: {}", e);
            }

            success!("Authentication successful!");
        }
        None => {
            error!("Authentication failed or timeed out.");
        }
    }
}

/// Waits for the OAuth callback to complete and return a token.
///
/// Polls the shared state for a completed authentication token with a 60-second
/// timeout. This function runs concurrently with the callback handler that
/// populates the token after successful OAuth exchange.
///
/// # Arguments
///
/// * `shared_state` - Shared state containing the PKCE token information
///
/// # Returns
///
/// Returns `Some(Token)` if authentication completes successfully within the
/// timeout period, or `None` if the timeout is reached without a token.
///
/// # Timeout Behavior
///
/// - Maximum wait time: 60 seconds
/// - Polling interval: 1 second
/// - Non-blocking: Uses async sleep to avoid CPU spinning
///
/// # Concurrency
///
/// This function is designed to run concurrently with the HTTP server callback
/// handler. The shared state is safely accessed using async mutex locks.
///
/// # Example
///
/// ```
/// let token = wait_for_token(shared_state).await;
/// match token {
///     Some(t) => println!("Got token: {}", t.access_token),
///     None => println!("Authentication timed out"),
/// }
/// ```
async fn wait_for_token(shared_state: Arc<Mutex<Option<PkceToken>>>) -> Option<Token> {
    use std::time::Instant;

    let max_wait = Duration::from_secs(60);
    let start = Instant::now();

    while start.elapsed() < max_wait {
        let lock = shared_state.lock().await;
        if let Some(pkce_token) = lock.as_ref() {
            if let Some(token) = &pkce_token.token {
                return Some(token.clone());
            }
        }
        drop(lock);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    None
}

/// Refreshes an expired access token using a refresh token.
///
/// Exchanges a refresh token for a new access token when the current token
/// has expired. This allows the application to maintain authenticated access
/// without requiring the user to re-authorize.
///
/// # Arguments
///
/// * `refresh_token` - Valid refresh token obtained from previous authentication
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Token)` - New token with fresh access token and updated expiration
/// - `Err(String)` - Error message describing the failure
///
/// # Token Response
///
/// The new token contains:
/// - Fresh access token for API requests
/// - Same or new refresh token (may rotate)
/// - Updated expiration time
/// - Current timestamp as obtained_at
///
/// # Error Conditions
///
/// Common failures include:
/// - Network connectivity issues
/// - Invalid or expired refresh token
/// - Spotify API service errors
/// - Malformed response data
///
/// # Example
///
/// ```
/// let new_token = refresh_token("AQC...refresh_token").await?;
/// println!("New access token expires in {} seconds", new_token.expires_in);
/// ```
///
/// # API Documentation
///
/// Uses Spotify's token refresh endpoint with the "refresh_token" grant type
/// as specified in the OAuth 2.0 specification.
pub async fn refresh_token(refresh_token: &str) -> Result<Token, String> {
    let client = Client::new();
    let res = client
        .post(&config::spotify_apitoken_url())
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &config::spotify_client_id()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    Ok(Token {
        access_token: json["access_token"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        refresh_token: json["refresh_token"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        scope: json["scope"].as_str().unwrap_or_default().to_string(),
        expires_in: json["expires_in"].as_i64().unwrap_or(3600) as u64,
        obtained_at: Utc::now().timestamp() as u64,
    })
}

/// Exchanges an authorization code for an access token using PKCE.
///
/// Completes the OAuth 2.0 PKCE flow by exchanging the authorization code
/// received from the callback for an access token. This is the final step
/// in the authentication process.
///
/// # Arguments
///
/// * `code` - Authorization code received from the OAuth callback
/// * `verifier` - PKCE code verifier that was generated at the start of the flow
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Token)` - Complete token with access token, refresh token, and metadata
/// - `Err(reqwest::Error)` - HTTP error, network error, or API error
///
/// # PKCE Security
///
/// The code verifier proves that the same client that initiated the auth flow
/// is completing it, preventing authorization code interception attacks. The
/// verifier must match the challenge that was sent in the initial auth request.
///
/// # Token Contents
///
/// The returned token includes:
/// - Access token for API authentication
/// - Refresh token for token renewal
/// - Scope permissions granted by the user
/// - Expiration time in seconds
/// - Timestamp when the token was obtained
///
/// # Error Handling
///
/// Common failure scenarios:
/// - Invalid or expired authorization code
/// - Code verifier doesn't match the challenge
/// - Network connectivity issues
/// - Spotify API service errors
///
/// # Example
///
/// ```
/// let token = exchange_code_pkce("AQA...auth_code", "dBjftJeZ...verifier").await?;
/// println!("Access token: {}", token.access_token);
/// ```
///
/// # Security Note
///
/// The authorization code is single-use and expires quickly (typically 10 minutes).
/// The exchange should happen immediately after receiving the code.
pub async fn exchange_code_pkce(code: &str, verifier: &str) -> Result<Token, reqwest::Error> {
    let client_id = &config::spotify_client_id();
    let redirect_uri = &&config::spotify_redirect_uri();

    let client = Client::new();
    let res = client
        .post(&config::spotify_apitoken_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?;

    let json: Value = res.json().await?;

    Ok(Token {
        access_token: json["access_token"].as_str().unwrap().to_string(),
        refresh_token: json["refresh_token"].as_str().unwrap().to_string(),
        scope: json["scope"].as_str().unwrap().to_string(),
        expires_in: json["expires_in"].as_i64().unwrap() as u64,
        obtained_at: chrono::Utc::now().timestamp() as u64,
    })
}
