use dotenv::dotenv;
use std::env;

pub fn load_env() {
    dotenv().expect("Failed to load .env file");
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
