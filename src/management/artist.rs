use crate::types::{Album, Artist, ArtistReleases};
use std::path::PathBuf;

/// Manages artist data and their associated releases with persistent caching.
///
/// Provides a comprehensive interface for managing artists and their releases,
/// including loading from and saving to a local cache file. This manager handles
/// the relationship between artists and their albums, allowing for efficient
/// storage and retrieval of release data.
///
/// # Cache Storage
///
/// Data is stored in a JSON file at:
/// - Linux: `~/.local/share/sporlcli/cache/artist-releases.json`
/// - macOS: `~/Library/Application Support/sporlcli/cache/artist-releases.json`
/// - Windows: `%LOCALAPPDATA%/sporlcli/cache/artist-releases.json`
///
/// # Data Structure
///
/// The manager maintains a vector of `ArtistReleases` objects, each containing
/// an artist and their associated albums. This structure allows for efficient
/// querying and updating of artist-specific release data.
pub struct ArtistReleaseManager {
    /// Optional vector of artist-release pairs
    artist_releases: Option<Vec<ArtistReleases>>,
}

impl ArtistReleaseManager {
    /// Creates a new ArtistReleaseManager with optional initial data.
    ///
    /// Initializes the manager with provided artist-release data or an empty
    /// vector if None is provided. This constructor is useful when creating
    /// a manager with existing data or starting fresh.
    ///
    /// # Arguments
    ///
    /// * `artist_releases` - Optional vector of existing artist-release data
    ///
    /// # Returns
    ///
    /// A new `ArtistReleaseManager` instance ready for use.
    ///
    /// # Example
    ///
    /// ```
    /// // Create empty manager
    /// let manager = ArtistReleaseManager::new(None);
    ///
    /// // Create manager with existing data
    /// let existing_data = vec![/* artist releases */];
    /// let manager = ArtistReleaseManager::new(Some(existing_data));
    /// ```
    pub fn new(artist_releases: Option<Vec<ArtistReleases>>) -> Self {
        Self {
            artist_releases: Some(artist_releases.unwrap_or(Vec::new())),
        }
    }

    /// Loads artist-release data from the local cache file.
    ///
    /// Reads the cached JSON file and deserializes it into a manager instance.
    /// This is the primary method for retrieving previously stored artist and
    /// release data from persistent storage.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(ArtistReleaseManager)` - Successfully loaded manager with cached data
    /// - `Err(String)` - Error message describing the failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache file cannot be read (doesn't exist, permission issues, etc.)
    /// - The file content is not valid JSON
    /// - The JSON structure doesn't match the expected format
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// println!("Loaded {} artists from cache", manager.count_artists());
    /// ```
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

    /// Persists the current artist-release data to the cache file.
    ///
    /// Serializes the current state to JSON and writes it to the local cache file.
    /// Creates the necessary directory structure if it doesn't exist. The data
    /// is formatted with pretty printing for better readability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - Data successfully saved to cache
    /// - `Err(String)` - Error message describing the failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache directory cannot be created
    /// - The data cannot be serialized to JSON
    /// - The file cannot be written (permission issues, disk space, etc.)
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = ArtistReleaseManager::new(None);
    /// manager.add_artist(artist);
    /// manager.persist().await?;
    /// ```
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

    /// Adds a single artist to the manager with an empty releases list.
    ///
    /// Creates a new `ArtistReleases` entry for the artist with no associated
    /// releases initially. Releases can be added later using the
    /// `add_releases_to_artist` method.
    ///
    /// # Arguments
    ///
    /// * `artist` - The artist to add to the manager
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = ArtistReleaseManager::new(None);
    /// manager.add_artist(artist).persist().await?;
    /// ```
    pub fn add_artist(&mut self, artist: Artist) -> &mut Self {
        if let Some(ars) = &mut self.artist_releases {
            ars.push(ArtistReleases {
                artist,
                releases: Vec::new(),
            });
        }
        self
    }

    /// Adds multiple artists to the manager with empty releases lists.
    ///
    /// Efficiently adds a batch of artists, each with an initially empty
    /// releases list. This is more efficient than calling `add_artist`
    /// multiple times when adding many artists at once.
    ///
    /// # Arguments
    ///
    /// * `artists` - Vector of artists to add to the manager
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = ArtistReleaseManager::new(None);
    /// let artists = vec![artist1, artist2, artist3];
    /// manager.add_artists(artists).persist().await?;
    /// ```
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

    /// Adds or replaces releases for a specific artist.
    ///
    /// Updates the releases list for an artist identified by their Spotify ID.
    /// **Important**: This method clears all existing releases for the artist
    /// before adding the new ones, as the Spotify API doesn't provide
    /// incremental release updates by date.
    ///
    /// # Arguments
    ///
    /// * `artist_id` - Spotify ID of the artist to update
    /// * `releases` - Vector of albums/releases to associate with the artist
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Behavior
    ///
    /// - Finds the artist by ID
    /// - Clears their existing releases list
    /// - Adds all provided releases
    /// - If artist is not found, no action is taken
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = ArtistReleaseManager::load().await?;
    /// let releases = vec![album1, album2, album3];
    /// manager.add_releases_to_artist("artist_spotify_id", releases);
    /// manager.persist().await?;
    /// ```
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

    /// Retrieves all releases for a specific artist.
    ///
    /// Returns a cloned vector of all albums associated with the specified artist.
    /// Returns None if the artist is not found in the manager.
    ///
    /// # Arguments
    ///
    /// * `artist_id` - Spotify ID of the artist to get releases for
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<Album>)` if the artist is found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// if let Some(releases) = manager.get_releases_for_artist("artist_id") {
    ///     println!("Artist has {} releases", releases.len());
    /// }
    /// ```
    pub fn get_releases_for_artist(&self, artist_id: &str) -> Option<Vec<Album>> {
        self.artist_releases.as_ref().and_then(|ar| {
            ar.iter()
                .find(|ar| ar.artist.id == artist_id)
                .map(|ar| ar.releases.clone())
        })
    }

    /// Retrieves all artists managed by this instance.
    ///
    /// Returns a cloned vector of all artists in the manager, regardless
    /// of whether they have associated releases. Useful for getting a
    /// complete list of followed artists.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<Artist>)` if there are artists, `None` if empty or uninitialized.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// if let Some(artists) = manager.get_all_artists() {
    ///     for artist in artists {
    ///         println!("Artist: {}", artist.name);
    ///     }
    /// }
    /// ```
    pub fn get_all_artists(&self) -> Option<Vec<Artist>> {
        self.artist_releases
            .as_ref()
            .map(|ar| ar.iter().map(|ar| ar.artist.clone()).collect())
    }

    /// Returns the total number of artists in the manager.
    ///
    /// Provides a quick count of all artists, whether they have releases or not.
    /// Returns 0 if the manager is empty or uninitialized.
    ///
    /// # Returns
    ///
    /// The number of artists as a `usize`.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// println!("Managing {} artists", manager.count_artists());
    /// ```
    pub fn count_artists(&self) -> usize {
        self.artist_releases.as_ref().map_or(0, |ar| ar.len())
    }

    /// Returns the total number of releases across all artists.
    ///
    /// Calculates the sum of all releases for all artists in the manager.
    /// This provides insight into the total volume of release data being managed.
    ///
    /// # Returns
    ///
    /// The total number of releases as a `usize`.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// println!("Managing {} total releases", manager.count_releases());
    /// ```
    pub fn count_releases(&self) -> usize {
        self.artist_releases
            .as_ref()
            .map_or(0, |ar| ar.iter().map(|ar| ar.releases.len()).sum())
    }

    /// Returns a clone of all artist-release data.
    ///
    /// Provides access to the complete dataset managed by this instance.
    /// Returns None if the manager is uninitialized.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<ArtistReleases>)` containing all data, or `None` if empty.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ArtistReleaseManager::load().await?;
    /// if let Some(all_data) = manager.all() {
    ///     for artist_releases in all_data {
    ///         println!("{} has {} releases",
    ///                  artist_releases.artist.name,
    ///                  artist_releases.releases.len());
    ///     }
    /// }
    /// ```
    pub fn all(&self) -> Option<Vec<ArtistReleases>> {
        self.artist_releases.clone()
    }

    /// Returns the filesystem path where artist-release data is cached.
    ///
    /// Constructs the platform-specific path to the cache file using the
    /// system's local data directory. Creates the path consistently across
    /// different operating systems.
    ///
    /// # Returns
    ///
    /// A `PathBuf` pointing to the cache file location.
    ///
    /// # File Location
    ///
    /// - Linux: `~/.local/share/sporlcli/cache/artist-releases.json`
    /// - macOS: `~/Library/Application Support/sporlcli/cache/artist-releases.json`
    /// - Windows: `%LOCALAPPDATA%/sporlcli/cache/artist-releases.json`
    fn cache_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sporlcli/cache/artist-releases.json");
        path
    }
}
