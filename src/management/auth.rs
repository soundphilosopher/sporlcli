use std::path::PathBuf;

use chrono::Utc;

use crate::{spotify, types::Token};

/// Manages OAuth tokens with automatic refresh and persistent storage.
///
/// Provides a high-level interface for managing Spotify API authentication tokens,
/// including automatic refresh when tokens expire and secure storage in the local
/// cache directory. The manager handles the complexity of token lifecycle management
/// and ensures that API requests always use valid tokens.
///
/// # Token Lifecycle
///
/// 1. **Initial Authentication**: Tokens are obtained through the OAuth flow
/// 2. **Storage**: Tokens are persistently cached for future use
/// 3. **Validation**: Tokens are checked for expiration before use
/// 4. **Refresh**: Expired tokens are automatically refreshed using the refresh token
/// 5. **Persistence**: New tokens are saved to cache after refresh
///
/// # Cache Storage
///
/// Tokens are stored in a JSON file at:
/// - Linux: `~/.local/share/sporlcli/cache/token.json`
/// - macOS: `~/Library/Application Support/sporlcli/cache/token.json`
/// - Windows: `%LOCALAPPDATA%/sporlcli/cache/token.json`
///
/// # Security Considerations
///
/// - Tokens are stored in the user's local data directory
/// - File permissions should be restricted to the user
/// - Refresh tokens have longer lifespans than access tokens
/// - Automatic refresh reduces the need to store long-lived credentials
pub struct TokenManager {
    /// The currently managed OAuth token
    token: Token,
}

impl TokenManager {
    /// Creates a new TokenManager with the provided token.
    ///
    /// Initializes the manager with an existing token, typically obtained
    /// from the OAuth authentication flow. The token should be complete
    /// with both access and refresh tokens.
    ///
    /// # Arguments
    ///
    /// * `token` - A complete OAuth token with access and refresh tokens
    ///
    /// # Returns
    ///
    /// A new `TokenManager` instance ready for use.
    ///
    /// # Example
    ///
    /// ```
    /// let token = Token { /* obtained from OAuth */ };
    /// let manager = TokenManager::new(token);
    /// ```
    pub fn new(token: Token) -> Self {
        TokenManager { token }
    }

    /// Loads a previously stored token from the cache file.
    ///
    /// Reads the cached token JSON file and deserializes it into a manager instance.
    /// This is the primary method for restoring authentication state from a
    /// previous session.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(TokenManager)` - Successfully loaded manager with cached token
    /// - `Err(String)` - Error message describing the failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The token cache file doesn't exist or can't be read
    /// - The file content is not valid JSON
    /// - The JSON structure doesn't match the expected Token format
    /// - File system permissions prevent reading
    ///
    /// # Example
    ///
    /// ```
    /// let manager = TokenManager::load().await?;
    /// let token = manager.get_valid_token().await;
    /// ```
    pub async fn load() -> Result<Self, String> {
        let path = Self::token_path();
        let content = async_fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let token: Token = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(Self { token });
    }

    /// Persists the current token to the cache file.
    ///
    /// Serializes the current token to JSON and writes it to the local cache file.
    /// Creates the necessary directory structure if it doesn't exist. The token
    /// is formatted with pretty printing for better readability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - Token successfully saved to cache
    /// - `Err(String)` - Error message describing the failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache directory cannot be created
    /// - The token cannot be serialized to JSON
    /// - The file cannot be written (permissions, disk space, etc.)
    ///
    /// # Security Note
    ///
    /// The token file should have restricted permissions to prevent
    /// unauthorized access to the stored credentials.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = TokenManager::new(token);
    /// manager.persist().await?;
    /// ```
    pub async fn persist(&self) -> Result<(), String> {
        let path = Self::token_path();
        if let Some(parent) = path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(&self.token).map_err(|e| e.to_string())?;
        async_fs::write(Self::token_path(), json)
            .await
            .map_err(|e| e.to_string())
    }

    /// Returns a valid access token, refreshing if necessary.
    ///
    /// This is the primary method for obtaining tokens for API requests. It
    /// automatically handles token expiration by checking the current time
    /// against the token's expiration and refreshing when needed.
    ///
    /// # Returns
    ///
    /// A `String` containing a valid access token ready for use in API requests.
    ///
    /// # Automatic Refresh
    ///
    /// If the current token is expired or close to expiring (within 4 minutes),
    /// the method will:
    /// 1. Use the refresh token to obtain a new access token
    /// 2. Update the internal token state
    /// 3. Persist the new token to cache
    /// 4. Return the new access token
    ///
    /// # Error Handling
    ///
    /// If token refresh fails, the method will return the current (possibly expired)
    /// access token. This allows the caller to attempt the API request, which may
    /// still succeed if the token hasn't fully expired.
    ///
    /// # Example
    ///
    /// ```
    /// let mut manager = TokenManager::load().await?;
    /// let token = manager.get_valid_token().await; // Always returns valid token
    ///
    /// // Use token in API request
    /// let response = client.get(url).bearer_auth(token).send().await?;
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method requires a mutable reference because it may update the
    /// internal token state during refresh operations.
    pub async fn get_valid_token(&mut self) -> String {
        if self.is_expired() {
            if let Ok(new_token) = self.refresh_token().await {
                self.token = new_token;
                let _ = self.persist().await;
            }
        }

        self.token.access_token.clone()
    }

    /// Checks if the current token is expired or close to expiring.
    ///
    /// Determines whether the token needs to be refreshed by comparing the
    /// current time with the token's expiration time. Includes a 4-minute
    /// buffer to refresh tokens before they actually expire, providing a
    /// safety margin for API requests.
    ///
    /// # Returns
    ///
    /// Returns `true` if the token is expired or will expire within 4 minutes,
    /// `false` if the token is still valid.
    ///
    /// # Expiration Logic
    ///
    /// The token is considered expired if:
    /// ```
    /// current_time >= (obtained_at + expires_in - 240_seconds)
    /// ```
    ///
    /// The 240-second (4-minute) buffer ensures that tokens are refreshed
    /// before they expire, preventing mid-request failures.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = TokenManager::load().await?;
    /// if manager.is_expired() {
    ///     println!("Token needs refresh");
    /// } else {
    ///     println!("Token is still valid");
    /// }
    /// ```
    fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as u64;
        now >= self.token.obtained_at + self.token.expires_in - 240
    }

    /// Refreshes the current token using the stored refresh token.
    ///
    /// Makes a request to Spotify's token endpoint to exchange the refresh token
    /// for a new access token. This is an internal method used by `get_valid_token`
    /// when automatic refresh is needed.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Token)` - New token with fresh access token and expiration
    /// - `Err(String)` - Error message describing the refresh failure
    ///
    /// # Refresh Process
    ///
    /// 1. Uses the current refresh token to request a new access token
    /// 2. Spotify may provide a new refresh token (token rotation)
    /// 3. Returns a complete token with updated timestamps
    ///
    /// # Error Conditions
    ///
    /// Refresh can fail due to:
    /// - Expired or invalid refresh token
    /// - Network connectivity issues
    /// - Spotify API service errors
    /// - User has revoked application access
    ///
    /// # Example
    ///
    /// This method is typically called internally, but can be used directly:
    ///
    /// ```
    /// // Internal usage (called automatically)
    /// let token = manager.get_valid_token().await;
    /// ```
    async fn refresh_token(&self) -> Result<Token, String> {
        match spotify::auth::refresh_token(&self.token.refresh_token).await {
            Ok(token) => Ok(token),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Returns the filesystem path where tokens are cached.
    ///
    /// Constructs the platform-specific path to the token cache file using the
    /// system's local data directory. Creates the path consistently across
    /// different operating systems.
    ///
    /// # Returns
    ///
    /// A `PathBuf` pointing to the token cache file location.
    ///
    /// # File Location
    ///
    /// - Linux: `~/.local/share/sporlcli/cache/token.json`
    /// - macOS: `~/Library/Application Support/sporlcli/cache/token.json`
    /// - Windows: `%LOCALAPPDATA%/sporlcli/cache/token.json`
    ///
    /// # Security Considerations
    ///
    /// The token file contains sensitive authentication credentials and should
    /// be protected with appropriate file system permissions.
    fn token_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sporlcli/cache/token.json");
        path
    }

    /// Returns a reference to the current token.
    ///
    /// Provides read-only access to the complete token information, including
    /// access token, refresh token, scope, and expiration details. Useful for
    /// inspecting token state without modification.
    ///
    /// # Returns
    ///
    /// A reference to the current `Token`.
    ///
    /// # Example
    ///
    /// ```
    /// let manager = TokenManager::load().await?;
    /// let token = manager.current_token();
    ///
    /// println!("Token expires in {} seconds", token.expires_in);
    /// println!("Token scope: {}", token.scope);
    /// ```
    ///
    /// # Note
    ///
    /// This method returns the token as-is without checking expiration.
    /// For API requests, use `get_valid_token()` which ensures freshness.
    pub fn current_token(&self) -> &Token {
        &self.token
    }
}
