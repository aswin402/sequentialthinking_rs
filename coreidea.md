# Core Idea & Philosophy: GoT & Clear Thought

---

### 1. Conceptual Background & Evolution

Modern artificial intelligence relies heavily on **Chain-of-Thought (CoT)** prompting to solve complex problems. By breaking a problem down into sequential steps, models generate better reasoning. However, standard CoT has a fatal flaw: it is **purely linear**. If a model makes a bad assumption at step 2, it is stuck with it for the rest of the chain unless it backtracks completely, which is computationally expensive and difficult for state-based agents.

```
Linear Chain-of-Thought (CoT):
[Thought 1] ──> [Thought 2 (Flawed)] ──> [Thought 3 (Corrupted)] ──> [Incorrect Output]
```

To solve this, advanced paradigms like **Tree of Thoughts (ToT)** introduced branching, allowing the agent to evaluate multiple paths. However, real-world reasoning is rarely a tree; it often requires combining ideas from different branches. 

This server implements **Graph of Thoughts (GoT)**, modeling reasoning as a Directed Acyclic Graph (DAG). GoT allows:
1.  **Divergent thinking (Branching)**: Exploring multiple parallel hypotheses.
2.  **Self-Correction (Revising/Criticizing)**: Re-evaluating and correcting previous steps.
3.  **Convergent thinking (Merging)**: Combining the green insights from separate branches into a single consolidated conclusion.

```
Graph of Thoughts (GoT) DAG:
                 ┌──> [Branch A (Replication Check)] ──┐
[Thought 1] ────┤                                     ├──> [Merge Thought 4 (pg_upgrade with --link)]
                 └──> [Branch B (pg_upgrade Check)] ──┘
```

---

### 2. The "Clear Thought" Philosophy

A major source of AI hallucination is the failure to externalize and critique meta-cognition. Models often mix assumptions with proven facts, fail to track their own level of certainty, and ignore their own internal contradictions.

The **Clear Thought** paradigm introduces explicit structure to force the agent to reflect on its own thinking on every tool call. It separates reasoning into concrete fields:
*   **Confidence Scores**: Forces the model to assign a probability metric to its line of reasoning (0.0 to 1.0). This prevents low-confidence reasoning paths from masquerading as absolute truth.
*   **Assumptions Tracking**: Explicitly listing what the model is assuming to be true.
*   **Assumption Verification**: Verifying or refuting assumptions actively, converting assumptions into validated facts or disproved hypotheses.
*   **Self-Criticism**: A dedicated channel for the model to critique its own prior thoughts, exposing logical leaps, circular reasoning, or unverified claims.

By defining these metrics in the tool schema, the host forces the model to fill them out, converting implicit, unstructured thoughts into structured, verifiable data.

---

### 3. Performance & System Design Philosophy

The original TypeScript Sequential Thinking server is excellent for prototype agents, but has several drawbacks in production:
*   **V8 Startup Overhead**: Every time an agent tool runs, Node has to initialize the V8 engine and load a large tree of NPM dependencies. This adds $150-200\text{ ms}$ of cold-start latency per step.
*   **Memory Footprint**: Node processes require $30-50\text{ MB}$ of resident set size (RSS), which is highly inefficient when running multiple agent instances concurrently.
*   **Dependency Bloat**: Managing NPM packages (`node_modules`) is prone to version conflicts, vulnerabilities, and installation issues on client machines.

#### The Rust Approach
This rewrite is built on a "native systems" philosophy:
1.  **Zero-overhead Execution**: Written in native Rust, compiling down to a single optimized binary. Cold-start latency drops to **$< 1\text{ ms}$**, and memory footprint is reduced to **$< 4\text{ MB}$**.
2.  **No Dependency Bloat**: No runtime interpreter or virtual machine is required. The binary runs directly on the host system.
3.  **Standard Stream Separation**: All JSON-RPC messages flow through `stdin`/`stdout`, while TUI formatting flows through `stderr`. This ensures 100% standard compatibility with MCP clients (like Claude Desktop and OpenZ) while providing developers with full terminal diagnostics.

---

### 4. Human Observability (The TUI & Mermaid Compiler)

Reasoning should not be a black box. For developers using AI agents, it is critical to see how the agent arrived at its conclusion. 

The server provides two levels of observability:
*   **Responsive Terminal Border Box Panel**: The server queries the client terminal's dimensions on each step, wraps the text, and renders a colored box panel on `stderr`. Revisions are highlighted in yellow, branches in green, and standard thoughts in blue. Metacognitive metadata (like confidence and assumptions) is rendered using distinct emojis and star gauges (`[★★★★☆] 80%`).
*   **Mermaid DAG Generator**: On every tool response, the server returns a fully compiled Mermaid syntax string (`thoughtGraphMermaid`). The host application can render this graph live, giving developers a visual representation of how branches were explored, revised, and merged.
