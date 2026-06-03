# Graph of Thoughts & Clear Thought Guide

This guide details how the advanced **Graph of Thoughts (GoT)** and **Clear Thought** parameters work, how they are structured, and how LLM hosts should invoke them.

---

## 1. What is Graph of Thoughts (GoT)?

Traditional LLM thinking operates strictly in a linear progression. If the model makes an error at step 2, it must either backtrack completely or carry the flawed assumption forward.

**Graph of Thoughts (GoT)** models reasoning as a Directed Acyclic Graph (DAG). This allows the agent to:
1.  Explore parallel ideas simultaneously (Branching).
2.  Refute weak branches and keep strong ones (Revising / Criticizing).
3.  Combine insights from multiple branches into a unified solution (Merging).

---

## 2. Using the Schema Parameters

### A. Branching & Revisions
*   **Branching**: To explore a separate path, set `branchFromThought` to the parent node number, and provide a unique `branchId`.
*   **Revisions**: If a realization shows a previous thought was flawed, set `isRevision` to `true` and reference the target in `revisesThought`.

### B. GoT Merging (`parentThoughts`)
When combining multiple lines of thinking, specify the parent nodes as an array of integers in `parentThoughts`:
```json
"parentThoughts": [2, 3]
```
This instructs the graph parser that the new thought is a merge node receiving edges from both nodes 2 and 3.

### C. Clear Thought Parameters
These parameters enforce structured self-evaluation during the reasoning process:
*   **`confidenceScore`** (float, `0.0` to `1.0`): The agent rates its certainty in the current step. In the TUI, this renders as a star rating (e.g. `[★★★★☆] 80%`).
*   **`assumptions`** (array of strings): Exposes what assumptions are being relied on.
*   **`verifiedAssumptions`** (array of strings): References previous assumptions and verifies or refutes them based on newer findings.
*   **`criticism`** (string): Allows the agent to explicitly log criticisms about past steps, encouraging self-correction.

---

## 3. Example JSON-RPC Payload Sequence

Below is a sequence of `tools/call` JSON payloads representing a complete GoT and Clear Thought reasoning cycle:

### Thought 1: Initial Linear Thought (Setup & Assumptions)
```json
{
  "jsonrpc": "2.0",
  "id": 101,
  "method": "tools/call",
  "params": {
    "name": "sequentialthinking",
    "arguments": {
      "thought": "Let's investigate the performance bottleneck in database queries.",
      "thoughtNumber": 1,
      "totalThoughts": 4,
      "nextThoughtNeeded": true,
      "assumptions": ["The database index is correctly configured on user_id"]
    }
  }
}
```

### Thought 2: Branch A (Re-evaluating Indexes)
```json
{
  "jsonrpc": "2.0",
  "id": 102,
  "method": "tools/call",
  "params": {
    "name": "sequentialthinking",
    "arguments": {
      "thought": "Exploring Index Performance. Let's check query plan outputs.",
      "thoughtNumber": 2,
      "totalThoughts": 4,
      "nextThoughtNeeded": true,
      "branchFromThought": 1,
      "branchId": "index-path",
      "confidenceScore": 0.9
    }
  }
}
```

### Thought 3: Branch B (Checking Connection Pool)
```json
{
  "jsonrpc": "2.0",
  "id": 103,
  "method": "tools/call",
  "params": {
    "name": "sequentialthinking",
    "arguments": {
      "thought": "Let's explore connection pool saturation as a parallel hypothesis.",
      "thoughtNumber": 3,
      "totalThoughts": 4,
      "nextThoughtNeeded": true,
      "branchFromThought": 1,
      "branchId": "pool-path",
      "confidenceScore": 0.65
    }
  }
}
```

### Thought 4: Merge Node (Combining Results)
The index check is clean, but the connection pool is saturated. We merge both paths:
```json
{
  "jsonrpc": "2.0",
  "id": 104,
  "method": "tools/call",
  "params": {
    "name": "sequentialthinking",
    "arguments": {
      "thought": "Conclusion: Index checks are green. Saturated connection pool verified as the cause. Resolution: Increase pool size to 50.",
      "thoughtNumber": 4,
      "totalThoughts": 4,
      "nextThoughtNeeded": false,
      "parentThoughts": [2, 3],
      "verifiedAssumptions": ["Connection limits are indeed saturated"]
    }
  }
}
```
---

## 4. Visual Rendering in the TUI
When the above sequence runs, the server will output styled panels on `stderr`:
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 💭 Thought 4/4 (Merge from T2, T3)                                            │
├──────────────────────────────────────────────────────────────────────────────┐
│  ✅ Verified: Connection limits are indeed saturated                         │
├──────────────────────────────────────────────────────────────────────────────┐
│ Conclusion: Index checks are green. Saturated connection pool                │
│ verified as the cause. Resolution: Increase pool size to 50.                 │
└──────────────────────────────────────────────────────────────────────────────┘
```
This ensures developers can track exactly how the model traversed the reasoning tree.
