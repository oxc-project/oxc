# Coding agent guides for `crates/oxc_formatter_toml`

## Overview

TOML formatter (`oxfmt`'s Tier 2 backend), a thin wrapper around [`oxc-toml`](https://github.com/oxc-project/oxc-toml).

- `oxc-toml` is a formatter-only fork of Taplo, maintained in its own repository
  - Formatting logic lives there; `tests/fixtures/` here are golden snapshots of its output, the reference for the IR rewrite (see "Known bugs")
  - It prints a string, it does NOT build the `oxc_formatter_core` IR
- Prettier has no TOML support; `prettier-plugin-toml` runs Taplo, so `oxc-toml`'s output is the reference as-is
- Entry points:
  - `format()`: standalone files
  - `format_to_ir()`: embedded use via the dispatcher (TOML front matter, Markdown code blocks)

### Divergences and the IR goal

All divergences below stem from `oxc-toml` printing a string instead of building the IR.
The goal is to move the formatting logic into this crate and build the `oxc_formatter_core` IR like the other `oxc_formatter_*` crates;
`oxc-toml` would then shrink to a parser.

Not done yet, since no case needs it today:

- The output already IS the reference (`prettier-plugin-toml` runs Taplo), so a rewrite risks diverging from it
- Every embedding host places the child at a root (front matter at column 0, Markdown code blocks under `mark_as_root`),
  where the string with literal lines prints the same as an IR would

Strong motivations, since these break rules every other language follows:

- Width: the child is formatted at the full `printWidth` wherever it sits
  - Other languages' IR is printed by the parent's printer, so their width counts from the embedding column (e.g. inside a list item)
  - Prettier with the plugin behaves the same as now, but consistency across our languages comes first
  - Pinned in `apps/oxfmt/conformance/fixtures/xxx-in-md/print-width-20/toml-code-block.md`

Known limits of the string embedding (text with literal lines, the same as Prettier's `replaceEndOfLine()` on the plugin's string):

- Blank lines inside an indented root (list item, blockquote) keep the root's prefix as trailing whitespace, as Prettier does
- Every line resumes at the parent's root, so lines would not follow an enclosing `indent()`
  - Once a host does that (e.g. an indented Vue custom block), structural lines must follow the indent but multi-line string contents must not
  - A string cannot tell them apart (the same reason js-in-vue moved off string embedding)

### Known bugs

The IR rewrite must keep every fixture snapshot byte-identical, except these `oxc-toml` bugs it may fix
(marked `# Known bug` in the fixture inputs):

- Width is counted in Unicode scalars, not display width (CJK, emoji, combining marks, ZWJ sequences): `width.toml`
- A tab counts as 1 column, in an indent (`use-tabs/width.toml`) or a string (`width.toml`)
- With `endOfLine: crlf`, the `\r` counts toward the width: `crlf/width.toml`
- Line breaks inside multi-line strings are not normalized: the `Input variants differ: CRLF` lines (`strings.toml`, `real-world/`)

Kept as-is by the IR rewrite, to fix afterwards as a breaking `fix(formatter_toml)!`:

- Blank lines: up to 2 are kept (Taplo's `allowed_blank_lines`), other languages keep 1;
  they are also kept at the file start and end, after a table header, and after `[` / before `]` of a commented array
  (`blank-lines.toml`, `arrays.toml`, `crlf/eol.toml`)
- Ignore comments (`# prettier-ignore`, `# oxfmt-ignore`) are not supported, every other language supports them
- Once an entry expands, every nested array expands too, even one that fits (`width.toml`);
  other languages expand only the group that does not fit
- A whitespace-only line is not counted as a blank line and is dropped (`whitespace-only-lines.toml`)
- Trailing whitespace after a comment is kept (`whitespace-only-lines.toml`)
- An empty array inside a forced expansion prints as `[` + newline + `]` (`width.toml`)

Not decided yet:

- A commented inline table printed one entry per line gets no trailing comma, unlike arrays (`inline-tables.toml`)
- `singleQuote` is ignored, strings and keys keep their quotes (`strings.toml`, `keys.toml`);
  a basic and a literal string are only interchangeable when no escape (or `\`) and no quote of the other kind is inside

Not aligned on purpose:

- `bracketSpacing` is ignored (`prettier-plugin-toml` does not pass it)
- An inline table without comments is always printed on one line, even when written multi-line or too long,
  since TOML 1.0 has no multi-line inline tables

## Verification

```sh
cargo c -p oxc_formatter_toml
cargo test -p oxc_formatter_toml  # fixture snapshots, review with `cargo insta review`
```

Embedded behavior is covered by `apps/oxfmt`'s fixtures (`apps/oxfmt/conformance/fixtures/front-matter/`, `apps/oxfmt/conformance/fixtures/xxx-in-md/`).
