use std::{collections::HashMap, sync::Arc};

use axum::{Extension, extract::Query, response::Html};
use tokio::sync::Mutex;

use crate::{spotify, types::PkceToken, warning};

/// Handles OAuth callback requests from Spotify's authorization server.
///
/// This endpoint is called by Spotify after the user completes the authorization
/// process. It receives an authorization code via query parameters and exchanges
/// it for an access token using the PKCE (Proof Key for Code Exchange) flow.
///
/// The function performs the following steps:
/// 1. Extracts the authorization code from query parameters
/// 2. Retrieves the stored PKCE code verifier from shared state
/// 3. Exchanges the authorization code for an access token
/// 4. Stores the resulting token in the shared state
/// 5. Returns an HTML response indicating success or failure
///
/// # Arguments
///
/// * `params` - Query parameters from the OAuth callback URL, expected to contain
///   the "code" parameter with the authorization code from Spotify
/// * `shared_state` - Thread-safe shared state containing the PKCE token information,
///   including the code verifier needed for token exchange
///
/// # Returns
///
/// Returns an `Html<&'static str>` response with:
/// - Success message if authentication completes successfully
/// - Error message if any step in the process fails
///
/// # OAuth Flow Context
///
/// This handler is part of the OAuth 2.0 PKCE flow:
/// 1. User is redirected to Spotify for authorization
/// 2. User grants permissions
/// 3. Spotify redirects back to this callback with an authorization code
/// 4. This handler exchanges the code for an access token
/// 5. The token is stored for future API requests
///
/// # Error Conditions
///
/// The function handles several error scenarios:
/// - Missing authorization code in query parameters
/// - Missing PKCE code verifier in shared state
/// - Token exchange failure (network issues, invalid code, etc.)
///
/// # Security Notes
///
/// - Uses PKCE flow for enhanced security without client secret exposure
/// - The code verifier is stored temporarily and used only once
/// - The authorization code is single-use and expires quickly
///
/// # Example Response HTML
///
/// Success: "Authentication successful. Close browser window."
/// Error: "Login failed." or "Missing PKCE token."
///
/// # Example
///
/// ```text
/// // Called automatically by Spotify's OAuth redirect:
/// // GET /callback?code=AQC...&state=xyz
/// ```
pub async fn callback(
    Query(params): Query<HashMap<String, String>>,
    Extension(shared_state): Extension<Arc<Mutex<Option<PkceToken>>>>,
) -> Html<&'static str> {
    if let Some(code) = params.get("code") {
        let mut state = shared_state.lock().await;
        // Take code verifier from state
        let Some(ref mut pkce_state) = state.as_mut() else {
            return Html("<h4>Missing PKCE code verifier.</h4>");
        };

        let verifier = pkce_state.code_verifier.clone();

        match spotify::auth::exchange_code_pkce(code, &verifier).await {
            Ok(token) => {
                pkce_state.token = Some(token.clone());
                Html("<h2>Authentication successful.</h2><p>Cloese browser window.</p>")
            }
            Err(e) => {
                warning!("Token exchange failed: {}", e);
                Html("<h4>Login failed.</h4>")
            }
        }
    } else {
        Html("<h4>Missing PKCE token.</h4>")
    }
}
