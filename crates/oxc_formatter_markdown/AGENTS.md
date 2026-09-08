# Coding agent guides for `crates/oxc_formatter_markdown`

Follow @../oxc_formatter_core/FORMATTER_POLICY.md , this file holds only the Markdown-specific rules.
Known divergences live in DIVERGENCES.md.

## Overview

Prettier compatible Markdown formatter on `oxc_formatter_core`.
Entry points and their contracts are documented in `src/format.rs`.

Parses with [`oxc-markdown-parser`](https://github.com/oxc-project/oxc-markdown-parser).
Parse behavior targets micromark (what Prettier parses with); its AGENTS.md / DIVERGENCES.md are the reference.
Style facts (markers, fence kind, setext, break kind, list padding, ...) are AST fields and `ParserReturn::blanks` lists the blank lines,
so the printer never re-derives layout from the source text.
Formatter policy (sentence splitting, CJK, aligned lists, escaping) stays here.

### Escaping

Text is printed as written. Escapes are added in three places only:
`*` / `_` runs inside emphasis, a lone `===` / `---` word starting a line, and link destinations / titles / labels.
Plain text outside emphasis is never escaped and entities are never decoded;
that is what keeps `snake_case`, `:emoji:`, `{#id}` attributes and `<Comp @click="a * b">` intact for every dialect.

### Envelope

Source is normalized to `\n` before parsing (verbatim slices go straight into the IR), the configured line ending is re-emitted at print time,
a leading BOM is preserved.

Front matter (`---` / `+++`) is not handled (`oxc_formatter_core::spec::parse_front_matter` + `envelope::write_front_matter`, as CSS does).

## Dialects

Markdown grammars are open-ended (VitePress, Docusaurus, Pandoc, kramdown, Obsidian, ...), and the parser learns none of them:
a construct exists only when micromark / a Prettier extension parses it or GitHub renders it.
The formatter keeps the rest intact without grammar: no escaping of plain text (see "Escaping" above) and line shapes.

Line shapes (`inline::line_shape`): a paragraph line starting with `:::`, `<<<`, a component tag (`<Badge`, `<my-element`),
or `[!` as the first line of a blockquote keeps both its line boundaries and is never re-wrapped, under every `proseWrap`.
The list is fixed and shape-based, not per dialect; `preserve` (the default) keeps every line anyway,
so only `always` / `never` are affected. Add a shape only with a fixture in `tests/fixtures/markdown/prose-wrap/line-shapes.md`.

When a dialect construct breaks: never add its grammar to the parser; add a shape (with the fixture), or record it below as a limit.

Known limits (not handled, by decision):

- kramdown IAL lines (`{: .class}` gets a blank line before it)
- MDX ESM in `.md` (Docusaurus; `import` lines re-wrap)
- 4-space nested lists re-indented to 2 (`tabWidth: 4` keeps them)
- Pandoc simple / grid tables, Python-Markdown admonitions without a blank line before them

## Verification

```sh
cargo run -p oxc_formatter_markdown --example markdown_formatter [filename]
DUMP_IR=1 cargo run -p oxc_formatter_markdown --example markdown_formatter [filename]

# The oracle. Without `--no-config --no-editorconfig` the repo's `.editorconfig` changes the output
node apps/oxfmt/node_modules/prettier/bin/prettier.cjs --parser markdown --no-config --no-editorconfig [filename]
# Prettier's doc, when the output alone does not explain a layout
node -e 'const p=require("./apps/oxfmt/node_modules/prettier");p.__debug.printToDoc(require("fs").readFileSync(process.argv[1],"utf8"),{parser:"markdown"}).then(d=>p.__debug.formatDoc(d)).then(console.log)' [filename]
```

### Prettier conformance

At the current version (v3.9.6), these divergences have been confirmed and are intentional (see DIVERGENCES.md):

Conformance failures classified as divergences: `thematicBreak/simple.md` (`#leading-thematic-break`), `link/encodedLink.md` (`#url-escaping`), `gfm-test-suite/snippet: example-491.md` (`#single-tilde-strikethrough`), `liquid/*.md` (`#liquid-flow-tags`), `blockquote/notext-end.md` (`#line-shapes`).

### Fixture fingerprint

`tests/fixtures/fingerprint.rs` serializes the AST's meaning (structure, decoded text, destinations, labels) and ignores what formatting may change (spans, markers, fence style, text splitting, whitespace runs, tightness).
The harness asserts it is identical for input and output: every fixture is a `parse(format(x)) ≅ parse(x)` check, which idempotency alone cannot give (a corrupted output is often a fixpoint).
Extend the ignore set only with a reason written next to it.
