use clap::Parser;

mod server;
mod types;
mod logging;
mod persistence;
mod tools;
mod graph;
mod transport;

use server::SequentialThinkingServer;

/// Sequential Thinking MCP Server in Rust (with Graph of Thoughts & Clear Thought features)
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Sequential Thinking MCP Server in Rust (with Graph of Thoughts & Clear Thought features)"
)]
struct Args {
    /// Disable thought boxes print on stderr
    #[arg(short, long, env = "DISABLE_THOUGHT_LOGGING")]
    disable_thought_logging: bool,

    /// Log format for structured logs ("pretty" or "json")
    #[arg(long, default_value = "pretty", env = "LOG_FORMAT")]
    log_format: String,

    /// Storage backend ("memory" or "sqlite")
    #[arg(long, default_value = "memory", env = "STORAGE")]
    storage: String,

    /// Path to SQLite database file
    #[arg(long, env = "DB_PATH")]
    db_path: Option<String>,

    /// Transport layer ("stdio" or "http")
    #[arg(long, default_value = "stdio", env = "TRANSPORT")]
    transport: String,

    /// Port to listen on (only applicable for "http" transport)
    #[arg(short, long, default_value = "3000", env = "PORT")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    logging::setup_logging(&args.log_format);

    let store: Box<dyn persistence::ThoughtStore> = match args.storage.as_str() {
        "sqlite" => {
            let db_path = args.db_path.clone().unwrap_or_else(|| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                let dir = std::path::Path::new(&home).join(".sequentialthinking");
                let _ = std::fs::create_dir_all(&dir);
                dir.join("history.db").to_str().unwrap().to_string()
            });
            Box::new(persistence::sqlite::SqliteThoughtStore::new(&db_path).expect("Failed to initialize SQLite store"))
        }
        _ => Box::new(persistence::memory::MemoryThoughtStore::new()),
    };

    let thinking_server = SequentialThinkingServer::new(store, args.disable_thought_logging);

    let mut tool_registry = tools::ToolRegistry::new();
    tool_registry.register(Box::new(tools::sequentialthinking::SequentialThinkingTool));
    tool_registry.register(Box::new(tools::analyze_graph::AnalyzeGraphTool));
    tool_registry.register(Box::new(tools::export_session::ExportSessionTool));
    tool_registry.register(Box::new(tools::summarize::SummarizeReasoningTool));
    tool_registry.register(Box::new(tools::templates::TemplatesTool));

    let server_arc = std::sync::Arc::new(tokio::sync::Mutex::new(thinking_server));
    let registry_arc = std::sync::Arc::new(tool_registry);

    let result = if args.transport == "http" {
        use crate::transport::Transport;
        let transport = transport::http::HttpTransport { port: args.port };
        transport.run(server_arc, registry_arc).await
    } else {
        use crate::transport::Transport;
        let transport = transport::stdio::StdioTransport;
        transport.run(server_arc, registry_arc).await
    };

    if let Err(e) = result {
        tracing::error!(error = %e, "Server transport execution failed");
        std::process::exit(1);
    }
}
