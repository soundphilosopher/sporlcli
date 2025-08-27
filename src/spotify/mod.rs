//! # Spotify Integration Module
//!
//! This module provides a comprehensive interface to the Spotify Web API, implementing
//! authentication, data retrieval, and playlist management functionality. It serves as
//! the primary integration layer between Sporlcli and Spotify's services, handling all
//! HTTP communication, authentication flows, error handling, and rate limiting.
//!
//! ## Overview
//!
//! The Spotify module implements a complete SDK-like interface for Spotify Web API
//! operations required by Sporlcli. It abstracts away the complexities of HTTP requests,
//! OAuth flows, and API quirks, providing a clean Rust interface for higher-level
//! application logic.
//!
//! ## Architecture
//!
//! The module follows a feature-based organization where each submodule handles a
//! specific domain of Spotify API functionality:
//!
//! ```text
//! Application Layer (CLI, Management)
//!          ↓
//! Spotify Integration Layer
//!     ├── Authentication (OAuth 2.0 PKCE)
//!     ├── Artist Operations (Following, Metadata)
//!     ├── Release Management (Albums, Singles)
//!     └── Playlist Operations (Create, Modify)
//!          ↓
//! HTTP Layer (reqwest, JSON)
//!          ↓
//! Spotify Web API
//! ```
//!
//! ## Core Modules
//!
//! ### Authentication Module
//!
//! [`auth`] - Implements OAuth 2.0 PKCE (Proof Key for Code Exchange) flow:
//! - **Complete Auth Flow**: Handles the full OAuth process from initial request to token storage
//! - **PKCE Security**: Implements cryptographically secure authentication without client secrets
//! - **Token Management**: Automatic token refresh and expiration handling
//! - **Browser Integration**: Automatic browser launch for user authorization
//! - **Local Callback Server**: Temporary HTTP server for receiving OAuth callbacks
//!
//! ### Artist Management Module
//!
//! [`artists`] - Handles artist-related API operations:
//! - **Followed Artists**: Retrieval of user's followed artists with pagination support
//! - **Artist Counting**: Efficient total count queries without full data transfer
//! - **Cursor Pagination**: Handles Spotify's cursor-based pagination system
//! - **Rate Limiting**: Intelligent retry logic for API rate limits
//!
//! ### Release Management Module
//!
//! [`releases`] - Manages album and release data retrieval:
//! - **Artist Releases**: Fetches albums, singles, and other releases by artist
//! - **Release Type Filtering**: Supports filtering by album type (album, single, compilation)
//! - **Batch Operations**: Efficient multi-album detail retrieval
//! - **Market Availability**: Handles geographic restrictions and availability
//!
//! ### Playlist Management Module
//!
//! [`playlist`] - Provides playlist creation and modification capabilities:
//! - **Playlist Creation**: Creates private playlists with automatic descriptions
//! - **Duplicate Detection**: Checks for existing playlists before creation
//! - **Track Management**: Adds tracks to playlists in batches
//! - **Playlist Ownership**: Handles user-owned and collaborative playlists
//!
//! ## Authentication Strategy
//!
//! The module implements OAuth 2.0 with PKCE for secure authentication:
//!
//! ### PKCE Flow Benefits
//! - **No Client Secrets**: Eliminates the need to store sensitive client credentials
//! - **Code Interception Protection**: Prevents authorization code interception attacks
//! - **Mobile/Desktop Safe**: Designed for applications that cannot securely store secrets
//! - **Modern Security**: Follows current OAuth 2.0 security best practices
//!
//! ### Flow Implementation
//! 1. **Code Verifier Generation**: Creates cryptographically random verifier
//! 2. **Challenge Creation**: Derives SHA256 challenge from verifier
//! 3. **Authorization Request**: Directs user to Spotify with challenge
//! 4. **Local Callback**: Receives authorization code via temporary HTTP server
//! 5. **Token Exchange**: Exchanges code + verifier for access token
//! 6. **Token Storage**: Securely stores tokens for future use
//!
//! ## Error Handling Philosophy
//!
//! The module implements comprehensive error handling strategies:
//!
//! ### Rate Limiting
//! - **Automatic Retry**: Handles 429 Too Many Requests with appropriate delays
//! - **Retry-After Headers**: Respects Spotify's recommended retry timing
//! - **Exponential Backoff**: Implements intelligent delay strategies
//! - **Rate Limit Warnings**: Provides user feedback for excessive delays
//!
//! ### Network Resilience
//! - **Connection Failures**: Graceful handling of network connectivity issues
//! - **Timeout Management**: Appropriate timeouts for different operation types
//! - **Service Errors**: Specific handling of Spotify API service errors
//! - **Retry Logic**: Automatic retry for transient failures (502 Bad Gateway)
//!
//! ### Authentication Errors
//! - **Token Expiration**: Automatic token refresh using refresh tokens
//! - **Invalid Credentials**: Clear error messages directing to re-authentication
//! - **Scope Issues**: Handles insufficient permission errors
//! - **User Revocation**: Graceful handling of revoked application access
//!
//! ## Performance Optimization
//!
//! Several strategies are employed for optimal performance:
//!
//! ### Batch Processing
//! - **Multi-Album Requests**: Fetches up to 20 albums in a single API call
//! - **Pagination Efficiency**: Uses cursor-based pagination for large datasets
//! - **Selective Data**: Requests only necessary fields to reduce transfer time
//!
//! ### Request Optimization
//! - **HTTP Keep-Alive**: Reuses connections where possible
//! - **Compression**: Leverages HTTP compression for large responses
//! - **Minimal Requests**: Combines operations to reduce API call volume
//!
//! ### Token Management
//! - **Proactive Refresh**: Refreshes tokens before expiration (4-minute buffer)
//! - **Token Caching**: Avoids repeated authentication flows
//! - **Automatic Lifecycle**: Handles token refresh transparently
//!
//! ## API Coverage
//!
//! The module covers the following Spotify Web API endpoints:
//!
//! ### User Data
//! - `GET /me/following` - User's followed artists with pagination
//! - `GET /me/playlists` - User's playlists for duplicate checking
//!
//! ### Artist Information
//! - `GET /artists/{id}/albums` - Artist's discography with filtering
//!
//! ### Album Details
//! - `GET /albums` - Batch album information with track listings
//!
//! ### Playlist Operations
//! - `POST /users/{user_id}/playlists` - Create new playlists
//! - `POST /playlists/{playlist_id}/tracks` - Add tracks to playlists
//!
//! ### Authentication
//! - `POST /api/token` - Token exchange and refresh operations
//!
//! ## Configuration Integration
//!
//! The module integrates with the application's configuration system for:
//! - **API Endpoints**: Base URLs for different Spotify API services
//! - **Authentication**: Client ID, redirect URI, and scope configuration
//! - **User Settings**: Default user ID for playlist operations
//! - **Server Settings**: Local callback server configuration
//!
//! ## Usage Patterns
//!
//! ### Authentication Flow
//! ```rust
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! let shared_state = Arc::new(Mutex::new(None));
//! spotify::auth::auth(shared_state).await;
//! // User is now authenticated and tokens are stored
//! ```
//!
//! ### Data Retrieval
//! ```rust
//! // Get followed artists
//! let (artists, cursor) = spotify::artists::get_artist(&token, 20, None).await?;
//!
//! // Get artist releases
//! let releases = spotify::releases::get_release_for_artist(
//!     artist_id,
//!     &token,
//!     50,
//!     &release_types
//! ).await?;
//! ```
//!
//! ### Playlist Management
//! ```rust
//! // Create playlist
//! let playlist = spotify::playlist::create("My Playlist".to_string()).await?;
//!
//! // Add tracks
//! spotify::playlist::add_tracks(playlist.id, tracks).await?;
//! ```
//!
//! ## Error Types
//!
//! All functions return `Result` types with specific error handling:
//! - **`reqwest::Error`** - HTTP client errors, network issues, API errors
//! - **`String`** - Authentication and token management errors
//!
//! ## Thread Safety
//!
//! The module is designed for async single-threaded use:
//! - All operations use async/await for non-blocking I/O
//! - Shared state uses Arc<Mutex<>> for safe concurrent access
//! - No global mutable state or unsafe operations
//!
//! ## Future Extensibility
//!
//! The module architecture supports future enhancements:
//! - Additional API endpoints can be added as new functions
//! - New authentication flows can be implemented alongside PKCE
//! - Error handling can be enhanced with more specific error types
//! - Caching layers can be added for frequently accessed data
//!
//! ## Dependencies
//!
//! The module relies on several external crates:
//! - **reqwest** - HTTP client with JSON support and async capabilities
//! - **serde_json** - JSON serialization and deserialization
//! - **chrono** - Date and time handling for token expiration
//! - **tokio** - Async runtime and utilities
//!
//! ## Security Considerations
//!
//! - **No Secrets Storage**: Client secrets are not stored or transmitted
//! - **Token Security**: Access tokens are stored securely in local directories
//! - **HTTPS Only**: All API communication uses HTTPS
//! - **Limited Scope**: Requests only necessary permissions from users

pub mod artists;
pub mod auth;
pub mod playlist;
pub mod releases;
