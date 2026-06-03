# Code Structure Guide: Sequential Thinking MCP Server

This document explains the organization of files, modules, and classes within the Rust project.

---

## 1. Directory Layout

The project follows standard Cargo structure:

```
sequentialthinking_rs/
├── Cargo.toml            # Project dependencies and configuration
├── Dockerfile            # Container build recipe (Rust multi-stage)
├── README.md             # Project documentation and guide
├── docs/                 # Detailed architectural guides
│   ├── architecture.md
│   ├── code_structure.md
│   └── got_and_clear_thought.md
└── src/                  # Source files
    ├── main.rs           # Protocol transport & CLI entrypoint
    ├── server.rs         # State engine, formatting, and graph compiler
    └── types.rs          # JSON-RPC & MCP struct definitions
```

---

## 2. File In-Depth Details

### 1. `src/types.rs`
Contains all serializable data transfer objects (DTOs) parsed via `serde`:
*   `JsonRpcRequest` / `JsonRpcResponse`: Standard JSON-RPC packet envelopes.
*   `CallToolParams`: Expected parameters when invoking a tool call.
*   `ThoughtData`: The structure capturing all parameters passed by the LLM when thinking, including the new enhanced fields like `parent_thoughts`, `confidence_score`, `assumptions`, `verified_assumptions`, and `criticism`.
*   `ToolCallResponse`: Encapsulates the output text content payload.
*   `ToolResult`: The structured JSON content returned inside `ToolCallResponse`.

### 2. `src/server.rs`
Contains the core state manager `SequentialThinkingServer`:
*   **State Containers**:
    *   `thought_history`: `Vec<ThoughtData>` stores every thought step in order.
    *   `branches`: `HashMap<String, Vec<ThoughtData>>` groups branch thoughts by their ID.
*   **Key Functions**:
    *   `get_terminal_width`: Computes responsive column bounds based on query metrics.
    *   `format_thought`: Formats and builds the colored visual bordered boxes printed to `stderr`. It integrates the metadata (stars for confidence, markers for assumptions/verifications).
    *   `generate_mermaid`: Analyzes the thought sequence and compiles a valid Mermaid diagram string, mapping paths, branches, revisions, and GoT parent merges.
    *   `process_thought`: Handles sequence boundary checking, appends states, logs to `stderr`, and packages the execution stats.

### 3. `src/main.rs`
Initializes the runtime and coordinates standard input streams:
*   Uses `clap` to process CLI options.
*   Runs an asynchronous loop (`tokio::io::stdin()`) line-by-line using `BufReader`.
*   Validates incoming JSON strings, routes requests, and serializes output results to `stdout`.

---

## 3. Extending the Code

### How to Add a New Visual Metadata Field
If you want to track a new reasoning parameter (e.g. `risks` or `mitigations` array):
1.  **Add to DTO**: In `src/types.rs`, add the new optional field to `ThoughtData`:
    ```rust
    pub risks: Option<Vec<String>>,
    ```
2.  **Add to Schema Advertising**: In `src/main.rs`, update the JSON input schema advertised in `tools/list` to include the property description.
3.  **Render in Box**: In `src/server.rs` (`format_thought`), parse the option and format it into the TUI output:
    ```rust
    if let Some(ref risks) = thought_data.risks {
        for risk in risks {
            let line = format!(" ⚠️ Risk: {}", risk);
            // Wrap and push to lines array...
        }
    }
    ```
4.  **Verify**: Recompile with `cargo build` and test using the verification script.
