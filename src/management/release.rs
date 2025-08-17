use std::{io::Error, path::PathBuf};

use crate::types::Album;

/// Error types that can occur during release management operations.
///
/// Provides comprehensive error handling for release week management,
/// covering I/O operations, serialization/deserialization, and critical
/// application errors. This enum allows for specific error handling
/// depending on the failure type.
#[derive(Debug)]
pub enum ReleaseError {
    /// File system I/O error (reading, writing, directory creation)
    IoError(Error),
    /// Critical application error with descriptive message
    CriticalError(String),
    /// JSON serialization/deserialization error
    SerdeError(serde_json::Error),
}

impl From<Error> for ReleaseError {
    /// Automatically converts standard I/O errors to ReleaseError::IoError.
    ///
    /// Enables the use of the `?` operator when working with file operations
    /// that return `std::io::Error`.
    ///
    /// # Arguments
    ///
    /// * `err` - The I/O error to convert
    ///
    /// # Returns
    ///
    /// A `ReleaseError::IoError` wrapping the original error.
    fn from(err: Error) -> Self {
        ReleaseError::IoError(err)
    }
}

/// Manages release data for a specific week and year with persistent caching.
///
/// Provides functionality to store and retrieve music release data organized
/// by week and year. Each week's releases are cached separately, allowing for
/// efficient access to historical release data and reducing API calls to Spotify.
///
/// # Cache Organization
///
/// Releases are stored in a hierarchical directory structure:
/// ```
/// ~/.local/share/sporlcli/releases/
/// ├── 2023/
/// │   ├── 1/releases.json
/// │   ├── 2/releases.json
/// │   └── ...
/// └── 2024/
///     ├── 1/releases.json
///     └── ...
/// ```
///
/// # Use Cases
///
/// - Caching weekly release data to avoid repeated API calls
/// - Historical tracking of release patterns
/// - Offline access to previously fetched release information
/// - Bulk operations on weekly release datasets
pub struct ReleaseWeekManager {
    /// The week number within the year (1-52/53)
    week: u32,
    /// The year for this release week
    year: i32,
    /// The albums/releases for this week
    releases: Vec<Album>,
}

impl ReleaseWeekManager {
    /// Creates a new ReleaseWeekManager for a specific week and year.
    ///
    /// Initializes the manager with the specified time period and optional
    /// initial release data. If no releases are provided, an empty vector
    /// is used as the starting point.
    ///
    /// # Arguments
    ///
    /// * `week` - Week number within the year (typically 1-52, sometimes 53)
    /// * `year` - The year for this release week
    /// * `releases` - Optional vector of existing releases for this week
    ///
    /// # Returns
    ///
    /// A new `ReleaseWeekManager` instance ready for use.
    ///
    /// # Week Numbering
    ///
    /// Week numbers should follow the application's week calculation logic,
    /// typically starting with week 1 being the first Saturday of January
    /// or the Saturday before/on January 1st.
    ///
    /// # Example
    ///
    /// ```
    /// // Create manager for week 42 of 2023 with no initial data
    /// let manager = ReleaseWeekManager::new(42, 2023, None);
    ///
    /// // Create manager with existing release data
    /// let releases = vec![/* album data */];
    /// let manager = ReleaseWeekManager::new(42, 2023, Some(releases));
    /// ```
    pub fn new(week: u32, year: i32, releases: Option<Vec<Album>>) -> Self {
        Self {
            week,
            year,
            releases: releases.unwrap_or(Vec::new()),
        }
    }

    /// Loads release data from the cache file for this week and year.
    ///
    /// Reads the cached JSON file containing release data for the specific
    /// week and year combination. Creates a new manager instance with the
    /// loaded data while preserving the original week and year parameters.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(ReleaseWeekManager)` - New manager instance with loaded release data
    /// - `Err(ReleaseError)` - Error indicating the load failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache file doesn't exist for this week/year combination
    /// - The file cannot be read due to permissions or I/O issues
    /// - The file content is not valid JSON
    /// - The JSON structure doesn't match the expected `Vec<Album>` format
    ///
    /// # Cache File Location
    ///
    /// The file is expected at:
    /// `{local_data_dir}/sporlcli/releases/{year}/{week}/releases.json`
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ReleaseWeekManager::new(42, 2023, None);
    /// let loaded_manager = manager.load_from_cache().await?;
    /// let releases = loaded_manager.get_releases().await?;
    /// println!("Loaded {} releases from cache", releases.len());
    /// ```
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

    /// Saves the current release data to the cache file.
    ///
    /// Serializes the current releases to JSON and writes them to the
    /// appropriate cache file. Creates the necessary directory structure
    /// if it doesn't exist. The data is formatted with pretty printing
    /// for better readability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - Data successfully saved to cache
    /// - `Err(ReleaseError)` - Error indicating the save failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache directory structure cannot be created
    /// - The release data cannot be serialized to JSON
    /// - The file cannot be written due to permissions or disk space issues
    /// - I/O errors occur during the write operation
    ///
    /// # Directory Creation
    ///
    /// The function automatically creates the full directory path:
    /// `{local_data_dir}/sporlcli/releases/{year}/{week}/`
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = ReleaseWeekManager::new(42, 2023, Some(releases));
    /// manager.save_to_cache().await?;
    /// println!("Releases cached for week 42, 2023");
    /// ```
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

    /// Returns a clone of all releases managed by this instance.
    ///
    /// Provides access to the complete release dataset for this week and year.
    /// Returns a cloned vector to prevent accidental modification of the
    /// internal state while allowing the caller full ownership of the data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Vec<Album>)` - All releases for this week and year
    /// - `Err(String)` - Error message (currently always succeeds)
    ///
    /// # Performance Note
    ///
    /// This method clones the entire releases vector. For large datasets,
    /// consider whether you need ownership or if a reference would suffice.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ReleaseWeekManager::load_from_cache().await?;
    /// let releases = manager.get_releases().await?;
    ///
    /// for album in releases {
    ///     println!("Album: {} by {}", album.name, album.artists[0].name);
    /// }
    /// ```
    ///
    /// # Future Enhancements
    ///
    /// The return type is a Result to allow for future error conditions,
    /// such as lazy loading or data validation failures.
    pub async fn get_releases(&self) -> Result<Vec<Album>, String> {
        Ok(self.releases.clone())
    }

    /// Constructs the filesystem path for the cache file.
    ///
    /// Builds the platform-specific path where release data should be stored
    /// based on the week and year. Uses the system's local data directory
    /// as the base and creates a hierarchical structure for organization.
    ///
    /// # Returns
    ///
    /// A `PathBuf` pointing to the cache file location for this week and year.
    ///
    /// # Path Structure
    ///
    /// The path follows the pattern:
    /// `{local_data_dir}/sporlcli/releases/{year}/{week}/releases.json`
    ///
    /// Example paths:
    /// - Linux: `~/.local/share/sporlcli/releases/2023/42/releases.json`
    /// - macOS: `~/Library/Application Support/sporlcli/releases/2023/42/releases.json`
    /// - Windows: `%LOCALAPPDATA%/sporlcli/releases/2023/42/releases.json`
    ///
    /// # Directory Hierarchy Benefits
    ///
    /// The year/week hierarchy provides:
    /// - Easy browsing of historical data
    /// - Efficient file system organization
    /// - Simple cleanup of old data by year
    /// - Clear separation between different time periods
    ///
    /// # Example
    ///
    /// ```
    /// let manager = ReleaseWeekManager::new(42, 2023, None);
    /// let path = manager.get_path();
    /// println!("Cache file: {}", path.display());
    /// ```
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
