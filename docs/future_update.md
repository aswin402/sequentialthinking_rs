# Future Roadmap & Enhancements: Sequential Thinking MCP

This document tracks potential future improvements to the Rust Sequential Thinking server. We will continue using the current in-memory implementation, verify its long-term stability in active agent workflows, and prioritize these features when updates are needed.

---

## 📅 Planned Features & Enhancements

### 1. Persistent Database Storage (SQLite)
*   **Goal**: Persist thoughts across restarts and session boundaries.
*   **Implementation**: Use a lightweight in-process database like `rusqlite` or `sqlx` storing thoughts in a local SQLite file (`~/.sequentialthinking/history.db`).
*   **Parameters to add**:
    *   `sessionId` (string, optional): Links thoughts to a specific session, allowing the agent to resume past reasoning chains.

### 2. Multi-Agent Batch Submissions (Parallelism)
*   **Goal**: Allow multiple collaborative agents to submit thoughts simultaneously.
*   **Implementation**: Add a batch RPC method (`batch_thoughts`) supporting bulk insertions.
*   **Use Case**: A researcher agent and developer agent posting parallel thought nodes which are then merged.

### 3. TUI Graph Visualizer
*   **Goal**: Provide an interactive visual representation of the Graph of Thoughts in the command line.
*   **Implementation**: Add a CLI command (e.g. `visualize --session-id <id>`) rendering the thoughts DAG using terminal graphics or standard unicode boxes.

### 4. Integration with OpenZ Skill Builder
*   **Goal**: Save completed, high-confidence thought graphs directly to the agent's long-term skill database.
*   **Implementation**: Create an export routine to serialize successful reasoning graphs as Toml/Markdown playbooks in `~/.zeroclaw/workspace/skills/`.
