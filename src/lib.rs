//! Spotify Release Tracker CLI Library
//!
//! This library provides functionality for tracking music releases from followed
//! artists on Spotify. It includes modules for API communication, CLI operations,
//! configuration management, and various utilities for managing music release data.
//!
//! # Modules
//!
//! - `api` - HTTP API endpoints for the local callback server
//! - `cli` - Command-line interface implementations
//! - `config` - Configuration management and environment variables
//! - `management` - High-level data management and caching
//! - `server` - Local HTTP server for OAuth callbacks
//! - `spotify` - Spotify Web API client implementation
//! - `types` - Data structures and type definitions
//! - `utils` - Utility functions and helpers
//!
//! # Example
//!
//! ```
//! use sporlcli::{config, cli};
//!
//! #[tokio::main]
//! async fn main() -> sporlcli::Res<()> {
//!     config::load_env().await?;
//!     // Use CLI functions...
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod cli;
pub mod config;
pub mod management;
pub mod server;
pub mod spotify;
pub mod types;
pub mod utils;

/// A convenient Result type alias for operations that may fail.
///
/// Provides a standard error handling pattern throughout the application
/// using a boxed dynamic error trait object. This allows for flexible
/// error handling while maintaining Send + Sync bounds for async contexts.
///
/// # Type Parameters
///
/// - `T` - The success type returned on successful operations
///
/// # Example
///
/// ```
/// use sporlcli::Res;
///
/// async fn fetch_data() -> Res<String> {
///     Ok("data".to_string())
/// }
/// ```
pub type Res<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Prints an informational message with a blue bullet point.
///
/// Creates a formatted output line with a distinctive blue "o" indicator
/// followed by the provided message. Used for general information and
/// status updates throughout the application.
///
/// # Arguments
///
/// The macro accepts the same arguments as `println!`, supporting format
/// strings and interpolation.
///
/// # Example
///
/// ```
/// info!("Starting authentication process...");
/// info!("Found {} releases", count);
/// ```
#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "o".blue().bold(), std::format_args!($($arg)*));
  })
}

/// Prints a success message with a green checkmark.
///
/// Creates a formatted output line with a green "✓" indicator to signify
/// successful completion of operations. Used to provide positive feedback
/// when operations complete successfully.
///
/// # Arguments
///
/// The macro accepts the same arguments as `println!`, supporting format
/// strings and interpolation.
///
/// # Example
///
/// ```
/// success!("Authentication completed successfully");
/// success!("Updated {} artists", count);
/// ```
#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "✓".green().bold(), std::format_args!($($arg)*));
  })
}

/// Prints an error message with a red exclamation mark and exits the program.
///
/// Creates a formatted error output with a red "!" indicator and immediately
/// terminates the program with exit code 1. Used for unrecoverable errors
/// that require immediate program termination.
///
/// # Arguments
///
/// The macro accepts the same arguments as `println!`, supporting format
/// strings and interpolation.
///
/// # Behavior
///
/// This macro will cause the program to exit immediately after printing
/// the error message. It should only be used for fatal errors where
/// recovery is not possible.
///
/// # Example
///
/// ```
/// error!("Failed to load configuration");
/// error!("Missing required environment variable: {}", var_name);
/// // Program exits here - code after this will not execute
/// ```
#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "!".red().bold(), std::format_args!($($arg)*));
    std::process::exit(1);
  })
}

/// Prints a warning message with a yellow exclamation mark.
///
/// Creates a formatted output line with a yellow "!" indicator to highlight
/// potential issues or important notices that don't require program termination.
/// Used for recoverable issues or important information that users should notice.
///
/// # Arguments
///
/// The macro accepts the same arguments as `println!`, supporting format
/// strings and interpolation.
///
/// # Example
///
/// ```
/// warning!("Cache file not found, will create new one");
/// warning!("Rate limit approaching: {} requests remaining", remaining);
/// ```
#[macro_export]
macro_rules! warning {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "!".yellow().bold(), std::format_args!($($arg)*));
  })
}
