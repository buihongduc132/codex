# Tasks (handoff)

## Track 1 — Tests and Stabilization
- [ ] Fix `codex-tui` compilation errors (API and tests):
  - [ ] Duplicate imports in `tui/src/history_cell.rs` (use `codex_core::auth` only) — initial fix applied.
  - [ ] Add deps `dirs`, `uuid`; import `format_si_suffix` — initial fix applied.
  - [ ] Update `/status` auth helpers to `codex_core::auth::{get_auth_file, try_read_auth_json}` — applied.
  - [ ] Update `ChatWidget` call sites to new `set_token_usage(Option<TokenUsageInfo>)` signature.
  - [ ] Update tests to wrap `Uuid` in `ConversationId` where required.
  - [ ] Provide `timeout_ms` in `ExecCommandBeginEvent` initializers.
  - [ ] Update `default_user_shell(session_id, codex_home)` call sites in tests.
  - [ ] Resolve missing helper(s) (e.g., `exec_command_lines`) or adjust tests accordingly.
  - [ ] Remove/rename duplicate test fns (e.g., `slash_popup_model_first_for_mo_ui`).
- [ ] Build: `cargo test -p codex-tui --no-run`; then targeted tests for core/cli/exec.
- [ ] Full: `cargo test --all-features`.
- [ ] Capture results to `artifacts/summary_track1_<ts>.md` and update `flow/progress.md`.

## Track 2 — Merge base/main
- [ ] Fetch and merge/rebase `base/main` into a temp branch.
- [ ] Conflict policy: keep our features; keep upstream improvements otherwise.
- [ ] Verify compatibility changes (examples from base):
  - [ ] `SessionConfiguredEvent` gaining `rollout_path`.
  - [ ] ArchiveConversation and InitialHistory re‑exports.
  - [ ] Responses originator env override rename.
- [ ] Build affected crates; run targeted tests; then full workspace tests.
- [ ] Summarize merge decisions; update `flow/progress.md` and `flow/notes.md`.

## Policy & Constraints
- No upstream writes; origin only for any GH interactions.
- No custom compact prompt; compact restored to server‑fixed version.
- Keep `/init` per‑profile override support; do not change system prompt.

---

## Commands reference (run from repo root)
- just fmt
- cargo test -p codex-core --no-run
- cargo test -p codex-tui --no-run
- cargo test -p codex-cli --no-run
- cargo test -p codex-exec --no-run
- cargo test --all-features

