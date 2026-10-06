# Coding agent guides for `crates/oxc_formatter_toml`

## Overview

TOML formatter (`oxfmt`'s Tier 2 backend), a thin wrapper around [`oxc-toml`](https://github.com/oxc-project/oxc-toml).

- `oxc-toml` is a formatter-only fork of Taplo, maintained in its own repository
  - Formatting logic and its tests live there, not here
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
  - Pinned in `apps/oxfmt/test/api/markdown.test.ts`
- No BOM split, which the other crates do

Known limits of the string embedding (text with literal lines, the same as Prettier's `replaceEndOfLine()` on the plugin's string):

- Blank lines inside an indented root (list item, blockquote) keep the root's prefix as trailing whitespace, as Prettier does
- Every line resumes at the parent's root, so lines would not follow an enclosing `indent()`
  - Once a host does that (e.g. an indented Vue custom block), structural lines must follow the indent but multi-line string contents must not
  - A string cannot tell them apart (the same reason js-in-vue moved off string embedding)

## Verification

```sh
cargo c -p oxc_formatter_toml
```

Embedded behavior is covered by `apps/oxfmt` tests (`test/api/front_matter.test.ts`, `test/api/markdown.test.ts`).
