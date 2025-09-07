#!/usr/bin/env bash
set -euo pipefail

# Install script to download the correct prebuilt binary from GitHub Releases
# and install it into ~/.local/bin (or a custom prefix).
#
# Usage examples:
#   REPO="<owner>/<repo>" bash -c "$(curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/main/scripts/install.sh)"
#   REPO="<owner>/<repo>" VERSION="v0.1.0" ./scripts/install.sh
#   REPO="<owner>/<repo>" PREFIX="/usr/local" ./scripts/install.sh
#
# Env vars:
#   REPO      - Required. GitHub repo in the form owner/name
#   VERSION   - Optional. Tag name (e.g., v0.1.0). If empty, uses "latest".
#   PREFIX    - Optional. Install prefix. Defaults to "$HOME/.local".
#   BINARY    - Optional. Defaults to "codex".

REPO="${REPO:-}"
if [[ -z "${REPO}" ]]; then
  echo "REPO env var is required (e.g., owner/name)" >&2
  exit 1
fi

VERSION="${VERSION:-}"
PREFIX="${PREFIX:-$HOME/.local}"
BINARY="${BINARY:-codex}"

mkdir -p "${PREFIX}/bin"

# Detect target triple
UNAME_S=$(uname -s)
UNAME_M=$(uname -m)

case "${UNAME_S}" in
  Linux)   OS=unknown-linux-gnu ;;
  Darwin)  OS=apple-darwin ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "Windows is not supported by this script yet." >&2
    exit 1 ;;
  *) echo "Unsupported OS: ${UNAME_S}" >&2; exit 1 ;;
esac

case "${UNAME_M}" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=aarch64 ;;
  *) echo "Unsupported arch: ${UNAME_M}" >&2; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"
ASSET="${BINARY}-${TARGET}.tar.gz"

if [[ -z "${VERSION}" ]]; then
  # Use latest release
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

TMPDIR=$(mktemp -d)
TARPATH="${TMPDIR}/${ASSET}"

echo "Downloading ${URL}..."
curl -fL "${URL}" -o "${TARPATH}"

echo "Extracting..."
tar -xzf "${TARPATH}" -C "${TMPDIR}"

if [[ ! -f "${TMPDIR}/${BINARY}" ]]; then
  echo "Extracted archive does not contain ${BINARY}" >&2
  exit 1
fi

install -m 0755 "${TMPDIR}/${BINARY}" "${PREFIX}/bin/${BINARY}"
echo "Installed ${BINARY} to ${PREFIX}/bin/${BINARY}"

if ! command -v "${BINARY}" >/dev/null 2>&1; then
  echo "Note: ${BINARY} is not on your PATH. Add ${PREFIX}/bin to your PATH." >&2
fi

