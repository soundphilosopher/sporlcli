use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{spotify, types::PkceToken};

pub async fn auth(shared_state: Arc<Mutex<Option<PkceToken>>>) {
    spotify::auth::auth(shared_state).await;
}
