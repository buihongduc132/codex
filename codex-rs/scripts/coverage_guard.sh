#!/usr/bin/env bash
# Coverage guard for Codex Rust workspace
# Fails if coverage regresses by more than thresholds compared to a stored baseline.
#
# Thresholds (drops beyond these are blocked):
#   - Per-file:    >3.0% drop
#   - Overall:     >1.0% drop
#
# Baseline file: codex-rs/.coverage-baseline.tsv
#   Format: one entry per line: "<PATH>\t<COVERAGE_PERCENT>"
#   Special total line uses path __TOTAL__
#
# Usage:
#   scripts/coverage_guard.sh check         # Check against baseline (create if missing)
#   scripts/coverage_guard.sh write-baseline  # Force regenerate baseline from current coverage
set -euo pipefail

REPO_ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
BASELINE_FILE="$REPO_ROOT_DIR/.coverage-baseline.tsv"
THRESHOLD_PER_FILE=3.0
THRESHOLD_OVERALL=1.0

have_cmd() { command -v "$1" >/dev/null 2>&1; }

die() { echo "[coverage-guard] ERROR: $*" >&2; exit 1; }
info() { echo "[coverage-guard] $*"; }

# Ensure tooling exists
ensure_tooling() {
  if ! have_cmd cargo; then die "cargo not found in PATH"; fi
  if ! have_cmd rustup; then die "rustup not found in PATH"; fi
  if ! have_cmd cargo-llvm-cov; then
    die "cargo-llvm-cov not found. Install with: rustup component add llvm-tools-preview && cargo install cargo-llvm-cov --locked"
  fi
}

# Run cargo-llvm-cov summary-only and emit TSV lines: "<PATH>\t<COVERAGE_PERCENT>"
# Includes a TOTAL line with key __TOTAL__ for overall coverage (Lines%).
collect_coverage_tsv() {
  # We avoid clean to speed up pre-commit; rely on cargo-llvm-cov cache
  local out
  out=$(cargo llvm-cov --all-features --workspace --summary-only 2>/dev/null || true)
  if [ -z "$out" ]; then
    die "cargo llvm-cov produced no output"
  fi
  # Parse table lines. We compute Line coverage as (1 - MissedLines/Lines)*100 to avoid column index ambiguity.
  # Table tail columns are: Lines, Missed Lines, Cover, Branches, Missed Branches, Cover
  # We'll grab path as $1 and then from the right take Lines and MissedLines.
  echo "$out" | awk '
    BEGIN { OFS="\t"; }
    # Data lines start with a non-space (filename or TOTAL) and have many numeric fields.
    /^[^[:space:]][^|]*[[:space:]][0-9]/ {
      n = NF;
      # Guard: we expect at least 7 fields (path + 6 tail stats)
      if (n < 7) next;
      path=$1;
      lines=$(n-5); missed=$(n-4);
      # ensure numeric
      if (lines+0==0) next;
      cov=(1.0 - (missed+0.0)/(lines+0.0))*100.0;
      # Normalize TOTAL row key
      if (path=="TOTAL") path="__TOTAL__";
      printf "%s\t%.2f\n", path, cov;
    }
  '
}

write_baseline() {
  info "Generating coverage baseline at $BASELINE_FILE"
  mkdir -p "$(dirname "$BASELINE_FILE")"
  collect_coverage_tsv | sort > "$BASELINE_FILE"
  info "Baseline written. Entries: $(wc -l < "$BASELINE_FILE" | tr -d ' ')"
}

check_against_baseline() {
  if [ ! -f "$BASELINE_FILE" ]; then
    info "No baseline found. Creating one from current coverage..."
    write_baseline
    info "Baseline created; allowing commit."
    exit 0
  fi

  local tmp_cur
  tmp_cur=$(mktemp)
  collect_coverage_tsv | sort > "$tmp_cur"

  # Load overall baseline and current
  local base_total cur_total
  base_total=$(awk -F"\t" '$1=="__TOTAL__"{print $2}' "$BASELINE_FILE")
  cur_total=$(awk -F"\t" '$1=="__TOTAL__"{print $2}' "$tmp_cur")
  if [ -z "$base_total" ] || [ -z "$cur_total" ]; then
    rm -f "$tmp_cur"
    die "Missing TOTAL rows in coverage output or baseline"
  fi

  # Compare overall
  # If current < baseline - THRESHOLD_OVERALL -> fail
  awk -v base="$base_total" -v cur="$cur_total" -v th="$THRESHOLD_OVERALL" 'BEGIN{
    drop = base - cur;
    if (drop > th) {
      printf("[coverage-guard] Overall coverage drop %.2f%% exceeds threshold %.2f%% (base=%.2f%%, current=%.2f%%)\n", drop, th, base, cur) > "/dev/stderr";
      exit 1;
    } else {
      exit 0;
    }
  }'
  local overall_ok=$?

  # Compare per-file for files that exist in baseline and in current
  # Output violations as TSV to a temp file
  local violations
  violations=$(mktemp)
  join -t $'\t' -j 1 <(grep -v '^__TOTAL__' "$BASELINE_FILE" | sort) <(grep -v '^__TOTAL__' "$tmp_cur" | sort) \
    | awk -F"\t" -v th="$THRESHOLD_PER_FILE" '{
        base=$2+0.0; cur=$3+0.0; drop=base-cur;
        if (drop > th) { printf("%s\t%.2f\t%.2f\t%.2f\n", $1, base, cur, drop); }
      }' > "$violations"

  local status=0
  if [ "$overall_ok" -ne 0 ]; then status=1; fi
  if [ -s "$violations" ]; then
    echo "[coverage-guard] Per-file coverage drops exceeding ${THRESHOLD_PER_FILE}%:" >&2
    printf "  %-60s  base%%   curr%%   drop%%\n" "FILE" >&2
    awk -F"\t" '{printf "  %-60s  %6.2f  %6.2f  %6.2f\n", $1, $2, $3, $4}' "$violations" >&2
    status=1
  fi

  rm -f "$tmp_cur" "$violations"

  if [ $status -ne 0 ]; then
    echo "[coverage-guard] To update the baseline after an intentional improvement across the codebase (e.g. after merging into main), run:" >&2
    echo "  $0 write-baseline" >&2
    exit 1
  fi

  info "Coverage check passed (overall and per-file within thresholds)."
}

cd "$REPO_ROOT_DIR"
ensure_tooling

cmd=${1:-check}
case "$cmd" in
  check)
    check_against_baseline
    ;;
  write-baseline)
    write_baseline
    ;;
  *)
    echo "Usage: $0 [check|write-baseline]" >&2
    exit 2
    ;;
 esac

