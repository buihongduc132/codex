#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v stow >/dev/null 2>&1; then
  echo "Error: GNU stow not found. Install it (apt install stow / brew install stow)." >&2
  exit 1
fi

cd "$ROOT_DIR/stow"

backup_dir="$HOME/.stow-backups/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir"

# Pre-flight: back up existing non-symlink files that would conflict
maybe_backup() {
  local target="$1"
  if [[ -e "$target" && ! -L "$target" ]]; then
    echo "Backing up existing $target -> $backup_dir" >&2
    mkdir -p "$(dirname "$backup_dir/$target")"
    mv "$target" "$backup_dir/$target"
  fi
}

maybe_backup "$HOME/bin/qox"
maybe_backup "$HOME/.local/bin/qoo"

echo "Linking 'codex' (qox) and 'codex-dev' (qoo) into home..."
stow -v -R -t "$HOME" codex codex-dev

echo "Done. Wrappers installed: $HOME/bin/qox and $HOME/.local/bin/qoo"

# Centralize qoo config in repo and create symlinks
CONF_ROOT_REPO="$ROOT_DIR/config/qoo"
mkdir -p "$CONF_ROOT_REPO"

# If a non-symlink ~/.qoo exists and repo config is empty, import it once
if [[ -d "$HOME/.qoo" && ! -L "$HOME/.qoo" ]]; then
  if [[ ! -e "$CONF_ROOT_REPO/config.toml" && -e "$HOME/.qoo/config.toml" ]]; then
    echo "Importing existing ~/.qoo/config.toml into prompts/config/qoo" >&2
    cp -a "$HOME/.qoo/config.toml" "$CONF_ROOT_REPO/config.toml"
  fi
fi

# Backup then replace ~/.qoo with symlink to prompts/config/qoo
maybe_backup "$HOME/.qoo"
ln -snf "$CONF_ROOT_REPO" "$HOME/.qoo"
echo "Linked ~/.qoo -> $CONF_ROOT_REPO"

# Create project-local ./.qoo symlink for dev workflows
ln -snf "$CONF_ROOT_REPO" "$ROOT_DIR/../.qoo"
echo "Linked repo ./.qoo -> $CONF_ROOT_REPO"
