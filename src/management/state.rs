use std::{io::Error, path::PathBuf};

/// State type identifier for tracking artist update status.
///
/// Used to identify state files that track which artists have been
/// processed during update operations. This helps prevent duplicate
/// processing and provides resume capability for interrupted operations.
pub const STATE_TYPE_ARTISTS: &str = "state_artists";

/// State type identifier for tracking release update status.
///
/// Used to identify state files that track which releases have been
/// processed during update operations. This ensures consistency and
/// allows for incremental updates of release data.
pub const STATE_TYPE_RELEASES: &str = "state_releases";

/// Error types that can occur during state management operations.
///
/// Provides comprehensive error handling for state persistence and retrieval,
/// covering I/O operations, serialization/deserialization, and critical
/// application errors. This allows for specific error handling based on
/// the type of failure encountered.
#[derive(Debug)]
pub enum StateError {
    /// File system I/O error (reading, writing, file operations)
    IoError(Error),
    /// Critical application error with descriptive message
    CriticalError(String),
    /// JSON serialization/deserialization error
    SerdeError(serde_json::Error),
}

impl From<Error> for StateError {
    /// Automatically converts standard I/O errors to StateError::IoError.
    ///
    /// Enables the use of the `?` operator when working with file operations
    /// that return `std::io::Error`, providing seamless error propagation.
    ///
    /// # Arguments
    ///
    /// * `err` - The I/O error to convert
    ///
    /// # Returns
    ///
    /// A `StateError::IoError` wrapping the original error.
    fn from(err: Error) -> Self {
        StateError::IoError(err)
    }
}

/// Manages application state with persistent storage for tracking operations.
///
/// Provides functionality to track the state of various operations (like artist
/// updates or release processing) by maintaining lists of processed items. This
/// enables resume capability, prevents duplicate processing, and provides audit
/// trails for long-running operations.
///
/// # State Types
///
/// The manager supports different state types for different operations:
/// - `STATE_TYPE_ARTISTS` - Tracks processed artists during updates
/// - `STATE_TYPE_RELEASES` - Tracks processed releases during updates
///
/// # Storage Format
///
/// State is stored as JSON arrays of strings, typically containing IDs of
/// processed items. This simple format allows for easy inspection and
/// modification if needed.
///
/// # File Organization
///
/// State files are stored in:
/// `{local_data_dir}/sporlcli/state/{state_type}.json`
///
/// # Use Cases
///
/// - **Resume Capability**: Continue interrupted update operations
/// - **Duplicate Prevention**: Skip already processed items
/// - **Progress Tracking**: Monitor completion of long operations
/// - **Audit Trail**: Keep record of processed items
/// - **Incremental Updates**: Process only new or changed items
pub struct StateManager {
    /// The type of state being managed (e.g., "state_artists")
    state_type: String,
    /// The current state as a vector of string identifiers
    state: Vec<String>,
}

impl StateManager {
    /// Creates a new StateManager for the specified state type.
    ///
    /// Initializes the manager with an empty state vector. The state type
    /// determines the filename used for persistent storage and helps organize
    /// different types of operational state.
    ///
    /// # Arguments
    ///
    /// * `state_type` - Identifier for the type of state (use the provided constants)
    ///
    /// # Returns
    ///
    /// A new `StateManager` instance ready for use.
    ///
    /// # Example
    ///
    /// ```
    /// use sporlcli::management::state::{StateManager, STATE_TYPE_ARTISTS};
    ///
    /// let mut manager = StateManager::new(STATE_TYPE_ARTISTS.to_string());
    /// manager.add("artist_id_123".to_string());
    /// manager.persist().await?;
    /// ```
    pub fn new(state_type: String) -> Self {
        Self {
            state_type,
            state: Vec::new(),
        }
    }

    /// Adds an item to the current state.
    ///
    /// Appends a string identifier to the state vector. This is typically
    /// used to mark an item as processed during operations. The item is
    /// added to memory but not automatically persisted.
    ///
    /// # Arguments
    ///
    /// * `item` - String identifier to add to the state (e.g., artist ID, release ID)
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = StateManager::new(STATE_TYPE_ARTISTS.to_string());
    /// manager.add("spotify_artist_id_123".to_string());
    /// manager.add("spotify_artist_id_456".to_string());
    ///
    /// // Remember to persist changes
    /// manager.persist().await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method does not check for duplicates. If duplicate prevention
    /// is needed, use the `has()` method to check before adding.
    pub fn add(&mut self, item: String) {
        self.state.push(item);
    }

    /// Returns a reference to the current state vector.
    ///
    /// Provides read-only access to the complete state without cloning.
    /// Useful for inspecting the current state, checking its size, or
    /// iterating over processed items.
    ///
    /// # Returns
    ///
    /// A reference to the vector of string identifiers in the current state.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = StateManager::new(STATE_TYPE_ARTISTS.to_string()).load().await?;
    /// let state = manager.get_state();
    ///
    /// println!("Processed {} items", state.len());
    /// for item in state {
    ///     println!("Processed: {}", item);
    /// }
    /// ```
    pub fn get_state(&self) -> &Vec<String> {
        &self.state
    }

    /// Persists the current state to the cache file.
    ///
    /// Serializes the current state vector to JSON and writes it to the
    /// appropriate state file. Creates the necessary directory structure
    /// if it doesn't exist. The data is formatted with pretty printing
    /// for better readability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - State successfully saved to file
    /// - `Err(StateError)` - Error indicating the save failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The state directory cannot be created
    /// - The state data cannot be serialized to JSON
    /// - The file cannot be written due to permissions or disk space issues
    /// - I/O errors occur during the write operation
    ///
    /// # File Location
    ///
    /// The state is saved to:
    /// `{local_data_dir}/sporlcli/state/{state_type}.json`
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = StateManager::new(STATE_TYPE_ARTISTS.to_string());
    /// manager.add("processed_artist_1".to_string());
    /// manager.add("processed_artist_2".to_string());
    ///
    /// manager.persist().await?;
    /// println!("State saved successfully");
    /// ```
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

    /// Loads state data from the cache file.
    ///
    /// Reads the cached JSON file and deserializes it into a new manager instance
    /// with the same state type. This replaces the current instance with one
    /// containing the loaded state data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(StateManager)` - New manager instance with loaded state data
    /// - `Err(StateError)` - Error indicating the load failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The state file doesn't exist for this state type
    /// - The file cannot be read due to permissions or I/O issues
    /// - The file content is not valid JSON
    /// - The JSON structure doesn't match the expected `Vec<String>` format
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = StateManager::new(STATE_TYPE_ARTISTS.to_string());
    /// let loaded_manager = manager.load().await?;
    ///
    /// println!("Loaded state with {} items", loaded_manager.get_state().len());
    /// ```
    ///
    /// # Note
    ///
    /// This method requires a mutable reference but returns a new instance.
    /// Consider using a pattern like `manager = manager.load().await?;`
    /// to replace the current instance with the loaded one.
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

    /// Checks if a specific item exists in the current state.
    ///
    /// Performs a linear search through the state vector to determine if
    /// the specified item has already been processed. This is useful for
    /// preventing duplicate processing and implementing conditional logic.
    ///
    /// # Arguments
    ///
    /// * `artist_id` - String identifier to search for in the state
    ///
    /// # Returns
    ///
    /// Returns `true` if the item exists in the state, `false` otherwise.
    ///
    /// # Performance Note
    ///
    /// This method performs a linear search, so performance will degrade
    /// with very large state vectors. For frequent lookups on large datasets,
    /// consider using a HashSet-based approach.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = StateManager::new(STATE_TYPE_ARTISTS.to_string()).load().await?;
    ///
    /// if !manager.has("artist_id_123".to_string()) {
    ///     // Process the artist
    ///     process_artist("artist_id_123").await?;
    ///     manager.add("artist_id_123".to_string());
    /// } else {
    ///     println!("Artist already processed, skipping");
    /// }
    /// ```
    ///
    /// # Parameter Name Note
    ///
    /// Despite the parameter name `artist_id`, this method works with any
    /// string identifier depending on the state type being managed.
    pub fn has(&self, artist_id: String) -> bool {
        self.state.contains(&artist_id)
    }

    /// Clears the current state and removes the state file.
    ///
    /// Empties the in-memory state vector and deletes the corresponding
    /// state file from disk. This is useful for resetting operations,
    /// starting fresh, or cleaning up after completed operations.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - State successfully cleared and file removed
    /// - `Err(StateError)` - Error indicating the clear operation failed
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The state file cannot be removed due to permissions
    /// - I/O errors occur during file deletion
    /// - The file doesn't exist (though this might be acceptable in some cases)
    ///
    /// # Behavior
    ///
    /// 1. Clears the in-memory state vector
    /// 2. Attempts to remove the state file from disk
    /// 3. Both operations must succeed for the method to return Ok(())
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = StateManager::new(STATE_TYPE_ARTISTS.to_string());
    /// // ... process items and add to state ...
    ///
    /// // Operation completed, clean up state
    /// manager.clear().await?;
    /// println!("State cleared and reset");
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Resetting interrupted operations
    /// - Cleaning up after successful completion
    /// - Starting fresh when operation logic changes
    /// - Removing stale state files
    pub async fn clear(&mut self) -> Result<(), StateError> {
        let path = Self::get_path(&self);
        self.state.clear();
        async_fs::remove_file(path)
            .await
            .map_err(|e| StateError::IoError(e))
    }

    /// Constructs the filesystem path for the state file.
    ///
    /// Builds the platform-specific path where state data should be stored
    /// based on the state type. Uses the system's local data directory as
    /// the base and creates a consistent file naming scheme.
    ///
    /// # Returns
    ///
    /// A `PathBuf` pointing to the state file location for this state type.
    ///
    /// # Path Structure
    ///
    /// The path follows the pattern:
    /// `{local_data_dir}/sporlcli/state/{state_type}.json`
    ///
    /// Example paths:
    /// - Linux: `~/.local/share/sporlcli/state/state_artists.json`
    /// - macOS: `~/Library/Application Support/sporlcli/state/state_artists.json`
    /// - Windows: `%LOCALAPPDATA%/sporlcli/state/state_artists.json`
    ///
    /// # State File Organization
    ///
    /// All state files are stored in the same directory but with different
    /// names based on their type, making it easy to:
    /// - Browse all state files
    /// - Clear all state for a fresh start
    /// - Back up or transfer state data
    /// - Debug state-related issues
    fn get_path(&self) -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(format!(
            "sporlcli/state/{state}.json",
            state = self.state_type
        ));
        path
    }
}
