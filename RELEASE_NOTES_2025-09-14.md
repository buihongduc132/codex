# Release 2025-09-14: Hooks + Named Summaries

Highlights:

- Hooks (issue #2)
  - Runs `session_start`/`session_end` and `pre_command`/`post_command` hooks with routing to UI/LLM and fail-closed handling for pre_command.
  - Adds TOML parsing: `[hooks]` with `command`, `route`, `timeout_ms`, `on_error`.
  - PR: https://github.com/buihongduc132/codex/pull/11

- Named summaries + Compact rationale (issue #9)
  - Exec: `--summarize-name` and `--from-summarize`; auto-save to `~/.codex/summaries/<name>.json` with metadata; seed from saved summary.
  - TUI: `--from-summarize` to preload initial prompt from a named summary.
  - Compact rationale: explained and implemented as a normal chat turn (not a system instruction) for ChatGPT/Pro compatibility.
  - PR: https://github.com/buihongduc132/codex/pull/12

Notes:
- Core tests updated to reflect Compact-as-chat behavior.
- TUI unit tests have pre-existing compile-time expectations; the TUI crate compiles and the flags are present in `tui --help`.

