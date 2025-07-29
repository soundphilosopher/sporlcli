use std::{io::Error, path::PathBuf};

pub const STATE_TYPE_ARTISTS: &str = "state_artists";
pub const STATE_TYPE_RELEASES: &str = "state_releases";

#[derive(Debug)]
pub enum StateError {
    IoError(Error),
    CriticalError(String),
    SerdeError(serde_json::Error),
}

impl From<Error> for StateError {
    fn from(err: Error) -> Self {
        StateError::IoError(err)
    }
}

pub struct StateManager {
    state_type: String,
    state: Vec<String>,
}

impl StateManager {
    pub fn new(state_type: String) -> Self {
        Self {
            state_type,
            state: Vec::new(),
        }
    }

    pub fn add(&mut self, item: String) {
        self.state.push(item);
    }

    pub fn get_state(&self) -> &Vec<String> {
        &self.state
    }

    pub async fn persist(&self) -> Result<(), StateError> {
        let path = Self::get_path(&self);
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| StateError::IoError(e))?;
        }

        let json =
            serde_json::to_string_pretty(&self.state).map_err(|e| StateError::SerdeError(e))?;
        async_fs::write(path, json)
            .await
            .map_err(|e| StateError::IoError(e))
    }

    pub async fn load(&mut self) -> Result<Self, StateError> {
        let path = Self::get_path(&self);
        let json = async_fs::read_to_string(path)
            .await
            .map_err(|e| StateError::IoError(e))?;
        let state: Vec<String> =
            serde_json::from_str(&json).map_err(|e| StateError::SerdeError(e))?;
        Ok(Self {
            state_type: self.state_type.clone(),
            state,
        })
    }

    pub fn has(&self, artist_id: String) -> bool {
        self.state.contains(&artist_id)
    }

    pub async fn clear(&mut self) -> Result<(), StateError> {
        let path = Self::get_path(&self);
        self.state.clear();
        async_fs::remove_file(path)
            .await
            .map_err(|e| StateError::IoError(e))
    }

    fn get_path(&self) -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(format!(
            "sporlcli/state/{state}.json",
            state = self.state_type
        ));
        path
    }
}
