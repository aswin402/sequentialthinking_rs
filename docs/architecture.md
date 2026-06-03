# Architecture Overview: Sequential Thinking MCP Server

This document outlines the architecture, data models, and transport protocols used in the Rust implementation of the Sequential Thinking MCP Server.

---

## 1. System Components

The server is built as a single, decoupled native Rust application. The flow of data is linear and event-driven:

```
[MCP Host CLI/TUI] (Claude, OpenZ)
      │  (stdin: JSON-RPC 2.0)
      ▼
┌──────────────┐      ┌─────────────────────────┐
│  src/main.rs │ ───> │       src/types.rs      │ (Zod-equivalent JSON Deserialization)
│  (Transport) │      │  (Data models & Schemas)│
└──────────────┘      └─────────────────────────┘
      │
      ▼ (ThoughtData structure)
┌──────────────┐      ┌─────────────────────────┐
│src/server.rs │ ───> │     colored / textwrap  │ (Responsive Bordered Output)
│ (State / GoT)│      │    (Stderr TUI Engine)  │ ───> Terminal stderr
└──────────────┘      └─────────────────────────┘
      │
      ▼ (ToolResult structure)
[MCP Host CLI/TUI] (stdout: JSON-RPC Response)
```

---

## 2. JSON-RPC over Stdio Transport

The Model Context Protocol communicates via **JSON-RPC 2.0** over standard input/output (`stdin`/`stdout`):
*   **Inbound requests** are read line-by-line from `stdin`. Each line contains a complete JSON string.
*   **Outbound responses** are written line-by-line to `stdout`, followed by a newline (`\n`) and flushed immediately (`io::stdout().flush()`).
*   **Logs and visual formatting** are written exclusively to `stderr`. Standard output is strictly reserved for JSON-RPC messages; writing any text to `stdout` will break the MCP protocol parsing on the host side.

### Protocol Handshake Lifecycle
1.  **Initialize Request** (`initialize`): Host sends client information. The server responds with its protocol version (`2024-11-05`) and tool capabilities.
2.  **Initialized Notification** (`notifications/initialized`): Host acknowledges setup.
3.  **List Tools Request** (`tools/list`): Host queries available tools. The server responds with the `sequentialthinking` tool configuration and input schemas.
4.  **Call Tool Request** (`tools/call`): Host executes a reasoning thought. The server processes it, prints visual boxes to `stderr`, and returns the history state on `stdout`.

---

## 3. Graph of Thoughts (GoT) Architecture

Traditional Chain-of-Thought (CoT) models operate on a linear sequence of thoughts:
$$\text{Thought 1} \rightarrow \text{Thought 2} \rightarrow \text{Thought 3}$$

In contrast, our **Graph of Thoughts (GoT)** representation permits arbitrary **branching** and **merging** operations, creating a Directed Acyclic Graph (DAG):

*   **Branching**: Explicitly handled by the `branchFromThought` and `branchId` fields.
*   **Merging**: Handled by the `parentThoughts` array, which links a single new node back to multiple preceding thought nodes.
*   **Mermaid Representation**: The server maintains a complete adjacency list of thoughts in memory. On every step, it resolves node classifications (Standard, Branch, Revision) and compiles a standard Mermaid diagram representation. The host or TUI can display this diagram to let developers track the reasoning flow visually.

---

## 4. Responsive TUI Rendering

To provide a premium developer experience, the server formats thoughts inside bordered ASCII panels:
1.  **Query dimensions**: The `terminal_size` crate is queried on every step to get the current width of the user's terminal.
2.  **Width boundary constraints**: The box width is bound between a minimum of 40 columns (for small split views) and a maximum of 100 columns (to prevent extremely long, unreadable single lines on wide monitors).
3.  **Wrapping**: The `textwrap` library wraps the thought content and metadata lines to fit perfectly inside the computed width constraints.
4.  **Color escapes**: Colored tags (e.g. green for branches, yellow for revisions) are applied via ANSI escape sequences using the `colored` crate.
