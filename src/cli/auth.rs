use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{spotify, types::PkceToken};

/// Initiates the OAuth authentication flow for Spotify API access.
///
/// This is the CLI entry point for user authentication, providing a simplified
/// interface to the underlying OAuth 2.0 PKCE (Proof Key for Code Exchange) flow.
/// The function handles the complete authentication process including browser
/// interaction, callback handling, and token persistence.
///
/// # Arguments
///
/// * `shared_state` - Thread-safe shared state for managing PKCE tokens and
///   authentication results between the CLI interface and callback handlers
///
/// # Authentication Flow
///
/// The function orchestrates the complete OAuth flow:
/// 1. **PKCE Generation**: Creates cryptographically secure code verifier and challenge
/// 2. **Server Startup**: Launches local HTTP server for OAuth callbacks
/// 3. **Browser Launch**: Opens Spotify authorization URL in user's default browser
/// 4. **User Interaction**: User grants permissions in their browser
/// 5. **Callback Handling**: Local server receives authorization code from Spotify
/// 6. **Token Exchange**: Authorization code is exchanged for access/refresh tokens
/// 7. **Token Persistence**: Tokens are securely stored for future API requests
///
/// # User Experience
///
/// The function provides a seamless authentication experience:
/// - Automatically opens the browser to Spotify's authorization page
/// - Provides fallback instructions if browser launch fails
/// - Shows clear success/failure messages
/// - Handles timeouts and errors gracefully
/// - Guides users through any required steps
///
/// # Security Features
///
/// Implements OAuth 2.0 security best practices:
/// - **PKCE Flow**: Uses Proof Key for Code Exchange to prevent code interception
/// - **Local Callback**: Receives callbacks on localhost to prevent redirect attacks
/// - **Temporary Server**: HTTP server runs only during authentication
/// - **Secure Storage**: Tokens are stored in user's local data directory
/// - **No Client Secrets**: Avoids storing sensitive client credentials
///
/// # Error Handling
///
/// The function handles various failure scenarios:
/// - Browser launch failures (provides manual URL)
/// - Network connectivity issues
/// - User denial of permissions
/// - Authentication timeouts
/// - Token exchange failures
/// - Storage/persistence errors
///
/// # Thread Safety
///
/// Uses Arc<Mutex<>> for thread-safe communication between:
/// - The main authentication flow
/// - The HTTP callback server
/// - Timeout monitoring
/// - Progress tracking
///
/// # Example Usage
///
/// ```
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// let shared_state = Arc::new(Mutex::new(None));
/// auth(shared_state).await;
/// // User is now authenticated and tokens are stored
/// ```
///
/// # Post-Authentication
///
/// After successful authentication:
/// - Access tokens are available for API requests
/// - Refresh tokens enable automatic token renewal
/// - User can immediately use other CLI commands
/// - Authentication state persists across sessions
///
/// # Troubleshooting
///
/// Common issues and solutions:
/// - **Browser doesn't open**: Manual URL is provided in output
/// - **Permission denied**: User must grant required scopes
/// - **Timeout**: Process can be restarted safely
/// - **Port conflicts**: Default callback port may need configuration
///
/// # Dependencies
///
/// This function delegates the actual implementation to `spotify::auth::auth()`,
/// providing a clean separation between CLI interface and authentication logic.
/// The CLI layer focuses on user interaction while the spotify module handles
/// the technical OAuth implementation details.
pub async fn auth(shared_state: Arc<Mutex<Option<PkceToken>>>) {
    spotify::auth::auth(shared_state).await;
}
