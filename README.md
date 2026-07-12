# ORBVYNX

**An Intent Operating Layer** — turns human goals into deterministic, observable, replayable execution.Intent -> Plan -> Workflow -> Execution -> Verification
## Status

**Early-stage / experimental.** Core pipeline (Intent -> Planner -> Workflow -> Executor) is implemented and tested. Not yet hardened for untrusted input or production infrastructure — read Safety Notes below first.

Built entirely on Termux (Android), no PC used.

## Architecture

Microkernel-based, event-driven, capability-based system written in Rust:

| Crate | Purpose |
|---|---|
| orbvynx-kernel | Boot sequence, async event bus, module/service registries, lifecycle state machine |
| orbvynx-intent | Captures and validates user goals into structured Intents |
| orbvynx-planner | Turns a validated Intent into a risk-scored, cost-estimated Plan |
| orbvynx-workflow | Converts a Plan into a dependency-checked task graph (DAG) |
| orbvynx-executor | Runs tasks via pluggable Capabilities, with timeouts and sandboxing |
| orbvynx-security | Permission store, policy engine, permission dispatcher |
| orbvynx-plugin-runtime | Loads third-party subprocess-based plugins dynamically |
| orbvynx-sdk | Helper macros for writing new Capabilities |
| apps/cli | The orbvynx command-line entry point |

Full design rationale is in docs/.

## Quick Start

Requires Rust (2021 edition) and Git.

```bash
git clone https://github.com/waheed786dar-cell/orbvynx.git
cd orbvynx
cargo build --workspace
cargo test --workspace
cargo run -p orbvynx-cli -- git status
On Termux: pkg install rust git — do not use the rustup install script, it fails on Android.
Writing a Capability
use orbvynx_sdk::*;

capability!(GreetCapability, "example.greet", |input| {
    let name = param_str(&input, "name")?;
    ok(json!({ "greeting": format!("Hello, {name}!") }))
});
Writing a Plugin
Any executable understanding two flags becomes a dynamically-loaded plugin — no ORBVYNX recompilation needed:
--orbvynx-manifest — print a JSON manifest (name, version, description, capability_name), exit
--orbvynx-invoke — read JSON from stdin, write JSON to stdout
See examples/plugins/echo_plugin.sh for a minimal working example.
Safety Notes
Capabilities like git.push, filesystem.write_file, and android.gradle_build run real commands against your real filesystem and repos. Review a plan before running it against anything you care about.
orbvynx-security exists but is not yet a mandatory gate in front of every invocation — it's a building block, not a finished sandbox.
Plugins are regular subprocesses with your user's OS permissions. Only load plugins you trust.
No security audit has been done. Do not use this for production infrastructure yet.
Contributing
See CONTRIBUTING.md.
License
MIT — see LICENSE. Copyright (c) 2026 Waheed.
