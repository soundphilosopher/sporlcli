//! # API Module
//!
//! This module provides HTTP API endpoints for the Sporlcli application's web server
//! functionality. It implements essential endpoints for OAuth authentication and
//! health monitoring.
//!
//! ## Overview
//!
//! The API module serves as the web interface layer for Sporlcli, a command-line
//! interface for the Spotify API. It provides HTTP endpoints that handle:
//!
//! - **OAuth Authentication Flow**: Implements the Spotify OAuth 2.0 PKCE
//!   (Proof Key for Code Exchange) callback handler for secure token exchange
//! - **Health Monitoring**: Provides a health check endpoint for system monitoring
//!   and deployment verification
//!
//! ## Endpoints
//!
//! ### Authentication
//!
//! - [`callback`] - Handles OAuth callback requests from Spotify's authorization server.
//!   This endpoint completes the PKCE authentication flow by exchanging authorization
//!   codes for access tokens.
//!
//! ### Monitoring
//!
//! - [`health`] - Provides a health check endpoint that returns application status
//!   and version information for monitoring systems and load balancers.
//!
//! ## Architecture
//!
//! The module is built using the [Axum](https://docs.rs/axum) web framework and follows
//! RESTful design principles. Each endpoint is implemented as an async function that
//! can be easily integrated into Axum's routing system.
//!
//! ## Security Considerations
//!
//! - Uses OAuth 2.0 PKCE flow for enhanced security without exposing client secrets
//! - Implements proper state management for temporary authentication data
//! - Handles authentication failures gracefully with appropriate error responses
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use axum::{Router, routing::get};
//! use sporlcli::api::{callback, health};
//!
//! let app = Router::new()
//!     .route("/callback", get(callback))
//!     .route("/health", get(health));
//! ```
//!
//! ## Dependencies
//!
//! This module depends on:
//! - [`axum`] for HTTP server functionality
//! - [`tokio`] for async runtime support
//! - [`serde_json`] for JSON serialization
//!
//! ## Related Modules
//!
//! - [`crate::spotify`] - Spotify API integration
//! - [`crate::types`] - Type definitions for authentication tokens

mod callback;
mod health;

pub use callback::callback;
pub use health::health;
