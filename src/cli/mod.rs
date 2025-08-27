//! # CLI Module
//!
//! This module provides the command-line interface layer for Sporlcli, a Spotify API
//! client for tracking new releases from followed artists. It implements all user-facing
//! CLI commands and coordinates between the underlying API services, data management,
//! and user interaction components.
//!
//! ## Overview
//!
//! The CLI module serves as the primary interface between users and the Sporlcli
//! application's functionality. It provides a comprehensive set of commands for:
//!
//! - **Authentication Management**: OAuth 2.0 PKCE flow for Spotify API access
//! - **Artist Management**: Following, caching, and listing followed artists
//! - **Release Tracking**: Fetching, caching, and displaying new releases
//! - **Playlist Generation**: Creating weekly playlists from new releases
//! - **Information Queries**: Various data and status information commands
//!
//! ## Command Categories
//!
//! ### Authentication
//!
//! - [`auth`] - Initiates Spotify OAuth authentication flow with PKCE security
//!
//! ### Artist Operations
//!
//! - [`update_artists`] - Synchronizes local artist cache with followed artists from Spotify
//! - [`list_artists`] - Displays cached followed artists with optional search filtering
//!
//! ### Release Operations
//!
//! - [`update_releases`] - Fetches and caches new releases from all followed artists
//! - [`list_releases`] - Shows releases organized by week with time-range filtering
//!
//! ### Playlist Operations
//!
//! - [`playlist`] - Creates Spotify playlists from releases in specified time periods
//!
//! ### Information Commands
//!
//! - [`info`] - Provides various information about application state and data
//!
//! ## Architecture Design
//!
//! The CLI module follows a layered architecture approach:
//!
//! ```text
//! CLI Layer (User Interface)
//!     ↓
//! Management Layer (Data/Cache Management)
//!     ↓
//! API Layer (Spotify Integration)
//!     ↓
//! Network Layer (HTTP Requests)
//! ```
//!
//! Each CLI command delegates to appropriate management and API modules while
//! handling user interaction, progress feedback, and error presentation.
//!
//! ## Data Flow Patterns
//!
//! ### Update Operations
//! 1. **Authentication Check**: Verify valid tokens exist
//! 2. **State Management**: Load or create processing state
//! 3. **API Interaction**: Fetch data from Spotify with rate limiting
//! 4. **Cache Management**: Organize and persist data locally
//! 5. **Progress Feedback**: Provide real-time user feedback
//!
//! ### Query Operations
//! 1. **Cache Loading**: Load requested data from local storage
//! 2. **Data Processing**: Filter, sort, and format for display
//! 3. **Output Generation**: Create formatted tables or information
//! 4. **Error Handling**: Gracefully handle missing or corrupt data
//!
//! ## Error Handling Philosophy
//!
//! The CLI module implements user-friendly error handling:
//!
//! - **Graceful Degradation**: Partial failures don't prevent useful operations
//! - **Helpful Messages**: Clear guidance on how to resolve issues
//! - **Context Preservation**: Error messages include relevant context information
//! - **Recovery Suggestions**: Actionable advice for user recovery steps
//!
//! ## Progress and User Experience
//!
//! All long-running operations provide comprehensive user feedback:
//!
//! - **Progress Indicators**: Visual progress bars and spinners for operations
//! - **Status Messages**: Informative messages about current operation status
//! - **Success Confirmation**: Clear indication when operations complete successfully
//! - **Detailed Output**: Rich formatting using tables and color coding
//!
//! ## Caching Strategy
//!
//! The CLI coordinates a sophisticated caching system:
//!
//! - **Artist Cache**: Stores followed artists with metadata
//! - **Release Cache**: Organizes releases by week/year for efficient queries
//! - **State Cache**: Tracks operation progress for resume capability
//! - **Token Cache**: Manages OAuth tokens with automatic refresh
//!
//! ## Security Considerations
//!
//! - **OAuth 2.0 PKCE**: Implements secure authentication without client secrets
//! - **Token Management**: Secure storage and automatic refresh of access tokens
//! - **Local Storage**: All sensitive data stored in user's local data directory
//! - **Network Security**: Uses HTTPS for all API communications
//!
//! ## Performance Optimization
//!
//! - **Incremental Updates**: Only fetch new data when cache is outdated
//! - **Batch Processing**: Group API requests to minimize network overhead
//! - **Concurrent Operations**: Use async/await for parallel processing where safe
//! - **Memory Efficiency**: Process data in chunks to manage memory usage
//!
//! ## Usage Patterns
//!
//! ### Initial Setup
//! ```bash
//! sporlcli auth                    # Authenticate with Spotify
//! sporlcli artists update          # Cache followed artists
//! sporlcli releases update         # Cache release data
//! ```
//!
//! ### Regular Usage
//! ```bash
//! sporlcli releases                # View current week's releases
//! sporlcli playlist               # Create playlist for current week
//! sporlcli info --artists         # Check cache status
//! ```
//!
//! ### Advanced Queries
//! ```bash
//! sporlcli releases --previous-weeks 4    # View last 4 weeks
//! sporlcli artists --search rock          # Find specific artists
//! sporlcli playlist --release-date 2023-12-25  # Historical playlists
//! ```
//!
//! ## Dependencies
//!
//! This module depends on several core application components:
//! - [`crate::spotify`] - Spotify API integration and authentication
//! - [`crate::management`] - Data caching and state management
//! - [`crate::types`] - Data structures and type definitions
//! - [`crate::utils`] - Date handling and utility functions
//!
//! ## Error Logging and Debugging
//!
//! - Uses structured logging macros for consistent output formatting
//! - Provides different log levels (info, warning, error, success)
//! - Includes contextual information in error messages
//! - Supports detailed debugging through verbose output modes

mod artists;
mod auth;
mod info;
mod playlist;
mod releases;

pub use artists::list_artists;
pub use artists::update_artists;
pub use auth::auth;
pub use info::info;
pub use playlist::playlist;
pub use releases::list_releases;
pub use releases::update_releases;
