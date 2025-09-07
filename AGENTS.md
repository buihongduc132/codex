# Rust/codex-rs

In the codex-rs folder where the rust code lives:

- Crate names are prefixed with `codex-`. For example, the `core` folder's crate is named `codex-core`
- When using format! and you can inline variables into {}, always do that.
- Never add or modify any code related to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` or `CODEX_SANDBOX_ENV_VAR`.
  - You operate in a sandbox where `CODEX_SANDBOX_NETWORK_DISABLED=1` will be set whenever you use the `shell` tool. Any existing code that uses `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` was authored with this fact in mind. It is often used to early exit out of tests that the author knew you would not be able to run given your sandbox limitations.
  - Similarly, when you spawn a process using Seatbelt (`/usr/bin/sandbox-exec`), `CODEX_SANDBOX=seatbelt` will be set on the child process. Integration tests that want to run Seatbelt themselves cannot be run under Seatbelt, so checks for `CODEX_SANDBOX=seatbelt` are also often used to early exit out of tests, as appropriate.

Before finalizing a change to `codex-rs`, run `just fmt` (in `codex-rs` directory) to format the code and `just fix -p <project>` (in `codex-rs` directory) to fix any linter issues in the code. Prefer scoping with `-p` to avoid slow workspace‑wide Clippy builds; only run `just fix` without `-p` if you changed shared crates. Additionally, run the tests:
1. Run the test for the specific project that was changed. For example, if changes were made in `codex-rs/tui`, run `cargo test -p codex-tui`.
2. Once those pass, if any changes were made in common, core, or protocol, run the complete test suite with `cargo test --all-features`.
When running interactively, ask the user before running these commands to finalize.

## TUI style conventions

See `codex-rs/tui/styles.md`.

## TUI code conventions

- Use concise styling helpers from ratatui’s Stylize trait.
  - Basic spans: use "text".into()
  - Styled spans: use "text".red(), "text".green(), "text".magenta(), "text".dim(), etc.
  - Prefer these over constructing styles with `Span::styled` and `Style` directly.
  - Example: patch summary file lines
    - Desired: vec!["  └ ".into(), "M".red(), " ".dim(), "tui/src/app.rs".dim()]

## Snapshot tests

This repo uses snapshot tests (via `insta`), especially in `codex-rs/tui`, to validate rendered output. When UI or text output changes intentionally, update the snapshots as follows:

- Run tests to generate any updated snapshots:
  - `cargo test -p codex-tui`
- Check what’s pending:
  - `cargo insta pending-snapshots -p codex-tui`
- Review changes by reading the generated `*.snap.new` files directly in the repo, or preview a specific file:
  - `cargo insta show -p codex-tui path/to/file.snap.new`
- Only if you intend to accept all new snapshots in this crate, run:
  - `cargo insta accept -p codex-tui`

If you don’t have the tool:
- `cargo install cargo-insta`

## End‑to‑end verification (agent policy)

When a change affects runtime behavior (CLI flags, output, prompts, UI text, etc.), the agent must:
- Build the modified crate(s) successfully (`cargo build -p <crate>`).
- Perform a real, non‑interactive run to verify behavior when feasible (e.g., a dedicated status or dry‑run mode) and capture sample output in the handoff.
- Avoid offloading basic verification to the user. Only ask for manual steps when external systems (e.g., network accounts, API keys, OS‑level wrappers) are required and cannot be exercised in the sandbox.

## Root‑Cause Fixes Over Workarounds

- Always fix problems at the fundamental level. Do not ship monkey patches, ad‑hoc workarounds, or band‑aids that mask the real issue.
- Prefer small, targeted changes that address the root cause in core code paths (protocol, path resolution, config) over app‑level hacks.
- When the issue stems from ambiguous context (e.g., missing cwd), make that context explicit and thread it through the system.

## Persistent CWD and Workspace Discipline

To avoid confusion and inconsistent behavior between tools, the agent MUST operate with a persistent, explicit working directory:

- Carry `cwd` explicitly in session/config and honor it everywhere (exec, apply_patch, file reads/writes).
- Resolve relative paths for apply_patch and shell operations against the correct workdir, not the binary’s directory.
- When the model emits commands like `cd foo && …`, treat `foo` as the intended workdir for that operation (parse and apply consistently).
- Validate the workdir before spawn. If it doesn’t exist, fail clearly or fall back in a predictable way (and surface the chosen cwd in logs/events).

Upstream reference PRs in base that implement these behaviors (use them as patterns when changing code here):
- feat: make cwd a required field of Config (#800)
- fix: when a shell tool call invokes apply_patch, resolve relative paths against workdir (#556)
- fix: ensure apply_patch resolves relative paths against workdir or project cwd (#810)
- parse `cd foo && …` for exec and apply_patch (#3083)
- fix: check workdir before spawn (#221)

## Initial Environment Context (Always Provide)

At session start and before performing file or shell actions, the agent must ensure environment context is captured and visible to both the user and the model:

- cwd: absolute path of the current working directory.
- git repo: repo root path if inside a Git repository; otherwise “none”.
- branch: current branch name (or detached HEAD SHA) if in a Git repo.

Expose this context via:
- Status lines or the first system/assistant message in a session.
- Exec summaries for failures (include cwd and tails where applicable).
- When possible, pass `-C/--cd <cwd>` at launch so the runtime and UI agree on cwd.

## Machine State (_STATE.md)

- This repo includes a root‑level `_STATE.md` that records the current machine’s setup for Codex wrappers, config symlinks, Ansible rollout settings, and versions.
- Agents and contributors should skim `_STATE.md` first to understand the local environment before making changes.
- `_STATE.md` is intentionally git‑ignored and may differ per machine; treat it as a local source of truth for operational context.
