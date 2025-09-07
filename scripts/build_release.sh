#!/usr/bin/env bash
set -euo pipefail

# Build and package the Rust CLI binary for the host platform.
# Output: dist/codex-<target-triple>.tar.gz (+ .sha256)
#
# Usage:
#   scripts/build_release.sh [--target <triple>] [--features <features>] [--profile <profile>] \
#                            [--skip-tar]
#
# Notes:
# - Cross-compiling requires the appropriate Rust targets/toolchains installed.
# - By default, builds the host triple reported by `rustc -vV`.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
mkdir -p "${ROOT_DIR}/dist"

TARGET=""
FEATURES=""
PROFILE="release"
DO_TAR=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="$2"; shift 2 ;;
    --features)
      FEATURES="$2"; shift 2 ;;
    --profile)
      PROFILE="$2"; shift 2 ;;
    --skip-tar)
      DO_TAR=0; shift ;;
    *)
      echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "${TARGET}" ]]; then
  TARGET=$(rustc -vV | awk -F ': ' '/host:/ {print $2}')
fi

pushd "${ROOT_DIR}/codex-rs" >/dev/null

# Build the main CLI binary (bin name: codex, crate: codex-cli)
BUILD_ARGS=("--release")
if [[ "${PROFILE}" != "release" ]]; then
  BUILD_ARGS=("--profile" "${PROFILE}")
fi
if [[ -n "${TARGET}" ]]; then
  BUILD_ARGS+=("--target" "${TARGET}")
fi
BUILD_ARGS+=("-p" "codex-cli")
if [[ -n "${FEATURES}" ]]; then
  BUILD_ARGS+=("--features" "${FEATURES}")
fi

cargo build "${BUILD_ARGS[@]}"

# Locate built binary
BIN_DIR="target/${PROFILE}"
if [[ -n "${TARGET}" ]]; then
  BIN_DIR="target/${TARGET}/${PROFILE}"
fi

BIN_PATH="${BIN_DIR}/codex"
if [[ ! -f "${BIN_PATH}" ]]; then
  echo "Built binary not found at ${BIN_PATH}" >&2
  exit 1
fi

STAGE_DIR="${ROOT_DIR}/dist/_stage-${TARGET:-host}-${PROFILE}"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"

cp -f "${BIN_PATH}" "${STAGE_DIR}/"

# Package: codex-<target>.tar.gz
ASSET_TARGET="${TARGET}"
if [[ -z "${ASSET_TARGET}" ]]; then
  ASSET_TARGET=$(rustc -vV | awk -F ': ' '/host:/ {print $2}')
fi
ASSET_NAME="codex-${ASSET_TARGET}.tar.gz"

pushd "${STAGE_DIR}" >/dev/null
if [[ "${DO_TAR}" -eq 1 ]]; then
  tar -czf "${ROOT_DIR}/dist/${ASSET_NAME}" codex
  (cd "${ROOT_DIR}/dist" && sha256sum "${ASSET_NAME}" > "${ASSET_NAME}.sha256")
fi
popd >/dev/null

echo "Built artifact: ${ROOT_DIR}/dist/${ASSET_NAME}"
echo "SHA256: $(cut -d' ' -f1 "${ROOT_DIR}/dist/${ASSET_NAME}.sha256")"

popd >/dev/null
