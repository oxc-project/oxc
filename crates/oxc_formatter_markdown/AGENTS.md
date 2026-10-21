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

Container columns (list item, footnote, indented code) are `space_align`, never `align`:
the next parse strips exactly those columns and a tab spans up to 4,
while an `align` becomes a tab under `useTabs` once an `indent` or a blockquote prefix follows it.
`tests/fixtures/markdown/use-tabs/` pins it.

### Escaping

Text is printed as written. Escapes are added in three places only:

- `*` / `_` runs inside emphasis
- a lone `===` / `---` word starting a line
- link destinations / titles / labels

Emphasis marker normalization is verified before it is committed:
`print/pairing.rs` runs the parser's delimiter resolver on the printed text and falls back to the markers as written when the nodes would differ.
Plain text outside emphasis is never escaped and entities are never decoded;
that is what keeps `snake_case`, `:emoji:`, `{#id}` attributes and `<Comp @click="a * b">` intact for every dialect.

### Envelope

Source is normalized to `\n` before parsing (verbatim slices go straight into the IR), the configured line ending is re-emitted at print time,
a leading BOM is preserved.

Front matter (`---` / `+++`, `oxc_formatter_core::spec::parse_front_matter`) is blanked before parsing and printed by `envelope::write_front_matter` as CSS does:
its yaml formats through the session's dispatcher when there is one, anything else stays verbatim; a blank line separates it from the body.

A fenced code block with a language dispatches its content as a `VirtualDocument` (`print/code.rs`), its name as written (decoded), the dispatcher resolving it:
the child IR is wrapped in `mark_as_root` with every newline in its texts made a literal line, so continuation lines keep the fence's column,
and the fence outnumbers the backtick runs of the printed content (printed once to count them when a text holds a backtick).
No dispatcher, an unknown language or a failed parse keeps the block verbatim, as does a meta with line ranges (`apps/oxfmt/DIVERGENCES.md#line-ranged-code-block`).
Embedding is verified end to end by oxfmt's conformance (`markdown` / `md-in-js` categories), not here.

## Dialects

Markdown grammars are open-ended (VitePress, Docusaurus, Pandoc, kramdown, Obsidian, ...), and the parser learns none of them:
a construct exists only when micromark / a Prettier extension parses it or GitHub renders it.
The formatter keeps the rest intact without grammar: no escaping of plain text (see "Escaping" above) and line shapes.

Line shapes (`line_shape::line_shape`): a paragraph line starting with `:::`, `<<<`, a component tag (`<Badge`, `<my-element`),
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

Every failing file is accounted for by a DIVERGENCES.md entry's `Conformance:` line; there are no unclassified failures.

Pin fixtures are named after their entry's slug (a comment would be an HTML block and change the document).

### Fixture fingerprint

`tests/fixtures/fingerprint.rs` serializes the AST's meaning (structure, decoded text, destinations, labels) and ignores what formatting may change
(spans, markers, fence style, text splitting, whitespace runs, tightness, trailing whitespace of verbatim lines, info string whitespace).
The harness asserts it is identical for input and output: every fixture is a `parse(format(x)) ≅ parse(x)` check, which idempotency alone cannot give (a corrupted output is often a fixpoint).
Extend the ignore set only with a reason written next to it.

### Fuzzed invariants

`tests/invariants.rs` formats a deterministic token-soup corpus (the parser repo's differential generator) under every `proseWrap` and checks the fingerprint and idempotency.
The default corpus must fail exactly on `KNOWN_FAILURES` (the documents and their classes are listed there);
remove an entry when its class is fixed, any other failure is a regression.

Other seeds pass (3 / 5 / 7 / 8 / 13 / 21 / 42 / 99) except one known class, seed 11 #5:
a liquid tag continued on a lazy line (`2. {% x` + `y %}`) is inline in a paragraph,
re-indented into the item it becomes a liquid flow block (micromark reads the two the same way).
`MD_FUZZ_SEED` / `MD_FUZZ_COUNT` run other corpora, `MD_FUZZ_FILE` prints one document's fingerprints.
