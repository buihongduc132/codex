# Progress (summary)

- Compact prompt: Restored to match `base/main`; removed per‑profile compact overrides; `/compact` always uses built‑in prompt.
- Exec auto‑save: Internal function added; writes `~/.codex/saves.json` with path + metadata (branch, datetime UTC, cwd, repo, commit, worktree?, blank summary). Config: `exec.auto_save_metadata` (default true), `exec.save_name_pattern`.
- CLI UX: Added `export`, `import`, and `load --list` to manage rollouts and listing saved sessions.
- Hooks schema: Added per‑profile hooks types (internal “/cmd” or external argv). Execution wiring is pending finalization.
- Init prompt: Per‑profile `init_prompt_file` supported; system + compact unchanged.

Testing status (latest)
- `codex-core`: builds for tests (--no-run)
- `codex-tui`: fails to build; initial fixes applied (deps/imports), many API/test updates still required.
- Workspace: full tests not run due to tui build failures.

Artifacts
- Initial qoop run (spec + workspace scan):
  - artifacts/qoop_run_20250910_033949.log
- Feature implementation run (auto‑save, CLI UX, hooks schema, compact reset):
  - artifacts/qoop_run_20250910_200246.log
- Track 1 tests (fmt + targeted + workspace):
  - artifacts/qoop_track1_20250911_015022.log
  - artifacts/summary_track1_20250910T185204Z.md
- Track 2 initial merge adjustments (rollout_path adaptation seen in tail):
  - artifacts/qoop_track2_20250911_015540.log
- Follow‑ups (spawned, may still be running or may have timed out):
  - artifacts/qoop_followup_track1_fix_tui_*.log
  - artifacts/qoop_followup_track2_merge_*.log

Next checkpoints
- Track 1: Finish tui compile/test fixes, re‑run targeted tests, then full suite.
- Track 2: Complete merge from `base/main`, reconcile conflicts, build/tests, summarize decisions.

---
Merge status (2025-09-11)
- Branch merged: `tmp/track2-merge-20250910T213818Z` → `main`
- Merge commit on `main`: merge: integrate track1+track2 work, handoff files, and base/main reconciliation
- Formatting: ran `just fmt` in `codex-rs`.
- Lint: ran `just fix -p codex-core` and `just fix -p codex-tui`.

Build/Test summary post-merge
- codex-core: unit tests PASS (187/187) after targeted fixes.
  - Changes:
    - Added `profiles.gpt5` to the unit test fixture.
    - ZDR test now sets `disable_response_storage` via overrides (profile field intentionally ignored by loader).
    - Fixed precedence bug to prefer `ConfigOverrides.disable_response_storage` over `config.toml` for consistency with docs.
    - Updated `gpt‑5` context window expectation to 272k (upstream metadata).
- codex-tui: builds; tests PASS (309 passed).
- Workspace full suite: not executed yet.

---
Latest updates (2025-09-12)
- Fixed OpenAI Responses 400 ("Instructions are not valid") by aligning core/prompt.md to base/main and avoiding extra embedded tool instructions when desired.
- Restored MOODED build banner and added build metadata (sha, time, branch) to TUI and exec paths.
- Added branch/footer line in TUI composer: a second line now shows " <branch> • <dir> •".
- Delivered binaries:
  - TUI (prompt-sync + branch footer): dist/codex-tui-259636b0-mooded-branch2
  - CLI (with save/load): dist/codex-259636b0-mooded
- Built TUI variants during iteration: dist/codex-tui-259636b0-mooded-promptsync (prompt sync only).

Suggested follow-ups
- Investigate the two failing `codex-core` config precedence tests (likely fixture expectations updated after upstream config changes: `disable_response_storage` default and provider headers ordering).
- After addressing, run `cargo test --all-features` from `codex-rs`.
