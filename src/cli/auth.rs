use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::{
    common, error,
    management::TokenManager,
    server::start_api_server,
    success,
    types::{PkceToken, Token},
    utils, warning,
};

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
        "https://accounts.spotify.com/authorize?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&code_challenge={code_challenge}&code_challenge_method=S256&scope={scope}",
        client_id = common::SPOTIFY_API_AUTH_CLIENT_ID,
        redirect_uri = common::SPOTIFY_API_REDIRECT_URI,
        code_challenge = code_challenge,
        scope = common::SPOTIFY_API_AUTH_SCOPE
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
            if let Err(e) = token_manager.save_to_cache().await {
                error!("Failed to save token to cache: {}", e);
            }

            success!("Authentication successful!");
        }
        None => {
            error!("Authentication failed or timeed out.");
        }
    }
}

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
