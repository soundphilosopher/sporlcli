use dotenv;
use std::{env, path::PathBuf};

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

pub fn server_addr() -> String {
    env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set")
}

pub fn spotify_user() -> String {
    env::var("SPOTIFY_USER_ID").expect("SPOTIFY_USER_ID must be set")
}

pub fn spotify_client_id() -> String {
    env::var("SPOTIFY_API_AUTH_CLIENT_ID").expect("SPOTIFY_API_AUTH_CLIENT_ID must be set")
}

pub fn spotify_client_secret() -> String {
    env::var("SPOTIFY_API_AUTH_CLIENT_SECRET").expect("SPOTIFY_API_AUTH_CLIENT_SECRET must be set")
}

pub fn spotify_redirect_uri() -> String {
    env::var("SPOTIFY_API_REDIRECT_URI").expect("SPOTIFY_API_REDIRECT_URI must be set")
}

pub fn spotify_scope() -> String {
    env::var("SPOTIFY_API_AUTH_SCOPE").expect("SPOTIFY_API_AUTH_SCOPE must be set")
}

pub fn spotify_apiauth_url() -> String {
    env::var("SPOTIFY_API_AUTH_URL").expect("SPOTIFY_API_AUTH_URL must be set")
}

pub fn spotify_apiurl() -> String {
    env::var("SPOTIFY_API_URL").expect("SPOTIFY_API_URL must be set")
}

pub fn spotify_apitoken_url() -> String {
    env::var("SPOTIFY_API_TOKEN_URL").expect("SPOTIFY_API_TOKEN_URL must be set")
}
