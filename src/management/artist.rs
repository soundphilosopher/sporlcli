use crate::types::{Album, Artist, ArtistReleases};
use std::path::PathBuf;

pub struct ArtistReleaseManager {
    artist_releases: Option<Vec<ArtistReleases>>,
}

impl ArtistReleaseManager {
    pub fn new(artist_releases: Option<Vec<ArtistReleases>>) -> Self {
        Self {
            artist_releases: Some(artist_releases.unwrap_or(Vec::new())),
        }
    }

    pub async fn load() -> Result<Self, String> {
        let path = Self::cache_path();
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let artist_releases: Vec<ArtistReleases> =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Self {
            artist_releases: Some(artist_releases),
        })
    }

    pub async fn persist(&self) -> Result<(), String> {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(&self.artist_releases.clone())
            .map_err(|e| e.to_string())?;
        async_fs::write(Self::cache_path(), json)
            .await
            .map_err(|e| e.to_string())
    }

    pub fn add_artist(&mut self, artist: Artist) -> &mut Self {
        if let Some(ars) = &mut self.artist_releases {
            ars.push(ArtistReleases {
                artist,
                releases: Vec::new(),
            });
        }
        self
    }

    pub fn add_artists(&mut self, artists: Vec<Artist>) -> &mut Self {
        if let Some(ars) = &mut self.artist_releases {
            for artist in artists {
                ars.push(ArtistReleases {
                    artist,
                    releases: Vec::new(),
                });
            }
        }
        self
    }

    pub fn add_releases_to_artist(&mut self, artist_id: &str, releases: Vec<Album>) -> &mut Self {
        if let Some(ars) = &mut self.artist_releases {
            if let Some(ar) = ars.iter_mut().find(|ar| ar.artist.id == artist_id) {
                // because the Spotify API doesn't have any possibility to get release by release date for an artist we need to clear all previous releases
                ar.releases.clear();
                ar.releases.extend(releases);
            }
        }
        self
    }

    pub fn get_releases_for_artist(&self, artist_id: &str) -> Option<Vec<Album>> {
        self.artist_releases.as_ref().and_then(|ar| {
            ar.iter()
                .find(|ar| ar.artist.id == artist_id)
                .map(|ar| ar.releases.clone())
        })
    }

    pub fn get_all_artists(&self) -> Option<Vec<Artist>> {
        self.artist_releases
            .as_ref()
            .map(|ar| ar.iter().map(|ar| ar.artist.clone()).collect())
    }

    pub fn count_artists(&self) -> usize {
        self.artist_releases.as_ref().map_or(0, |ar| ar.len())
    }

    pub fn count_releases(&self) -> usize {
        self.artist_releases
            .as_ref()
            .map_or(0, |ar| ar.iter().map(|ar| ar.releases.len()).sum())
    }

    pub fn all(&self) -> Option<Vec<ArtistReleases>> {
        self.artist_releases.clone()
    }

    fn cache_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sporlcli/cache/artist-releases.json");
        path
    }
}
