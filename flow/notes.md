# Notes (handoff context)

Decisions
- Do not customize compact prompt. We reverted local compact to match `base/main` and removed per‑profile compact overrides. `/compact` always uses the built‑in prompt.
- Keep per‑profile init prompt override (`init_prompt_file`) and leave system prompt unchanged.
- Added internal exec auto‑save post‑run (no model tool). Writes `~/.codex/saves.json` with path + metadata; config gated.
- CLI gained `export`, `import`, `load --list` for rollouts/saves listing.
- Added per‑profile hook types (internal slash commands or external argv). Execution wiring remains to be finalized.

Open items
- `codex-tui` compilation: multiple API/test updates pending (see flow/tasks.md track 1 checklist).
- Merge from `base/main`: complete reconciliation and rebuild/tests.

Updates (2025-09-12)
- Aligned prompt.md to base/main to resolve Responses API 400 on ChatGPT auth.
- Restored MOODED banner with build metadata.
- Added TUI branch footer on its own line: " <branch> • <dir> •".
- Built and shipped both binaries into dist/: codex and codex-tui.

Paths & references
- Logs: see artifacts/ (recent: `qoop_run_*`, `qoop_track1_*`, `qoop_track2_*`, `qoop_followup_*`).
- Progress: flow/progress.md, flow/tasks.md.

Environment
- Current mode: danger-full-access, approval: never, non‑interactive. Commands should use timeouts.

Suggested next steps
- Finish `codex-tui` compile fixes per checklist, then rerun tests (targeted → full).
- Complete base/main merge reconciliation and revalidate.
