use std::{collections::HashMap, sync::Arc};

use axum::{Extension, extract::Query, response::Html};
use tokio::sync::Mutex;

use crate::{spotify, types::PkceToken, warning};

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
