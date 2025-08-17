use axum::response::Json;
use serde_json::{Value, json};

/// Provides a health check endpoint for monitoring server status.
///
/// Returns a JSON response indicating that the server is running and responsive.
/// This endpoint is commonly used by monitoring systems, load balancers, and
/// health check tools to verify that the application is operational.
///
/// The response includes:
/// - Status indicator ("ok" when healthy)
/// - Application version from Cargo.toml
///
/// # Returns
///
/// Returns a `Json<Value>` response containing:
/// ```json
/// {
///   "status": "ok",
///   "version": "x.y.z"
/// }
/// ```
///
/// # Usage
///
/// This endpoint is typically accessed via:
/// - Monitoring systems for uptime checks
/// - Load balancers for health verification
/// - Deployment scripts for readiness checks
/// - Manual testing to verify server responsiveness
///
/// # Example
///
/// ```
/// // GET /health
/// // Response: {"status": "ok", "version": "1.0.0"}
/// ```
///
/// # Notes
///
/// - This endpoint does not require authentication
/// - The response is lightweight and fast
/// - Version information helps with deployment tracking
/// - Always returns 200 OK status when the server is running
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
