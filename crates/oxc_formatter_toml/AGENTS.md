# Coding agent guides for `crates/oxc_formatter_toml`

## Overview

TOML formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Parses with [`oxc-toml`](https://github.com/oxc-project/oxc-toml) (maintained in its own repository) and builds the IR from its CST
  - `oxc-toml` also ships a string formatter for its other users; this crate does not use it
  - A CST shape that needs a workaround here is fixed in `oxc-toml` instead
- Prettier has no TOML support (`prettier-plugin-toml` runs Taplo), so `tests/fixtures/` is the spec
- Entry points:
  - `format()`: standalone files
  - `format_to_ir()`: embedded use via the dispatcher (TOML front matter, Markdown code blocks)

### Layout

Based on Taplo's default style, aligned with the other languages where they agree:

- Every array is its own group, so only the array that does not fit expands (`width.toml`)
  - A trailing comment counts toward the width (`] # c` when it does not fit)
  - Blank lines between its values are kept only when it expands
  - A multi-line string expands it, as in JS and YAML (`strings.toml`)
  - An array with a comment anywhere inside keeps one value or comment per line
- An inline table is always printed on one line, even when written multi-line or too long, since TOML 1.0 has no multi-line inline tables
  - Except with a comment of its own, then one entry or comment per line (TOML 1.1), blank lines dropped
- Blank lines: up to 1 is kept, whitespace-only lines included (`blank-lines.toml`, `whitespace-only-lines.toml`)
  - None at the file start and end, between a table header and its first entry, and after `[` / before `]` of a multi-line array
- Comments keep their text, without trailing whitespace
  - A second comment on an array value's line moves to its own line (`arrays.toml`)
- `# prettier-ignore` / `# oxfmt-ignore` keeps the next root entry, table header, or array value as written (`ignore.toml`)
- `endOfLine: "cr"` prints LF, a lone CR is not a TOML line break

Not decided yet:

- A commented inline table printed one entry per line gets no trailing comma, unlike arrays (`inline-tables.toml`)
- `singleQuote` is ignored, strings and keys keep their quotes (`strings.toml`, `keys.toml`);
  a basic and a literal string are only interchangeable when no escape (or `\`) and no quote of the other kind is inside

Not aligned on purpose:

- `bracketSpacing` is ignored (`prettier-plugin-toml` does not pass it)

## Verification

```sh
cargo c -p oxc_formatter_toml
cargo test -p oxc_formatter_toml  # fixture snapshots, review with `cargo insta review`
```

Embedded behavior is covered by `apps/oxfmt`'s fixtures (`apps/oxfmt/conformance/fixtures/front-matter/`, `apps/oxfmt/conformance/fixtures/xxx-in-md/`).
