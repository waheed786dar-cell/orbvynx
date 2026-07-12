Contributing to ORBVYNX
Early-stage project — expect rough edges and architectural shifts as real usage exposes design gaps.
Before you start
Read docs/ first. Each crate maps to a specific architecture part (Kernel, Intent, Planner, Workflow, Executor, Security, Plugin Runtime, SDK). Boundary rules matter for review speed:
The Kernel never contains business logic.
The Planner never executes anything — it only produces Plans.
The Workflow Engine never executes — it only organizes execution into a graph.
The Executor never makes planning decisions.
Check open issues before starting significant work.
Development setup
git clone https://github.com/waheed786dar-cell/orbvynx.git
cd orbvynx
cargo build --workspace
cargo test --workspace
No rustup needed — system Rust (1.85+) is enough. On Termux: pkg install rust, not the rustup script.
Making changes
Fork, branch from main.
Keep changes scoped to one crate/concern.
Every new module needs unit tests. Tests spawning real subprocesses or touching the network must be #[ignore]'d with a reason, so cargo test --workspace stays fast and hang-free.
cargo build --workspace && cargo test --workspace must pass cleanly before opening a PR.
Follow the existing error pattern: each crate has its own thiserror enum, wrapping lower-level errors via #[error(transparent)] #[from] rather than stringly-typed errors.
Adding a Capability
Built-in capabilities live in crates/executor/src/capabilities/. Implement the Capability trait (or use the orbvynx-sdk capability! macro), add tests, register it in the CLI's CapabilityRegistry.
Adding a Plugin
No source changes needed — see the Plugin section in the README. General-purpose plugins worth sharing can go under examples/plugins/ with a short description in the PR.
Reporting bugs / requesting features
Use the templates under .github/ISSUE_TEMPLATE/. Include exact command run, expected vs actual behavior, and error output.
Code of conduct
Be respectful. Disagree about architecture freely — argue the idea, not the person.
