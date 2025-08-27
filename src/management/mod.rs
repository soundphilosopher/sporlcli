//! # Management Module
//!
//! This module provides comprehensive data management and caching functionality for
//! Sporlcli, implementing persistent storage, state tracking, and lifecycle management
//! for various data types including artists, releases, authentication tokens, and
//! operation state. It serves as the data layer between the CLI interface and the
//! underlying Spotify API integration.
//!
//! ## Overview
//!
//! The management module implements a sophisticated caching and state management
//! system that enables offline operation, resume capability for interrupted operations,
//! and efficient data organization. It provides managers for different data types,
//! each with specialized functionality for their respective domains.
//!
//! ## Architecture
//!
//! The module follows a manager-based architecture where each manager handles a
//! specific type of data or operation:
//!
//! ```text
//! CLI Commands
//!     ↓
//! Management Layer
//!     ├── ArtistReleaseManager (Artist + Release associations)
//!     ├── TokenManager (OAuth token lifecycle)
//!     ├── ReleaseWeekManager (Weekly release organization)
//!     └── StateManager (Operation state tracking)
//!     ↓
//! File System Cache (JSON storage)
//! ```
//!
//! ## Core Managers
//!
//! ### Artist Management
//!
//! [`ArtistReleaseManager`] - Manages the relationship between followed artists
//! and their releases, providing:
//! - Persistent storage of artist metadata and release associations
//! - Efficient querying of artist-specific release data
//! - Bulk operations for artist and release management
//! - Cache synchronization with Spotify's followed artists
//!
//! ### Authentication Management
//!
//! [`TokenManager`] - Handles OAuth token lifecycle with features including:
//! - Automatic token refresh when approaching expiration
//! - Secure token storage in local cache directory
//! - Token validation and expiration checking
//! - Seamless integration with API authentication requirements
//!
//! ### Release Organization
//!
//! [`ReleaseWeekManager`] - Organizes releases by week and year for efficient access:
//! - Week-based release caching for historical data access
//! - Hierarchical storage organization (year/week/releases.json)
//! - Support for custom week numbering systems
//! - Efficient querying of releases within specific time ranges
//!
//! ### Operation State Tracking
//!
//! [`StateManager`] - Provides operation state persistence with capabilities for:
//! - Resume capability for interrupted long-running operations
//! - Duplicate prevention during batch processing
//! - Progress tracking and audit trails
//! - Multiple state types for different operation categories
//!
//! ## Data Storage Strategy
//!
//! The management layer implements a comprehensive caching strategy using JSON
//! files stored in platform-specific directories:
//!
//! ### Cache Organization
//!
//! ```text
//! ~/.local/share/sporlcli/  (Linux)
//! ~/Library/Application Support/sporlcli/  (macOS)
//! %LOCALAPPDATA%/sporlcli/  (Windows)
//! ├── cache/
//! │   ├── artist-releases.json     # Artist-release associations
//! │   └── token.json               # OAuth tokens
//! ├── releases/
//! │   ├── 2023/
//! │   │   ├── 1/releases.json      # Week 1, 2023 releases
//! │   │   ├── 2/releases.json      # Week 2, 2023 releases
//! │   │   └── ...
//! │   └── 2024/
//! │       └── 1/releases.json
//! └── state/
//!     ├── state_artists.json       # Artist processing state
//!     └── state_releases.json      # Release processing state
//! ```
//!
//! ### Storage Benefits
//!
//! - **Offline Access**: Previously fetched data available without network
//! - **Performance**: Local queries are much faster than API calls
//! - **Reliability**: Operations can continue despite network issues
//! - **Organization**: Hierarchical structure for easy data management
//! - **Inspection**: JSON format allows manual inspection and debugging
//!
//! ## Error Handling Philosophy
//!
//! Each manager implements robust error handling with specific error types:
//!
//! - **Graceful Degradation**: Partial failures don't prevent useful operations
//! - **Detailed Error Context**: Error messages include actionable information
//! - **Type-Specific Errors**: Different error types for different failure modes
//! - **Recovery Guidance**: Errors suggest specific remediation steps
//!
//! ## State Management Patterns
//!
//! The module implements several patterns for managing operational state:
//!
//! ### Resume Capability
//! ```rust
//! // Load existing state or create new
//! let mut state = StateManager::new(STATE_TYPE_RELEASES.to_string());
//! let state = match state.load().await {
//!     Ok(loaded_state) => loaded_state,
//!     Err(_) => state, // Use empty state for fresh start
//! };
//!
//! // Process items, skipping already completed ones
//! for item in items {
//!     if !state.has(item.id.clone()) {
//!         process_item(item).await?;
//!         state.add(item.id);
//!         state.persist().await?; // Save progress
//!     }
//! }
//! ```
//!
//! ### Cache Invalidation
//! ```rust
//! // Check if cache is outdated
//! let cached_count = manager.count_artists();
//! let remote_count = get_remote_artist_count().await?;
//!
//! if remote_count > cached_count {
//!     // Update cache with new data
//!     update_cache().await?;
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Lazy Loading**: Data is loaded only when needed
//! - **Efficient Serialization**: Uses JSON for balance of readability and performance
//! - **Hierarchical Organization**: Reduces file sizes and improves access patterns
//! - **Selective Caching**: Only caches data that provides significant benefit
//!
//! ## Security Considerations
//!
//! - **Token Security**: OAuth tokens stored in user's private data directory
//! - **File Permissions**: Relies on OS file permissions for security
//! - **No Secrets**: No client secrets or permanent credentials stored
//! - **Token Rotation**: Supports automatic token refresh and rotation
//!
//! ## Thread Safety
//!
//! The managers are designed for single-threaded use within async contexts:
//! - File operations use async I/O to avoid blocking
//! - State modifications are explicit and controlled
//! - No internal mutexes or locks (delegated to caller if needed)
//!
//! ## Integration Patterns
//!
//! ### CLI Integration
//! ```rust
//! // Typical usage pattern in CLI commands
//! let mut manager = ArtistReleaseManager::load().await?;
//! let artists = fetch_new_artists().await?;
//! manager.add_artists(artists);
//! manager.persist().await?;
//! ```
//!
//! ### API Integration
//! ```rust
//! // Token management for API calls
//! let mut token_manager = TokenManager::load().await?;
//! let valid_token = token_manager.get_valid_token().await;
//! let response = api_client.get().bearer_auth(valid_token).send().await?;
//! ```
//!
//! ## Error Recovery Strategies
//!
//! - **Corrupted Cache**: Falls back to empty cache and rebuilds
//! - **Token Expiry**: Automatically refreshes using refresh token
//! - **File System Issues**: Provides clear error messages with guidance
//! - **Network Failures**: Operations can resume from cached state
//!
//! ## Future Extensibility
//!
//! The management layer is designed for future enhancements:
//! - Additional data types can be added with new managers
//! - Storage format can evolve while maintaining compatibility
//! - State types can be extended for new operation types
//! - Caching strategies can be enhanced with more sophisticated policies
//!
//! ## Constants
//!
//! The module exports several constants for state type identification:
//! - [`STATE_TYPE_ARTISTS`] - Identifier for artist processing state
//! - [`STATE_TYPE_RELEASES`] - Identifier for release processing state

mod artist;
mod auth;
mod release;
mod state;

pub use artist::ArtistReleaseManager;
pub use auth::TokenManager;
pub use release::ReleaseWeekManager;
pub use state::STATE_TYPE_ARTISTS;
pub use state::STATE_TYPE_RELEASES;
pub use state::StateManager;
