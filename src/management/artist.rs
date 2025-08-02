use crate::types::Artist;
use std::path::PathBuf;

pub struct ArtistsManager {
    artists: Vec<Artist>,
}

impl ArtistsManager {
    pub fn new(artists: Vec<Artist>) -> Self {
        ArtistsManager { artists }
    }

    pub async fn load() -> Result<Self, String> {
        let path = Self::cache_path();
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let artists: Vec<Artist> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(Self { artists });
    }

    pub async fn persist(&self) -> Result<(), String> {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        let json =
            serde_json::to_string_pretty(&self.artists.clone()).map_err(|e| e.to_string())?;
        async_fs::write(Self::cache_path(), json)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn add(&mut self, artists: Vec<Artist>) -> Result<(), String> {
        self.artists.extend(artists);
        self.persist().await
    }

    pub fn get_artists(&self) -> Vec<Artist> {
        return self.artists.clone();
    }

    pub fn count(&self) -> u64 {
        return self.artists.len() as u64;
    }

    fn cache_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sporlcli/cache/artists.json");
        path
    }
}
