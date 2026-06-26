# 📝 Changelog

All notable changes to the **Sequential Thinking MCP Server (Rust)** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-06-26

This release implements **Milestone 1** of the implementation plan, wiring up previously unused data structures, adding checklist/TODO tracking, introducing automatic reasoning timestamps, and enhancing visual output representations.

### Added
- **Hypothesis & Verification Rendering**: Structured rendering blocks inside `format_thought()` for both `hypothesis` (`🔬 Hypothesis:`) and `verification_method` (`🧪 Verification:`) fields.
- **Mermaid Graph Hypothesis Nodes**: Added a new CSS class Definition (`classDef hypothesis`) highlighting hypothesis-based reasoning blocks in purple (`#d1b3ff`) to visually distinguish them in generated Mermaid diagrams.
- **TODO List (`leftToBeDone`)**: Added `left_to_be_done` array support to `ThoughtData` and `ToolResult` DTOs.
- **TUI TODO Rendering**: In-box rendering of tasks left to be done as yellow checklist items (`📋 TODO: <item>`).
- **Reasoning Timestamps**: Automatic population of thoughts with UTC timestamps using `chrono::Utc::now()` if not provided, displayed cleanly in the terminal thought box headers (e.g., `💭 Thought 1/3 @ 20:25:46`).
- **MCP Schema Expansion**: Exposed `hypothesis`, `verificationMethod`, and `leftToBeDone` in the JSON Schema definition for the `sequentialthinking` tool inside the `tools/list` response.
- **Unit Testing**: Added a new unit test `test_new_fields` in `src/server.rs` ensuring correctness of JSON propagation, timestamp auto-assignment, and todo listings.

### Changed
- Bumped project version to `v0.2.0` in `Cargo.toml`.
- Updated server initialization metadata to report version `0.2.0` in the JSON-RPC response.
- Updated all existing test fixtures in `src/server.rs` to support the new `left_to_be_done` and `timestamp` fields.

---

## [0.1.0] - 2026-06-26

### Added
- Initial Rust implementation of the Sequential Thinking MCP Server.
- Custom stdio JSON-RPC 2.0 message router.
- Core sequential reasoning loop with Graph of Thoughts (GoT) support via branch spawning, revision backtracking, and DAG merges.
- Dynamic terminal width adaptive boxed text formatter.
- Mermaid graph compiler for visualization of thought chains.
- Ported Dockerfile configuration.
