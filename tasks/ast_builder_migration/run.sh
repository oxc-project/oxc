#!/usr/bin/env bash
#
# Apply the `AstBuilder` migration + shortening rules, repeatedly, until no further changes remain.
#
# `ast-grep` applies a SINGLE pass per invocation: it rewrites every current match but does not
# re-scan its own output. Our rules cascade - one rule's output is another rule's input (e.g.
# box-collapse produces the `T::boxed(..)` that variant-wrap then consumes) - and nested calls only
# collapse one level per pass. So reaching a fixed point means running until a pass makes no changes.
#
# Usage:
#   tasks/ast_builder_migration/run.sh <path>...                 # paths to rewrite (in place)
#   tasks/ast_builder_migration/run.sh crates apps napi --globs '!**/generated/**'
#
# All arguments are forwarded verbatim to `ast-grep scan`, so any of its flags (e.g. `--globs` to
# exclude generated files) can be appended after the paths.

set -euo pipefail

dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rules="$dir/generated/rules.yml"

if [ "$#" -eq 0 ]; then
  echo "usage: $(basename "$0") <path>... [ast-grep flags]" >&2
  echo "example: $(basename "$0") crates apps napi --globs '!**/generated/**'" >&2
  exit 64
fi

max_passes=25
pass=0
while :; do
  pass=$((pass + 1))
  # `ast-grep` reports "Applied N changes" when it rewrites anything, and prints nothing when a pass
  # is a no-op. `|| true` stops `set -e` from aborting on a non-zero exit (e.g. a parse warning).
  output="$(ast-grep scan --rule "$rules" --update-all "$@" 2>&1 || true)"

  if ! printf '%s\n' "$output" | grep -q 'Applied [1-9]'; then
    echo "Fixed point reached after $((pass - 1)) pass(es) that made changes."
    # Surface anything `ast-grep` printed on the final pass (e.g. errors), if any.
    [ -n "$output" ] && printf '%s\n' "$output"
    break
  fi

  echo "Pass $pass: $(printf '%s\n' "$output" | grep -oE 'Applied [0-9]+ changes' | head -1)"

  if [ "$pass" -ge "$max_passes" ]; then
    echo "Stopped after $max_passes passes - the rules may be cycling." >&2
    exit 1
  fi
done
