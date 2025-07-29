use std::{io::Error, path::PathBuf};

use crate::types::Album;

#[derive(Debug)]
pub enum ReleaseError {
    IoError(Error),
    CriticalError(String),
    SerdeError(serde_json::Error),
}

impl From<Error> for ReleaseError {
    fn from(err: Error) -> Self {
        ReleaseError::IoError(err)
    }
}

pub struct ReleaseManager {
    artist_id: String,
    releases: Vec<Album>,
}

impl ReleaseManager {
    pub fn new(artist_id: String, releases: Option<Vec<Album>>) -> Self {
        Self {
            artist_id,
            releases: releases.unwrap_or(Vec::new()),
        }
    }

    pub async fn load_from_cache(&self) -> Result<Self, ReleaseError> {
        let path = Self::get_path(&self);
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| ReleaseError::IoError(e))?;
        let releases = serde_json::from_str(&content).map_err(|e| ReleaseError::SerdeError(e))?;
        Ok(Self {
            artist_id: self.artist_id.clone(),
            releases,
        })
    }

    pub async fn save_to_cache(&self) -> Result<(), ReleaseError> {
        let path = Self::get_path(&self);
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| ReleaseError::IoError(e))?;
        }

        let json = serde_json::to_string_pretty(&self.releases.clone())
            .map_err(|e| ReleaseError::SerdeError(e))?;
        async_fs::write(Self::get_path(&self), json)
            .await
            .map_err(|e| ReleaseError::IoError(e))
    }

    pub fn get_releases(&self) -> Vec<Album> {
        self.releases.clone()
    }

    fn get_path(&self) -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(format!(
            "sporlcli/cache/{artist_id}/releases.json",
            artist_id = self.artist_id.clone()
        ));
        path
    }
}

pub struct ReleaseWeekManager {
    week: u32,
    year: i32,
    releases: Vec<Album>,
}

impl ReleaseWeekManager {
    pub fn new(week: u32, year: i32, releases: Option<Vec<Album>>) -> Self {
        Self {
            week,
            year,
            releases: releases.unwrap_or(Vec::new()),
        }
    }

    pub async fn load_from_cache(&self) -> Result<Self, ReleaseError> {
        let path = Self::get_path(&self);
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| ReleaseError::IoError(e))?;

        let releases = serde_json::from_str(&content).map_err(|e| ReleaseError::SerdeError(e))?;
        Ok(Self {
            week: self.week,
            year: self.year,
            releases,
        })
    }

    pub async fn save_to_cache(&self) -> Result<(), ReleaseError> {
        let path = Self::get_path(&self);
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| ReleaseError::IoError(e))?;
        }

        let json = serde_json::to_string_pretty(&self.releases.clone())
            .map_err(|e| ReleaseError::SerdeError(e))?;
        async_fs::write(&path, json)
            .await
            .map_err(|e| ReleaseError::IoError(e))
    }

    pub async fn get_releases(&self) -> Result<Vec<Album>, String> {
        Ok(self.releases.clone())
    }

    fn get_path(&self) -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(format!(
            "sporlcli/releases/{year}/{week}/releases.json",
            year = self.year.clone(),
            week = self.week.clone(),
        ));
        path
    }
}
