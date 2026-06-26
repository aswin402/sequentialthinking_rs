# 📝 Changelog

All notable changes to the **Sequential Thinking MCP Server (Rust)** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-06-27

This release implements **Milestone 6** of the implementation plan, establishing support for interchangeable communication transports (Stdio and HTTP/SSE), providing seamless support for remote deployment via Docker, and introducing a dedicated server health monitoring endpoint.

### Added
- **Transport Layer Abstraction**: Designed the `Transport` trait and a shared JSON-RPC message processing engine in `src/transport/mod.rs` to support multiple messaging transport mechanisms.
- **Stdio Transport**: Extracted the terminal input/output listener into `src/transport/stdio.rs`.
- **HTTP/SSE Transport**: Implemented standard Server-Sent Events (SSE) server in `src/transport/http.rs` using `axum`. Generates a unique `sessionId` on GET connection, streams initial endpoint data event, and receives JSON-RPC requests via POST.
- **Health Check Endpoint**: Added a GET `/health` route returning server status for infrastructure health checks.
- **CORS Support**: Configured tower CORS layers allowing remote clients to securely connect to the HTTP server.
- **Docker Deployment Configuration**: Updated the `Dockerfile` to expose port 3000, initialize a database path `/data/history.db` for sqlite, and configure default environmental values.
- **Command Line Flags**: Added `--transport` and `--port` CLI options (with env mapping) to control transport backend setup.
- **Integration Testing**: Created full client GET -> SSE -> POST handshake loop tests.

---

## [0.6.0] - 2026-06-26

This release implements **Milestone 5** of the implementation plan, adding a dedicated `src/graph` module containing graph visualization and thought quality algorithms (contradiction and cycle/loop detectors), and a new templates tool providing structured reasoning patterns.

### Added
- **Graph Module Separation**: Extracted `generate_mermaid` from `src/server.rs` to a new isolated module `src/graph/mermaid.rs`.
- **Thought Quality Analysis (`src/graph/quality.rs`)**: Implemented a comprehensive thought quality evaluation engine calculating a grade, a quality score (0-100), and statistics for confidence and assumption verification.
- **Contradiction Detection**: Added contradiction detection matching declared assumptions against refuted/falsified outcomes in the session history.
- **Loop/Cycle Detector**: Created a DFS-based cycle detection algorithm that detects dependency loops inside the reasoning chain and extracts the loop path.
- **Extended Graph Analysis**: Added `"quality_report"` query type and integrated quality scores/grades directly into `"summary_stats"` query type in the `analyze_graph` tool.
- **Reasoning Templates Tool (`reasoning_templates`)**: Implemented a templates tool in `src/tools/templates.rs` returning structured guides for `"divide-and-conquer"`, `"hypothesis-test"`, and `"devils-advocate"` thinking flows.
- **Unit and Integration Testing**: Added test coverage for contradiction detection, loop detection, and reasoning template execution.

---

## [0.5.0] - 2026-06-26

This release implements **Milestone 4** of the implementation plan, establishing a multi-tool architecture with a dynamic tool registry and four distinct reasoning-related MCP tools.

### Added
- **Dynamic Tool Registry**: Introduced the `McpTool` trait and a dynamic `ToolRegistry` in `src/tools/mod.rs` allowing modular registration and routing of multiple MCP tools.
- **Tool refactoring**: Extracted the core `sequentialthinking` logic into its own tool file `src/tools/sequentialthinking.rs`.
- **Graph Analysis Tool (`analyze_graph`)**: Added a tool in `src/tools/analyze_graph.rs` to inspect the thought graph and filter for low-confidence nodes, contradictions, unverified assumptions, and dead branches.
- **Session Exporting Tool (`export_session`)**: Added a tool in `src/tools/export_session.rs` to serialize and export the thinking session in multiple formats: Mermaid, JSON graph, DOT graph, and Markdown document.
- **Reasoning Summarization Tool (`summarize_reasoning`)**: Added a tool in `src/tools/summarize.rs` providing metrics and a timeline overview of the thinking process for a session.
- **JSON-RPC Routing Refactoring**: Refactored the core message dispatcher in `src/main.rs` to dynamically route requests based on tool registration and dynamically query available tools.
- **Unit and Integration Testing**: Added dedicated test cases in `src/tools/mod.rs` covering registration lookup, execution, serialization, and correct behavior for all new tools.

---

## [0.4.0] - 2026-06-26

This release implements **Milestone 3** of the implementation plan, adding persistent storage backends (in-memory and SQLite) and multi-session capability with UUID session mapping.

### Added
- **ThoughtStore Trait**: Designed a decoupled `ThoughtStore` trait in `src/persistence/mod.rs` allowing interchangeable persistence layers.
- **In-Memory Storage**: Extracted memory-backed persistence to `src/persistence/memory.rs` storing thoughts inside memory hash structures.
- **SQLite Database Persistence**: Added a fully featured SQLite-backed storage in `src/persistence/sqlite.rs` using `rusqlite`.
- **Database Schema**: Setup schemas with dynamic creation of `sessions` and `thoughts` tables (with foreign keys and cascading deletes) and index optimizations (`idx_thoughts_session`).
- **Multi-session Support (`sessionId`)**: Added `sessionId` parameter to input and output JSON-RPC structures, enabling client agents to request and switch between separate thinking sessions. Auto-generates a UUID v4 session identifier if none is provided.
- **Storage Configuration CLI Flags**: Added `--storage` flag (options: `memory`, `sqlite`) and optional `--db-path` flag to configure the SQLite database file path (defaulting to `~/.sequentialthinking/history.db`).
- **Integration Testing**: Created a `test_sqlite_persistence` integration test verifying connection setup, table migration, and correct thought persistence/reload cycles.

---

## [0.3.0] - 2026-06-26

This release implements **Milestone 2** of the implementation plan, replacing raw stderr outputs with structured, level-filtered tracing logs and instrumentation spans.

### Added
- **Structured Logging Framework**: Added `tracing` and `tracing-subscriber` to manage server logs.
- **Log Format Customization**: Added `--log-format` command line argument, allowing switching logging format between human-readable colored output (`pretty`) and aggregatable JSON format (`json`).
- **Observability Configuration**: Added `src/logging.rs` configuring global logging targets to safely route all logs to standard error (`stderr`), keeping standard output (`stdout`) clean for JSON-RPC communication.
- **Thought Box Logging**: Formatted TUI thought boxes are now piped through standard `tracing::info!` under the target `thought_tui` instead of raw `eprintln!`.
- **System Instrumentation**: Decorated critical reasoning and formatting functions (`process_thought`, `format_thought`, `generate_mermaid`) with `#[tracing::instrument]` to support profiling and log tracking.
- **RPC Lifecycle Logs**: Added structured debug logging for incoming JSON-RPC methods and error/warning events (such as parse errors and missing parameters).

---

## [0.2.0] - 2026-06-26

This release implements **Milestone 1** of the implementation plan, wiring up previously unused data structures, adding checklist/TODO tracking, introducing automatic reasoning timestamps, and enhancing visual output representations.

### Added
- **Hypothesis & Verification Rendering**: Structured rendering blocks inside `format_thought()` for both `hypothesis` (`🔬 Hypothesis:`) and `verification_method` (`🧪 Verification:`) fields.
- **Mermaid Graph Hypothesis Nodes**: Added a new CSS class Definition (`classDef hypothesis`) highlighting hypothesis-based reasoning blocks in purple (`#d1b3ff`) to visually distinguish them in generated Mermaid diagrams.
- **TODO List (`leftToBeDone`)**: Added `left_to_be_done` array support to `ThoughtData` and `ToolResult` DTOs.
- **TUI TODO Rendering**: In-box rendering of tasks left to be done as yellow checklist items (`📋 TODO: <item>`).
- **Reasoning Timestamps**: Automatic population of thoughts with UTC timestamps using `chrono::Utc::now()` if not provided, displayed cleanly in the terminal thought box headers (e.g., `💭 Thought 1/3 @ 20:25:46`).
- **MCP Schema Expansion**: Exposed `hypothesis`, `verificationMethod`, and `leftToBeDone` in the JSON Schema definition for the `sequentialthinking` tool inside the `tools/list` response.
- **Unit Testing**: Added a new unit test `test_new_fields` in `src/server.rs` ensuring correctness of JSON propagation, timestamp auto-assignment, and todo listings.

### Changed
- Bumped project version to `v0.2.0` in `Cargo.toml`.
- Updated server initialization metadata to report version `0.2.0` in the JSON-RPC response.
- Updated all existing test fixtures in `src/server.rs` to support the new `left_to_be_done` and `timestamp` fields.

---

## [0.1.0] - 2026-06-26

### Added
- Initial Rust implementation of the Sequential Thinking MCP Server.
- Custom stdio JSON-RPC 2.0 message router.
- Core sequential reasoning loop with Graph of Thoughts (GoT) support via branch spawning, revision backtracking, and DAG merges.
- Dynamic terminal width adaptive boxed text formatter.
- Mermaid graph compiler for visualization of thought chains.
- Ported Dockerfile configuration.
