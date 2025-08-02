use std::path::PathBuf;

use chrono::Utc;
use reqwest::Client;

use crate::{common, types::Token};

pub struct TokenManager {
    token: Token,
}

impl TokenManager {
    pub fn new(token: Token) -> Self {
        TokenManager { token }
    }

    pub async fn load() -> Result<Self, String> {
        let path = Self::token_path();
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let token: Token = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(Self { token });
    }

    pub async fn persist(&self) -> Result<(), String> {
        let path = Self::token_path();
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(&self.token).map_err(|e| e.to_string())?;
        async_fs::write(Self::token_path(), json)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_valid_token(&mut self) -> String {
        if self.is_expired() {
            if let Ok(new_token) = self.refresh_token().await {
                self.token = new_token;
                let _ = self.persist().await;
            }
        }

        self.token.access_token.clone()
    }

    fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as u64;
        now >= self.token.obtained_at + self.token.expires_in - 240
    }

    async fn refresh_token(&self) -> Result<Token, String> {
        let client = Client::new();
        let res = client
            .post(common::SPOTIFY_API_TOKEN_URL)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &self.token.refresh_token),
                ("client_id", common::SPOTIFY_API_AUTH_CLIENT_ID),
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

    fn token_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sporlcli/cache/token.json");
        path
    }

    pub fn current_token(&self) -> &Token {
        &self.token
    }
}
