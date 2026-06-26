# Product Requirements Document (PRD)
## Sequential Thinking MCP Server (Rust)

---

### 1. Executive Summary

The **Sequential Thinking MCP Server (Rust)** is a high-performance, native implementation of the Model Context Protocol (MCP) sequential thinking server. While the original TypeScript implementation operates as a linear chain of thoughts, this Rust rewrite introduces a next-generation **Graph of Thoughts (GoT)** reasoning model, **Clear Thought** metacognition parameters, and highly optimized, low-latency execution.

By leveraging Rust, the server achieves near-instantaneous cold starts ($< 1\text{ ms}$) and an extremely small memory footprint ($< 4\text{ MB}$), making it ideal for local runtimes, IDE integrations, and resource-constrained environments.

---

### 2. Product Objectives

*   **Performance Optimization**: Deliver native, compile-time performance to eliminate V8 startup overhead in agent tool execution.
*   **Next-Gen Reasoning (GoT)**: Support non-linear reasoning paths including parallel branching, revision histories, and merging concepts into Directed Acyclic Graphs (DAGs).
*   **Metacognition Support (Clear Thought)**: Provide parameters for tracking confidence, assumptions, verifications, and self-criticism.
*   **Developer Observability**: Render responsive, colored ASCII boxes in the console (`stderr`) and compile live Mermaid graphs of the thought process for real-time visualization.

---

### 3. User Personas & Use Cases

| Persona | Description | Use Case |
| :--- | :--- | :--- |
| **AI Agents / LLMs** | Client systems that consume tools to solve complex, multi-step tasks. | Evaluates codebases, debugs systems, and writes code using multi-branch hypotheses. |
| **Software Developers** | End users executing agents in their IDEs (e.g., VS Code, Claude Desktop, Cursor). | Observes the agent's internal thought progression, debugs reasoning flaws, and tracks assumptions. |
| **System Administrators** | Teams deploying and monitoring agent runtimes at scale. | Deploys containerized or native binaries with minimum resource footprints. |

---

### 4. Technical Architecture & Protocols

#### 4.1 JSON-RPC over Stdio Transport
The server complies with the standard **Model Context Protocol (MCP)** specification:
*   **Transport Channels**: Uses `stdin`/`stdout` for JSON-RPC 2.0 communication.
*   **Separation of Concerns**: Standard output (`stdout`) is strictly reserved for JSON-RPC message payloads. All logs, diagnostic messages, and TUI boxes must be printed to standard error (`stderr`) to prevent protocol corruption.

#### 4.2 Lifecycle Protocol Handshake
1.  **Initialize**: Handshake is initiated by the client. The server responds with `protocolVersion: "2024-11-05"`.
2.  **Initialized Notification**: Client confirms registration.
3.  **List Tools**: The server advertises the `sequentialthinking` tool schema.
4.  **Call Tool**: Client passes thought details; server updates internal state, prints the reasoning frame to `stderr`, and returns progress metrics to `stdout`.

---

### 5. Functional Requirements

#### 5.1 Tool Schema: `sequentialthinking`

The server exposes a single tool called `sequentialthinking` with the following parameters:

##### A. Inbound Parameters (Schema)
*   `thought` (string, required): The reasoning content, analysis, or conclusion.
*   `thoughtNumber` (integer, required): The current index in the thinking process (starts at 1).
*   `totalThoughts` (integer, required): Current estimate of total thoughts needed.
*   `nextThoughtNeeded` (boolean, required): Indicates if the reasoning loop should continue.
*   `isRevision` (boolean, optional): Marks this thought as a correction of a previous step.
*   `revisesThought` (integer, optional): The thought index being corrected.
*   `branchFromThought` (integer, optional): The thought index from which this branch departs.
*   `branchId` (string, optional): Unique ID for the branch.
*   `parentThoughts` (array of integers, optional): Indices of parent thoughts to merge paths (enables GoT DAG).
*   `assumptions` (array of strings, optional): Explicit assumptions relied upon.
*   `verifiedAssumptions` (array of strings, optional): Refuted/validated assumptions from prior steps.
*   `confidenceScore` (number, optional): Certainty index between `0.0` and `1.0`.
*   `criticism` (string, optional): Self-criticism or evaluation of earlier steps.

##### B. Outbound Response
Returns a JSON object containing:
*   `thoughtNumber` (integer): Confirmed current thought index.
*   `totalThoughts` (integer): Auto-adjusted total thoughts estimate.
*   `nextThoughtNeeded` (boolean): Whether another thought is expected.
*   `branches` (array of strings): List of all active branch IDs.
*   `thoughtHistoryLength` (integer): Total nodes in the thought history.
*   `thoughtGraphMermaid` (string): Complied Mermaid syntax representing the thought DAG.
*   `confidenceHistory` (array of confidence scores): Historical record of confidence parameters.

#### 5.2 State Management & Graph Generation
*   **In-Memory Adjacency List**: Maintain an internal representation of nodes and directed edges.
*   **Auto-Correction**: Automatically adjust `totalThoughts` if the current `thoughtNumber` exceeds the previous estimate.
*   **Mermaid Output**: Construct a valid `graph TD` Mermaid diagram reflecting standard nodes (blue), revisions (yellow), branches (green), and parent links.

#### 5.3 Responsive Stderr TUI
*   **Dynamic Column Resizing**: Fetch terminal width on each thought.
*   **Clamping Bounds**: Constraint rendering box width between 40 and 100 columns.
*   **ANSI Formatting**: Color-code thought boxes by state (blue for thoughts, yellow for revisions, green for branches).
*   **Metacognition Displays**: Render confidence scores as star gauges (`[★★★★☆] 80%`) and highlight assumptions, verifications, and criticism using distinct prefix icons.

---

### 6. Non-Functional Requirements

*   **Ultra-Low Latency**: Cold-start latency must be under $1\text{ ms}$. Tool processing execution should complete in $< 0.1\text{ ms}$.
*   **Minimal Resource Footprint**: Memory usage under active load must stay below $4\text{ MB}$.
*   **Static Compilation**: Zero external library dependencies required at runtime. The build target must be a single portable binary.
*   **Safety & Stability**: Implemented in memory-safe, crash-resistant Rust with robust JSON deserialization and bounds validation.

---

### 7. Product Roadmap & Enhancements

#### Phase 1: Persistence Layer (SQLite)
*   Introduce `rusqlite`/`sqlx` to save thoughts to `~/.sequentialthinking/history.db`.
*   Support `sessionId` parameter to retrieve and extend past thinking graphs across agent restarts.

#### Phase 2: Collaborative Multi-Agent Reasoning
*   Support batch submissions via a `batch_thoughts` protocol extension to allow multiple parallel agents to log concurrent ideas.

#### Phase 3: Interactive TUI Graph Visualizer
*   Add a standalone CLI mode (e.g., `sequentialthinking --visualize <session-id>`) that renders the DAG within a terminal GUI using unicode box-drawing characters.
