use std::{collections::HashMap, sync::Arc};

use axum::{Extension, extract::Query, response::Html};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    config,
    types::{PkceToken, Token},
    warning,
};

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

        match exchange_code_pkce(code, &verifier).await {
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

async fn exchange_code_pkce(code: &str, verifier: &str) -> Result<Token, reqwest::Error> {
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
