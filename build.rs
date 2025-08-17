//! Build script for Spotify Release Tracker CLI.
//!
//! This build script handles setup tasks that need to occur during the compilation
//! process, primarily related to copying configuration templates to the user's
//! local data directory. This ensures that users have access to configuration
//! examples in the expected location after installation.

use std::{env, fs, path::PathBuf};

/// Main build script entry point that handles configuration file setup.
///
/// Executes during the cargo build process to copy configuration templates
/// from the project source to the user's local data directory. This provides
/// users with ready-to-use configuration examples in the standard location
/// where the application expects to find them.
///
/// # Build Process
///
/// The script performs the following operations:
/// 1. **Dependency Tracking**: Sets up cargo to re-run when template files change
/// 2. **Path Resolution**: Determines source and destination paths for templates
/// 3. **Directory Creation**: Ensures the target directory structure exists
/// 4. **File Copying**: Copies configuration templates to the local data directory
/// 5. **Error Handling**: Provides warnings for missing files instead of failing
///
/// # File Operations
///
/// ## Source Location
/// The script looks for `.env.example` in the crate root directory (where Cargo.toml resides).
///
/// ## Destination Location
/// Templates are copied to the platform-specific local data directory:
/// - Linux: `~/.local/share/sporlcli/.env.example`
/// - macOS: `~/Library/Application Support/sporlcli/.env.example`
/// - Windows: `%LOCALAPPDATA%/sporlcli/.env.example`
///
/// # Cargo Integration
///
/// The script integrates with cargo's build system:
/// - **Rebuild Triggers**: Uses `cargo:rerun-if-changed` to rebuild when templates change
/// - **Warning Output**: Uses `cargo:warning` for non-fatal issues
/// - **Error Propagation**: Returns errors for critical failures
///
/// # Error Handling Strategy
///
/// The script uses a graceful error handling approach:
/// - **Missing Templates**: Issues warnings but continues build
/// - **Directory Creation Failures**: Returns errors (critical)
/// - **File Copy Failures**: Returns errors (critical)
/// - **Path Resolution Failures**: Returns errors (critical)
///
/// # User Experience Benefits
///
/// This setup provides several benefits to users:
/// - **Ready-to-Use Templates**: Configuration examples are immediately available
/// - **Standard Locations**: Files are placed where the application expects them
/// - **Platform Compatibility**: Works across different operating systems
/// - **Easy Configuration**: Users can copy and modify templates easily
///
/// # Development Workflow
///
/// For developers working on the project:
/// - Template changes automatically trigger rebuilds
/// - Build failures are clear and actionable
/// - Local testing uses the same setup as installed versions
///
/// # Returns
///
/// Returns a `Result` indicating build success or failure:
/// - `Ok(())` - All operations completed successfully
/// - `Err(Box<dyn std::error::Error>)` - Critical failure occurred
///
/// # Example Output
///
/// Successful build with existing template:
/// ```text
/// (No output - silent success)
/// ```
///
/// Build with missing template:
/// ```text
/// warning: env.example not found at /path/to/project/.env.example
/// ```
///
/// # Environment Variables Used
///
/// - `CARGO_MANIFEST_DIR` - Path to the crate root directory (provided by cargo)
///
/// # Error Scenarios
///
/// The script may fail in these situations:
/// - Unable to read `CARGO_MANIFEST_DIR` environment variable
/// - Cannot determine user's local data directory
/// - Insufficient permissions to create directories
/// - Insufficient permissions to write files
/// - File system errors during copy operations
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Re-run if the template changes
    println!("cargo:rerun-if-changed=env.example");

    // Where to copy FROM (crate root)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let env_example_path = manifest_dir.join(".env.example");

    // Compute target dir (your local data dir) and ensure it exists
    let mut out_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    out_dir.push("sporlcli");
    fs::create_dir_all(&out_dir)?; // <-- create the actual directory, not only its parent

    // Only copy if the source exists; otherwise warn instead of failing
    if env_example_path.is_file() {
        let contents = fs::read_to_string(&env_example_path)?;
        fs::write(out_dir.join(".env.example"), contents)?;
    } else {
        println!(
            "cargo:warning=env.example not found at {}",
            env_example_path.display()
        );
        // If this should be fatal, replace with: return Err("env.example missing".into());
    }

    Ok(())
}
