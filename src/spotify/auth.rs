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
