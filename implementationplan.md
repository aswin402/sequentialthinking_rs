# 🗺️ Implementation Plan
## Sequential Thinking MCP Server (Rust) — v0.2.0 → v1.0.0

---

## Overview

This plan takes the project from its current v0.1.0 state (a single-tool, stdio-only, in-memory server) to a production-grade v1.0.0 (multi-tool, persistent, observable, transport-flexible reasoning platform). Work is organized into **5 milestones**, each shippable independently.

---

## Current State Assessment

### What Exists Today

```
src/
├── main.rs    (314 lines) — JSON-RPC router, CLI entrypoint, stdio transport
├── server.rs  (504 lines) — SequentialThinkingServer state engine, TUI formatting, Mermaid compiler
└── types.rs   (111 lines) — DTOs: JsonRpcRequest/Response, ThoughtData, ToolResult
```

### Known Technical Debt
1. `hypothesis` and `verification_method` fields exist in `ThoughtData` but are **never rendered** in `format_thought()` or used in `generate_mermaid()`.
2. `#[allow(unused_assignments)]` on `format_thought` — indicates a code smell in the variable initialization pattern.
3. All logging uses raw `eprintln!()` — no structured logging, no log levels.
4. `chrono` crate is declared as a dependency but **never imported or used** anywhere in `src/`.
5. No `sessionId` concept — all state is lost on process restart.
6. Single tool (`sequentialthinking`) — agents can only append thoughts, not query or export them.
7. Tests exist but coverage is minimal (4 tests, no edge cases for GoT merging with revisions).

---

## Target Architecture (v1.0.0)

```
src/
├── main.rs              — CLI entrypoint, transport selection (stdio vs HTTP)
├── transport/
│   ├── mod.rs           — Transport trait definition
│   ├── stdio.rs         — Stdio JSON-RPC transport (current behavior, extracted)
│   └── http.rs          — Streamable HTTP transport (axum + SSE)
├── server.rs            — Core SequentialThinkingServer (state engine)
├── tools/
│   ├── mod.rs           — Tool registry and dispatch
│   ├── sequentialthinking.rs  — Primary thinking tool (extracted from server.rs)
│   ├── analyze_graph.rs       — Query/filter thought graph
│   ├── export_session.rs      — Export session as JSON/Markdown/Mermaid
│   └── summarize.rs           — Summarize reasoning chain
├── persistence/
│   ├── mod.rs           — Persistence trait definition
│   ├── memory.rs        — In-memory storage (current behavior)
│   └── sqlite.rs        — SQLite-backed storage
├── graph/
│   ├── mod.rs           — Graph operations (quality scoring, contradiction detection)
│   ├── mermaid.rs       — Mermaid compiler (extracted from server.rs)
│   └── export.rs        — Multi-format export (JSON Graph, DOT, Markdown)
├── tui.rs               — TUI formatting (extracted from server.rs)
├── types.rs             — All DTOs and schemas
└── logging.rs           — Structured logging setup (tracing)
```

---

## Milestone 1: Fix Dead Code & Add Missing Fields
**Version**: `v0.2.0`
**Estimated Effort**: 1–2 days
**Risk**: Low

### Goal
Clean up existing technical debt and add the `leftToBeDone` field that competitors ship. Wire up the unused `hypothesis` and `verification_method` fields.

### Tasks

#### 1.1 — Wire up `hypothesis` in TUI rendering
**File**: [server.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/server.rs) → `format_thought()`
- Add a rendering block after the `criticism` section (after line ~197):
  ```rust
  if let Some(ref hypothesis) = thought_data.hypothesis {
      if !metadata_added {
          lines.push(mid_border.to_string());
          metadata_added = true;
      }
      let text = format!(" 🔬 Hypothesis: {}", hypothesis);
      let wrapped = wrap(&text, border_len);
      for w_line in wrapped {
          let padding = (border_len + 2).saturating_sub(w_line.len());
          lines.push(format!(
              "{} {}{} {}",
              color_func("│"),
              w_line.cyan(),
              " ".repeat(padding),
              color_func("│")
          ));
      }
  }
  ```
- Add the same pattern for `verification_method` with emoji `🧪 Verification:`.

#### 1.2 — Wire up `hypothesis` in Mermaid graph
**File**: [server.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/server.rs) → `generate_mermaid()`
- Add a new `classDef` for hypothesis nodes: `classDef hypothesis fill:#d1b3ff,stroke:#6a3d9a,stroke-width:2px,color:#000;`
- Apply `hypothesis` class to thoughts that contain a non-None `hypothesis` field.

#### 1.3 — Add `hypothesis` and `verificationMethod` to tool schema
**File**: [main.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/main.rs) → `tools/list` handler
- Add JSON schema properties for `hypothesis` and `verificationMethod` in the `inputSchema.properties` block (around line ~155).

#### 1.4 — Add `leftToBeDone` field
**File**: [types.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/types.rs)
- Add to `ThoughtData`:
  ```rust
  #[serde(rename = "leftToBeDone")]
  pub left_to_be_done: Option<Vec<String>>,
  ```
- Add to `ToolResult`:
  ```rust
  #[serde(rename = "leftToBeDone")]
  pub left_to_be_done: Vec<String>,
  ```
- Update `process_thought()` in `server.rs` to pass through the field.
- Update `format_thought()` to render remaining tasks as `📋 TODO: ...` lines.
- Update tool schema in `main.rs`.
- Update all test fixtures to include the new field.

#### 1.5 — Remove unused `chrono` dependency (or use it)
**File**: [Cargo.toml](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/Cargo.toml)
- `chrono` is declared but never used. Either remove it, or use it to timestamp thoughts (preferred — add a `timestamp` field to `ThoughtData` that auto-populates with `chrono::Utc::now()`).

#### 1.6 — Add unit tests for new fields
**File**: [server.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/server.rs) → `mod tests`
- Test hypothesis rendering.
- Test leftToBeDone pass-through.
- Test timestamp population.

#### 1.7 — Bump version to `v0.2.0`
**Files**: `Cargo.toml`, `main.rs` (server version in `initialize` handler)

---

## Milestone 2: Structured Logging & Observability
**Version**: `v0.3.0`
**Estimated Effort**: 1–2 days
**Risk**: Low

### Goal
Replace all `eprintln!()` calls with the `tracing` crate. Add structured spans for each thought processing step.

### Tasks

#### 2.1 — Add `tracing` dependencies
**File**: [Cargo.toml](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/Cargo.toml)
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

#### 2.2 — Create `src/logging.rs`
- Initialize `tracing_subscriber` in a `setup_logging()` function.
- Support two output modes:
  - **Human-readable** (default): Colored, formatted output to `stderr` (similar to current behavior).
  - **JSON mode** (via `--log-format json` CLI flag): Structured JSON logs for aggregation.
- Respect `RUST_LOG` env var for level filtering.

#### 2.3 — Replace all `eprintln!()` calls
**File**: [server.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/server.rs)
- Replace `eprintln!("{}", formatted)` with `tracing::info!(thought_number, total_thoughts, "thought_processed")`.
- Keep the TUI box rendering — but pipe it through `tracing` as the message payload, not raw stderr.

**File**: [main.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/main.rs)
- Replace `eprintln!("Sequential Thinking MCP Server running on stdio")` with `tracing::info!("server_started")`.

#### 2.4 — Add `--log-format` CLI flag
**File**: [main.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/main.rs) → `Args` struct
```rust
#[arg(long, default_value = "pretty", env = "LOG_FORMAT")]
log_format: String, // "pretty" or "json"
```

#### 2.5 — Add tracing spans
- Wrap `process_thought()` in a `#[tracing::instrument]` attribute.
- Add spans for `generate_mermaid()` and `format_thought()`.

---

## Milestone 3: Session Persistence (SQLite)
**Version**: `v0.4.0`
**Estimated Effort**: 3–5 days
**Risk**: Medium (new dependency, schema design, migration strategy)

### Goal
Persist thought sessions to SQLite so reasoning chains survive process restarts. Add `sessionId` support.

### Tasks

#### 3.1 — Add persistence dependencies
**File**: [Cargo.toml](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/Cargo.toml)
```toml
rusqlite = { version = "0.32", features = ["bundled"] }
uuid = { version = "1.0", features = ["v4"] }
```

#### 3.2 — Create `src/persistence/mod.rs`
Define a persistence trait:
```rust
pub trait ThoughtStore {
    fn save_thought(&mut self, session_id: &str, thought: &ThoughtData) -> Result<(), String>;
    fn load_session(&self, session_id: &str) -> Result<Vec<ThoughtData>, String>;
    fn list_sessions(&self) -> Result<Vec<SessionInfo>, String>;
    fn delete_session(&mut self, session_id: &str) -> Result<(), String>;
}
```

#### 3.3 — Create `src/persistence/memory.rs`
Extract current in-memory behavior into the `ThoughtStore` trait.

#### 3.4 — Create `src/persistence/sqlite.rs`
- Schema:
  ```sql
  CREATE TABLE sessions (
      id TEXT PRIMARY KEY,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      total_thoughts INTEGER DEFAULT 0
  );
  CREATE TABLE thoughts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      session_id TEXT NOT NULL REFERENCES sessions(id),
      thought_number INTEGER NOT NULL,
      thought_json TEXT NOT NULL,
      created_at TEXT NOT NULL
  );
  ```
- Default DB path: `~/.sequentialthinking/history.db`
- Configurable via `--db-path` CLI flag.

#### 3.5 — Add `sessionId` to `ThoughtData`
**File**: [types.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/types.rs)
```rust
#[serde(rename = "sessionId")]
pub session_id: Option<String>,
```
- If not provided, auto-generate a UUID v4.
- Return `sessionId` in `ToolResult` so the agent can pass it back.

#### 3.6 — Add `--storage` CLI flag
**File**: [main.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/src/main.rs)
```rust
#[arg(long, default_value = "memory", env = "STORAGE_BACKEND")]
storage: String, // "memory" or "sqlite"
```

#### 3.7 — Update `SequentialThinkingServer` to use trait
Refactor `server.rs` to accept a `Box<dyn ThoughtStore>` instead of raw `Vec<ThoughtData>`.

#### 3.8 — Write integration tests
- Test SQLite round-trip: save thoughts → restart server → reload session.
- Test session listing and deletion.
- Test auto-UUID generation when `sessionId` is not provided.

---

## Milestone 4: Multi-Tool Architecture
**Version**: `v0.5.0`
**Estimated Effort**: 3–5 days
**Risk**: Medium (requires refactoring the `tools/call` router)

### Goal
Expand from 1 tool to 5 tools. Refactor the tool dispatch system to be extensible.

### Tasks

#### 4.1 — Create `src/tools/mod.rs`
Define a tool trait:
```rust
pub trait McpTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn execute(
        &self,
        server: &mut SequentialThinkingServer,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String>;
}
```
- Create a `ToolRegistry` that holds `Vec<Box<dyn McpTool>>`.
- Refactor `main.rs` to use the registry for `tools/list` and `tools/call`.

#### 4.2 — Extract `sequentialthinking` tool
**File**: `src/tools/sequentialthinking.rs`
- Move the current `process_thought()` logic into a struct implementing `McpTool`.

#### 4.3 — Implement `analyze_graph` tool
**File**: `src/tools/analyze_graph.rs`
- **Purpose**: Query the thought graph without adding new thoughts.
- **Input schema**:
  ```json
  {
    "query": "low_confidence",          // Predefined query type
    "confidenceThreshold": 0.5,         // Optional filter
    "sessionId": "abc-123"              // Optional session filter
  }
  ```
- **Query types**:
  - `low_confidence` — Return thoughts below confidence threshold.
  - `contradictions` — Detect assumption conflicts.
  - `unverified_assumptions` — List assumptions not yet verified.
  - `dead_branches` — Branches that were never merged.
  - `summary_stats` — Return aggregate statistics.

#### 4.4 — Implement `export_session` tool
**File**: `src/tools/export_session.rs`
- **Purpose**: Export the current (or specified) session in various formats.
- **Input schema**:
  ```json
  {
    "sessionId": "abc-123",             // Optional, defaults to current
    "format": "mermaid"                 // "mermaid" | "json" | "markdown" | "dot"
  }
  ```
- Implement format converters:
  - **Mermaid**: Already exists in `generate_mermaid()` — extract and reuse.
  - **JSON Graph**: `{ "nodes": [...], "edges": [...] }` format.
  - **Markdown**: Human-readable summary with headers per thought.
  - **DOT/Graphviz**: Standard Graphviz format for rendering.

#### 4.5 — Implement `summarize_reasoning` tool
**File**: `src/tools/summarize.rs`
- **Purpose**: Generate a structured summary of the reasoning chain.
- Returns:
  ```json
  {
    "totalThoughts": 7,
    "totalBranches": 2,
    "mergePoints": [4],
    "averageConfidence": 0.78,
    "unverifiedAssumptions": ["DB index is correct"],
    "openTodos": ["Verify connection pool sizing"],
    "timeline": "T1 → T2(branch-a) + T3(branch-b) → T4(merge) → T5 → T6 → T7"
  }
  ```

#### 4.6 — Update `main.rs` tool dispatch
- Replace the hardcoded `if call_params.name != "sequentialthinking"` check with dynamic registry lookup.
- Update `tools/list` to iterate over all registered tools.

#### 4.7 — Add tests for each new tool

---

## Milestone 5: Thought Quality Scoring & Reasoning Templates
**Version**: `v0.6.0`
**Estimated Effort**: 3–4 days
**Risk**: Low-Medium

### Goal
Add automated quality analysis and pre-built reasoning pattern templates.

### Tasks

#### 5.1 — Create `src/graph/mod.rs`
- Extract `generate_mermaid()` from `server.rs` into `src/graph/mermaid.rs`.
- Create `src/graph/quality.rs` for quality scoring.

#### 5.2 — Implement quality scoring engine
**File**: `src/graph/quality.rs`

Scoring algorithm (compute on each `process_thought` call):
```
quality_score = weighted_average(
    avg_confidence * 0.3,
    verified_ratio * 0.25,        // verified_assumptions / total_assumptions
    branch_merge_ratio * 0.2,     // merged_branches / total_branches
    no_contradictions_bonus * 0.15,
    todos_completion_ratio * 0.1  // completed / total leftToBeDone items
)
```

Return in `ToolResult`:
```rust
pub quality_score: f64,
pub contradictions: Vec<String>,
pub suggested_actions: Vec<String>,  // e.g., "Verify assumption: ..."
```

#### 5.3 — Implement contradiction detection
Compare all `assumptions` against all `verifiedAssumptions` entries across the full history. Flag conflicts where the same assumption appears as both "assumed" and "refuted."

#### 5.4 — Add reasoning templates
**File**: `src/tools/templates.rs`

Expose a `reasoning_templates` tool that returns structured patterns:
```json
{
  "templates": [
    {
      "name": "divide-and-conquer",
      "description": "Split problem → Solve parts independently → Merge solutions",
      "steps": [
        {"step": 1, "action": "Decompose the problem into sub-problems"},
        {"step": 2, "action": "Branch: Solve sub-problem A", "branchId": "sub-a"},
        {"step": 3, "action": "Branch: Solve sub-problem B", "branchId": "sub-b"},
        {"step": 4, "action": "Merge solutions from both branches", "parentThoughts": [2, 3]}
      ]
    },
    {
      "name": "hypothesis-test",
      "description": "Hypothesize → Design verification → Execute → Conclude",
      "steps": [...]
    },
    {
      "name": "devils-advocate",
      "description": "Propose → Counter-argue → Synthesize",
      "steps": [...]
    }
  ]
}
```

---

## Milestone 6 (Future): HTTP Transport & Remote Deployment
**Version**: `v0.7.0` → `v1.0.0`
**Estimated Effort**: 5–7 days
**Risk**: High (new async architecture, session management)

### Goal
Add Streamable HTTP transport alongside stdio. This enables remote deployment, multi-agent access, and serverless hosting.

### Tasks

#### 6.1 — Add HTTP dependencies
```toml
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors"] }
```

#### 6.2 — Create `src/transport/mod.rs`
Define a transport trait:
```rust
pub trait Transport {
    async fn run(self, server: Arc<Mutex<SequentialThinkingServer>>) -> Result<(), Box<dyn Error>>;
}
```

#### 6.3 — Extract stdio transport
Move current stdio loop from `main.rs` into `src/transport/stdio.rs`.

#### 6.4 — Implement Streamable HTTP transport
**File**: `src/transport/http.rs`
- Single endpoint: `POST /mcp` for JSON-RPC requests.
- `GET /mcp` for SSE stream (server-to-client notifications).
- `Mcp-Session-Id` header for session continuity.
- CORS support for browser-based clients.

#### 6.5 — Add `--transport` CLI flag
```rust
#[arg(long, default_value = "stdio", env = "TRANSPORT")]
transport: String, // "stdio" or "http"

#[arg(long, default_value = "3000", env = "PORT")]
port: u16,
```

#### 6.6 — Update Dockerfile for HTTP mode
Add `EXPOSE 3000` and environment variable configuration.

#### 6.7 — Add health check endpoint
`GET /health` returning `{"status": "ok", "version": "0.7.0"}`.

---

## File Impact Summary

| File | M1 | M2 | M3 | M4 | M5 | M6 |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| `Cargo.toml` | ✏️ | ✏️ | ✏️ | — | — | ✏️ |
| `src/main.rs` | ✏️ | ✏️ | ✏️ | ✏️ (major) | — | ✏️ (major) |
| `src/types.rs` | ✏️ | — | ✏️ | — | ✏️ | — |
| `src/server.rs` | ✏️ | ✏️ | ✏️ (major) | ✏️ | ✏️ | ✏️ |
| `src/logging.rs` | — | 🆕 | — | — | — | — |
| `src/persistence/*.rs` | — | — | 🆕 | — | — | — |
| `src/tools/*.rs` | — | — | — | 🆕 | 🆕 | — |
| `src/graph/*.rs` | — | — | — | — | 🆕 | — |
| `src/transport/*.rs` | — | — | — | — | — | 🆕 |
| `Dockerfile` | — | — | — | — | — | ✏️ |

Legend: ✏️ = Modified, 🆕 = New file
