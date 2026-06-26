<p align="center">
  <img src="docs/assets/logo.png" alt="Sequential Thinking MCP Logo" width="300" />
</p>

<h1 align="center">💭 Sequential Thinking MCP Server (Rust)</h1>

<p align="center">
  <strong>A high-performance, persistent, graph-structured reasoning platform for LLMs and AI agents. Features Directed Acyclic Graph (DAG) merging, multi-session SQLite state storage, quality diagnostics, templates, and dual Stdio/HTTP/SSE transports.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-edition%202024-orange?logo=rust" alt="Rust Edition 2024" /></a>
  <a href="https://github.com/aswin402/sequentialthinking_rs"><img src="https://img.shields.io/badge/version-1.0.0-green" alt="Version 1.0.0" /></a>
</p>

---

## 📖 Overview & Core Philosophy

This server is a production-grade rewrite and extension of the official [Model Context Protocol (MCP) Sequential Thinking Server](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking). 

While traditional implementations support only linear sequences, this server implements a **Graph of Thoughts (GoT)** DAG engine enabling agents to explore alternative branches, backtrack, and merge reasoning paths. Additionally, it integrates a multi-session **SQLite persistence layer**, a **thought quality scoring engine** (detecting cycles and contradictions), **reasoning templates**, and **HTTP/SSE transport** for remote server deployment.

---

## ⚡ TS vs. Rust Comparison

| Feature | Original Node.js/TS Server | Enhanced Rust Server (v1.0.0) |
| :--- | :--- | :--- |
| **Cold-start Latency** | $\sim 150 - 200\text{ ms}$ | **$< 1\text{ ms}$** (compiled binary) |
| **Memory Footprint** | $\sim 30 - 50\text{ MB}$ | **$< 4\text{ MB}$** |
| **Reasoning Model** | Linear Chain + Single Branching | **Directed Acyclic Graph (DAG)** via GoT merging |
| **Persistence** | In-Memory (lost on restart) | **SQLite Database** & memory backends |
| **Multi-session** | None (global shared state) | **Multi-session isolation** via `sessionId` |
| **Tool Registry** | Single Tool (`sequentialthinking`) | **5-tool reasoning ecosystem** |
| **Transport** | Stdio Only | **Stdio & Server-Sent Events (HTTP/SSE)** |
| **Thought Quality** | None | Cycle/contradiction check + scoring (0-100) |

---

## 🛠️ The 5-Tool Reasoning Suite

### 1. `sequentialthinking`
The core reasoning loop. Allows agents to submit thoughts, confidence levels, hypotheses, verification methods, and dependencies.
- **Key Inputs**:
  - `thought` (string, required): Current thought step.
  - `thoughtNumber` (integer, required): Step number (starts at 1).
  - `totalThoughts` (integer, required): Current estimate of thoughts needed.
  - `nextThoughtNeeded` (boolean, required): Whether to continue thinking.
  - `sessionId` (string, optional): Target session identifier (auto-generated if omitted).
  - `parentThoughts` (array of integers, optional): Merge target thought IDs.
  - `assumptions` / `verifiedAssumptions` (array of strings, optional): Context assertions.
  - `confidenceScore` (number, optional): Path confidence (0.0 to 1.0).
  - `isRevision` / `revisesThought` (boolean/integer, optional): Backtracking/correction.

### 2. `analyze_graph`
Inspects the thought history for a session to run diagnostic checks without adding thoughts.
- **Queries**:
  - `low_confidence`: Lists thoughts below a `confidenceThreshold` (default `0.5`).
  - `contradictions`: Checks if assumptions are later marked as `refuted` or `false`.
  - `unverified_assumptions`: Lists assumptions not yet marked verified.
  - `dead_branches`: Identifies branches disconnected from the final conclusion.
  - `summary_stats`: Returns basic session metrics.
  - `quality_report`: Computes the full quality diagnostics.

### 3. `reasoning_templates`
Returns structured thinking frameworks to guide complex operations:
- `divide-and-conquer`: For breaking down and merging sub-problems.
- `hypothesis-test`: For root-cause debugging and scientific troubleshooting.
- `devils-advocate`: For challenging biases, checking edge cases, and hardening ideas.

### 4. `export_session`
Serializes and outputs the reasoning session history.
- **Formats**: `mermaid` (interactive graph visualization), `markdown` (formatted document), `json` (nodes & edges representation), or `dot` (Graphviz layout).

### 5. `summarize_reasoning`
Generates a structured text timeline summary of the thought progression, highlighting major branches and merges.

---

## ⚙️ Configuration & CLI Options

Run the binary with `--help` to inspect all options:
```bash
mcp-server-sequential-thinking [OPTIONS]
```

### Command Line Flags:
- `--transport <TRANSPORT>`: Set communication backend (`stdio` or `http`) [default: `stdio`] [env: `TRANSPORT`].
- `-p, --port <PORT>`: Port to listen on in HTTP mode [default: `3000`] [env: `PORT`].
- `--storage <STORAGE>`: Storage backend (`memory` or `sqlite`) [default: `memory`] [env: `STORAGE`].
- `--db-path <DB_PATH>`: Path to SQLite database file [env: `DB_PATH`]. Defaults to `~/.sequentialthinking/history.db` if `sqlite` is chosen.
- `--log-format <LOG_FORMAT>`: Logging format for standard error (`pretty` or `json`) [default: `pretty`] [env: `LOG_FORMAT`].
- `-d, --disable-thought-logging`: Disable outputting terminal thought boxes to `stderr` [env: `DISABLE_THOUGHT_LOGGING`].

---

## 🌐 HTTP/SSE Transport Mode

In `http` transport mode, the server opens a standard SSE listener on `/sse` and a POST handler on `/message`.

### 1. Connecting (GET `/sse`)
Establishing a persistent stream generates a new session and returns a routing URL:
```bash
curl -N http://localhost:3000/sse
```
**Output**:
```
event: endpoint
data: /message?sessionId=8139c0bc-8a6a-4462-aef0-4545f2b0703e
```

### 2. Posting Requests (POST `/message`)
Send JSON-RPC payloads to the message route. The server processes the request and streams the responses back over the active `/sse` channel:
```bash
curl -i -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}' \
  "http://localhost:3000/message?sessionId=8139c0bc-8a6a-4462-aef0-4545f2b0703e"
```

### 3. Monitoring Health
The `/health` endpoint is available for health checks:
```bash
curl http://localhost:3000/health
```

---

## 📦 Container Deployment (Docker)

To run the server inside Docker (ideal for remote setups):

### 1. Build Image
```bash
docker build -t sequential-thinking-mcp .
```

### 2. Run Container
Mount a local directory to preserve the SQLite history database:
```bash
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/data \
  sequential-thinking-mcp
```

---

## 🔌 Editor Integration

### Claude Desktop
Add the server config to `claude_desktop_config.json`:

**Stdio Mode:**
```json
{
  "mcpServers": {
    "sequential-thinking": {
      "command": "/absolute/path/to/target/release/mcp-server-sequential-thinking",
      "args": ["--storage", "sqlite"]
    }
  }
}
```

**SSE/HTTP Mode:**
```json
{
  "mcpServers": {
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/inspector", "http://localhost:3000/sse"]
    }
  }
}
```

### Cursor / VSCode MCP
Configure a new command-line MCP server in Cursor settings:
- **Name**: `sequential-thinking`
- **Type**: `command`
- **Command**: `/path/to/mcp-server-sequential-thinking --storage sqlite`

---

## 📚 Reference Docs
- [Architecture Overview](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/docs/architecture.md)
- [Code Structure & Layout](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/docs/code_structure.md)
- [GoT & Clear Thought In-Depth Guide](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/docs/got_and_clear_thought.md)
- [Agent Setup Guide](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/docs/install_instruction.md)

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/sequentialthinking_rs/LICENSE) file for details.
