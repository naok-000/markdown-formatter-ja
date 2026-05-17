#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/codex-review.sh MODE [VALUE]

Modes:
  --uncommitted       Review staged, unstaged, and untracked changes.
  --base BRANCH      Review changes against BRANCH.
  --commit SHA       Review changes introduced by SHA.
EOF
}

if [ "$#" -lt 1 ]; then
  usage
  exit 2
fi

mode="$1"
shift

case "$mode" in
  --uncommitted)
    codex_args=(review --uncommitted)
    label="uncommitted"
    ;;
  --base)
    if [ "$#" -ne 1 ]; then
      usage
      exit 2
    fi
    codex_args=(review --base "$1")
    label="base-$1"
    ;;
  --commit)
    if [ "$#" -ne 1 ]; then
      usage
      exit 2
    fi
    codex_args=(review --commit "$1")
    label="commit-$1"
    ;;
  *)
    usage
    exit 2
    ;;
esac

label="${label//\//-}"

review_dir=".codex/reviews"
mkdir -p "$review_dir"

timestamp="$(date -u '+%Y%m%dT%H%M%SZ')"
latest="$review_dir/latest.md"
output="$review_dir/$timestamp-$label.md"

codex "${codex_args[@]}" | tee "$output"
cp "$output" "$latest"
