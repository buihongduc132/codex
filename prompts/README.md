Prompts: Codex config rollout (Ansible + Stow)

This folder manages two Codex setups:

- qox (prod): official `codex` binary with full permissions (no approvals)
- qoo (dev): local/forked `codex` binary with full permissions (no approvals)

It provides:

- GNU Stow packages to install wrappers (`qox`, `qoo`) into `~/bin` / `~/.local/bin`
- Ansible playbooks to deploy configs and wrappers to localhost, SSH hosts, or Docker targets
- Backup and sync helpers to archive current configs and fetch remote configs into this repo

Quick start

- Local install (wrappers + config symlinks via stow/script):
  - `./bin/install.sh`
  - Installs wrappers and centralizes dev config:
    - Creates/updates `prompts/config/qoo/`
    - Symlinks `~/.qoo` -> `prompts/config/qoo/`
    - Symlinks repo-local `.qoo` -> `prompts/config/qoo/`

- Local deploy (wrappers + configs via Ansible templates):
  - `cd ansible && ansible-playbook -i inventory.ini playbooks/local.yml`

- Remote deploy (SSH/Docker):
  - Edit `ansible/inventory.ini`
  - Set secrets/vars in `ansible/group_vars/all.yml`
  - `cd ansible && ansible-playbook -i inventory.ini playbooks/remote.yml`

- Backup configs from hosts into this repo:
  - `cd ansible && ansible-playbook -i inventory.ini playbooks/backup.yml`

Wrappers

- `qox`: finds `codex` on PATH and runs with `--ask-for-approval never --sandbox danger-full-access`.
- `qoo`: uses a separate `CODEX_HOME` and runs a dev binary with the same flags. It prefers `$CODEX_DEV_BIN` if set and executable, falling back to `~/Documents/Projects/bhd/qox/codex-rs/target/debug/codex`.

Config management

- Centralized dev config lives in `prompts/config/qoo/`. Both `~/.qoo` and repo-local `.qoo` point to it.
- Prod config can be tracked in `prompts/config/codex/` (optional); wrapper flags enforce full permissions regardless.
- Ansible can render configs from templates or sync from `prompts/config/*` to remotes.

Backups

- `ansible/playbooks/backup.yml` pulls entire `~/.codex/` and `~/.qoo/` directories and wrapper scripts into `backups/<host>/` (git-ignored).
