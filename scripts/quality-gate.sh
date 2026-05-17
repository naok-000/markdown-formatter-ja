#!/usr/bin/env bash
set -euo pipefail

max_lines_for() {
  case "$1" in
    tests/commonmark_escaping.rs)
      echo 400
      ;;
    *)
      echo 300
      ;;
  esac
}

check_line_counts() {
  local failed=0

  while IFS= read -r file; do
    local lines
    local max_lines

    lines="$(wc -l < "$file" | tr -d ' ')"
    max_lines="$(max_lines_for "$file")"

    if [ "$lines" -gt "$max_lines" ]; then
      printf 'line-count: %s has %s lines, max is %s\n' "$file" "$lines" "$max_lines" >&2
      failed=1
    fi
  done < <(find src tests -name '*.rs' -type f | sort)

  return "$failed"
}

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
check_line_counts
