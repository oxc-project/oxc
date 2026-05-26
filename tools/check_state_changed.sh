#!/usr/bin/env bash
# Fails if any `state.changed =` write exists in crates/oxc_minifier/
# outside the four sanctioned helpers in ecma_context.rs.
#
# Scope is restricted to Rust source (`-t rust`) — markdown examples and
# other non-source files can't write to the field at runtime.
#
# See: docs/superpowers/specs/2026-05-25-minifier-eliminate-changed-flag-design.md
set -euo pipefail

violations=$(rg -n 'state\.changed\s*=' -t rust crates/oxc_minifier/ \
    --glob '!crates/oxc_minifier/src/traverse_context/ecma_context.rs' \
    || true)

if [ -n "$violations" ]; then
    echo "ERROR: Unauthorized state.changed writes detected." >&2
    echo "       Use ctx.replace_expression / ctx.replace_statement /" >&2
    echo "       ctx.notice_change / ctx.reset_changed instead." >&2
    echo "" >&2
    echo "$violations" >&2
    exit 1
fi
