# 📋 Sequential Thinking MCP Server — Project TODO List

This TODO list maps directly to [implementationplan.md](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/implementationplan.md) and tracks our progress from `v0.1.0` to `v1.0.0`.

---

## 🎯 Milestone 1: Fix Dead Code & Add Missing Fields (`v0.2.0`)
- [x] **Task 1.1**: Wire up `hypothesis` rendering in TUI box (`src/server.rs` -> `format_thought()`)
- [x] **Task 1.2**: Wire up `hypothesis` node highlight styling in Mermaid rendering (`src/server.rs` -> `generate_mermaid()`)
- [x] **Task 1.3**: Add `hypothesis` and `verificationMethod` properties to JSON schema in tools registration (`src/main.rs`)
- [x] **Task 1.4**: Add `leftToBeDone` list support
  - [x] Add `left_to_be_done` to `ThoughtData` and `ToolResult` DTOs (`src/types.rs`)
  - [x] Render remaining items as `📋 TODO: ...` in the TUI box (`src/server.rs` -> `format_thought()`)
  - [x] Include `leftToBeDone` in the JSON schema block (`src/main.rs`)
  - [x] Propagate `left_to_be_done` during thought insertion (`src/server.rs` -> `process_thought()`)
- [x] **Task 1.5**: Timestamp thoughts using `chrono`
  - [x] Add `timestamp` field to `ThoughtData` (`src/types.rs`)
  - [x] Populate `timestamp` automatically using `chrono::Utc::now()` upon inserting a thought
  - [x] Render timestamps in the TUI box or as metadata
- [x] **Task 1.6**: Write unit tests for new fields in `src/server.rs`
  - [x] Test hypothesis formatting and rendering
  - [x] Test verification method formatting and rendering
  - [x] Test leftToBeDone rendering
  - [x] Test timestamp auto-population
- [x] **Task 1.7**: Bump project version to `v0.2.0` in `Cargo.toml` and server initialization response

---

## 🪵 Milestone 2: Structured Logging & Observability (`v0.3.0`)
- [x] **Task 2.1**: Add `tracing` and `tracing-subscriber` to `Cargo.toml`
- [x] **Task 2.2**: Implement structured logging initialization in `src/logging.rs`
  - [x] Support `--log-format pretty` (human-readable stderr output)
  - [x] Support `--log-format json` (machine-readable structured JSON aggregation)
- [x] **Task 2.3**: Replace raw `eprintln!` calls with tracing macros
  - [x] Log server status on startup (`src/main.rs`)
  - [x] Log JSON-RPC routing details and parse errors (`src/main.rs`)
  - [x] Pipe formatted thought panels to `stderr` via tracing instead of raw print (`src/server.rs`)
- [x] **Task 2.4**: Introduce `--log-format` CLI command-line arg in `src/main.rs`
- [x] **Task 2.5**: Instrumentation & spans
  - [x] Decorate `process_thought()` with `#[tracing::instrument]`
  - [x] Add tracing spans for Mermaid generation and TUI formatting

---

## 💾 Milestone 3: Session Persistence (SQLite) (`v0.4.0`)
- [x] **Task 3.1**: Add `rusqlite` and `uuid` dependencies to `Cargo.toml`
- [x] **Task 3.2**: Define `ThoughtStore` trait in `src/persistence/mod.rs`
- [x] **Task 3.3**: Extract in-memory backend to `src/persistence/memory.rs`
- [x] **Task 3.4**: Implement SQLite backend in `src/persistence/sqlite.rs`
  - [x] Create `sessions` table
  - [x] Create `thoughts` table with foreign key reference
  - [x] Implement DB schema migrations/initialization on startup
- [x] **Task 3.5**: Implement multi-session support with `sessionId`
  - [x] Add `sessionId` to thought insertion requests/responses (`src/types.rs`)
  - [x] Generate UUID v4 for new sessions if `sessionId` is not provided
- [x] **Task 3.6**: Implement CLI flag `--storage` ("memory" or "sqlite") and `--db-path` in `src/main.rs`
- [x] **Task 3.7**: Refactor `SequentialThinkingServer` to use `Box<dyn ThoughtStore>` instead of inline `Vec`
- [x] **Task 3.8**: Write storage integration tests (DB init, round-trips, multi-sessions)

---

## 🛠️ Milestone 4: Multi-Tool Architecture (`v0.5.0`)
- [x] **Task 4.1**: Define the `McpTool` trait and a dynamic `ToolRegistry` in `src/tools/mod.rs`
- [x] **Task 4.2**: Refactor `sequentialthinking` into its own file `src/tools/sequentialthinking.rs`
- [x] **Task 4.3**: Implement `analyze_graph` tool (`src/tools/analyze_graph.rs`)
  - [x] `low_confidence` query filter
  - [x] `contradictions` query filter
  - [x] `unverified_assumptions` listing
  - [x] `dead_branches` branch path isolation
- [x] **Task 4.4**: Implement `export_session` tool (`src/tools/export_session.rs`)
  - [x] Support format: `mermaid`
  - [x] Support format: `json`
  - [x] Support format: `markdown`
  - [x] Support format: `dot`
- [x] **Task 4.5**: Implement `summarize_reasoning` tool (`src/tools/summarize.rs`)
- [x] **Task 4.6**: Integrate the tool registry with JSON-RPC handlers in `src/main.rs`
- [x] **Task 4.7**: Write unit and integration tests for all new tools

---

## 🧠 Milestone 5: Thought Quality Scoring & Templates (`v0.6.0`)
- [x] **Task 5.1**: Split graph logic into `src/graph/` (extract `mermaid.rs` from `server.rs`)
- [x] **Task 5.2**: Implement quality scoring algorithm in `src/graph/quality.rs`
- [x] **Task 5.3**: Implement contradiction and loop detection across thought history
- [x] **Task 5.4**: Add reasoning templates tool (`src/tools/templates.rs`)
  - [x] Expose `divide-and-conquer` template
  - [x] Expose `hypothesis-test` template
  - [x] Expose `devils-advocate` template

---

## 🌐 Milestone 6: HTTP Transport & Remote Deployment (`v0.7.0` → `v1.0.0`)
- [ ] **Task 6.1**: Add `axum`, `tower`, and `tower-http` dependencies to `Cargo.toml`
- [ ] **Task 6.2**: Define `Transport` trait in `src/transport/mod.rs`
- [ ] **Task 6.3**: Extract stdio transport code to `src/transport/stdio.rs`
- [ ] **Task 6.4**: Implement HTTP/SSE transport in `src/transport/http.rs`
- [ ] **Task 6.5**: Add `--transport` ("stdio" or "http") and `--port` CLI flags in `src/main.rs`
- [ ] **Task 6.6**: Expose port 3000 in `Dockerfile` and setup environment defaults
- [ ] **Task 6.7**: Implement HTTP `/health` check endpoint
