use tracing_subscriber::EnvFilter;

/// Initializes the global tracing subscriber configured to write structured logs to stderr.
///
/// Supports two format modes:
/// - `"json"`: Machine-readable structured JSON format.
/// - `"pretty"`: Colored, human-readable format (default).
pub fn setup_logging(log_format: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    match log_format {
        "json" => {
            tracing_subscriber::fmt()
                .json()
                .with_writer(std::io::stderr)
                .with_env_filter(filter)
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_env_filter(filter)
                .init();
        }
    }
}
