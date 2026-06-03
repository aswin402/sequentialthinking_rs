# AI Agent Setup Guide: Sequential Thinking MCP Server

This document contains copy-pasteable instructions for AI coding assistants and autonomous agents (like Claude, cursor, or OpenZ) to automatically download, build, and configure the Sequential Thinking MCP Server.

---

## 🤖 Instructions for the AI Agent

Please execute the following steps to configure the Rust Sequential Thinking MCP Server in this environment:

### Step 1: Clone the Repository
Clone the repository to the local environment and enter the directory:
```bash
git clone https://github.com/aswin402/sequentialthinking_rs.git
cd sequentialthinking_rs
```

### Step 2: Build the Release Binary
Compile the optimized release binary using Cargo:
```bash
cargo build --release
```
*The compiled binary will be placed at `./target/release/mcp-server-sequential-thinking`.*

### Step 3: Register the Server
Add the following configuration to the MCP host settings file (e.g., `mcp_config.json`, `claude_desktop_config.json`, or `.vscode/mcp.json`):

```json
{
  "mcpServers": {
    "sequential-thinking": {
      "command": "/absolute/path/to/sequentialthinking_rs/target/release/mcp-server-sequential-thinking",
      "args": []
    }
  }
}
```
*Note: Replace `/absolute/path/to/sequentialthinking_rs` with the actual absolute path to the cloned directory.*

### Step 4: Verify the Installation
Run the binary with the `--help` flag to confirm it launches and executes cleanly:
```bash
./target/release/mcp-server-sequential-thinking --help
```

### Step 5: How to Use
Use the `sequentialthinking` tool for multi-step, complex logical reasoning, utilizing advanced Graph of Thoughts parameters such as `parentThoughts` (to merge reasoning paths) and `confidenceScore`.
