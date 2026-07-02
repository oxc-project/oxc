# Coding agent guides for `crates/oxc_formatter_graphql`

## Overview

Prettier compatible GraphQL formatter (`oxfmt`'s Tier 1 backend), using the `oxc_formatter_core` APIs.

- Built on `oxc_formatter_core` for the language-agnostic IR + Printer + builders + macros
  - See `crates/oxc_formatter_core/AGENTS.md` for the IR/pipeline details
- Two entry points:
  - `format()`: standalone files (returns a printable `Formatted`)
  - `format_to_ir()`: embedded use via the dispatcher (e.g. graphql-in-js);
    allocates from the shared `EmbeddedContext` arena, emits no BOM / trailing newline,
    and leaves `propagate_expand()` to the parent document
- The canonical reference is Prettier's `src/language-graphql/printer-graphql.js`
  - port its layout decisions, do not invent new ones

### Forked parser

Prettier uses `graphql-js`, which is also the reference implementation of the GraphQL specification.
This means it is not locked to a specific GraphQL version and can parse a wide range of syntaxes, including draft syntax.

On the other hand, `apollo-parser`, the upstream we selected, strictly follows its versioning and currently only supports syntax up to Oct2021.
Therefore, we forked it as [`oxc-graphql-parser`](https://crates.io/crates/oxc-graphql-parser) and added some support ourselves, pinned via the workspace `Cargo.toml`.

The fork adds, behind opt-in parser flags, what Prettier(v3.8.4)'s graphql-js v16.x also accepts:

- executable descriptions (Sep2025 spec)
- and legacy fragment variables (`fragment F($x: Int) on T`)

For Prettier(v3.9)'s graphql-js v17.x syntax support, see the Roadmap below.

### Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- `oxc-graphql-parser` is error-tolerant (returns an AST even for invalid input), but any parse error bails out; never format a broken AST.
  - Several printer shortcuts (e.g. `close_delim_start`) are sound only under this guarantee
- The caller (oxfmt) decides what happens next
  - Diagnostics for standalone files, template-as-is for embedded

### Comments

`graphql-js` does not attach comments to the AST;
Prettier collects them from the token stream and attaches leading/trailing/dangling per node.

This crate instead collects comment spans into a positional cursor and flushes them at sequence items, closing delimiters, and document tail.

Node spans are significant-token spans (trivia is never included), so layout decisions use them directly.

All span bridging lives in `src/print/span.rs`:

- the usize→u32 conversion (`to_span`), the crate-local `Spanned` trait
  - (the `oxc_span::Span` facade over the parser's own `span` fields / `span()` accessors)
- and closing-delimiter derivation: `close_delim_start` for containers whose span ends one past their `}` / `]`
- `find_close_after` for paren lists that have no wrapper node

### Strings

Prettier prints `StringValue` from `graphql-js`'s _cooked_ value and re-encodes it.
`oxc-graphql-parser`'s `StringValue.value` is cooked but not to spec (no block-string dedent / blank-line trimming, no surrogate pairing), so `src/print/string.rs` cooks from `raw` itself:

- the GraphQL spec `BlockStringValue` algorithm (dedent + blank-line trimming)
- escape decoding for regular strings (incl. surrogate pairs)
- Prettier's re-encoding (`"`/`\` escaped, newline as `\n`, `"""` as `\"""`)

Blank-line runs inside block strings are part of the string VALUE;
the printer collapses consecutive line breaks, so they are emitted as raw `\n` text plus a `hard_line_break()` that only re-arms indentation (see `write_block_string_break`).

### Notable layout rules

- Blank-line preservation classifies the inter-token gap (`classify_gap`):
  a blank line is a whitespace-only line strictly inside the gap.
  Counting raw newlines would over-report when tokens (e.g. the `&` between two `implements` comments, or an insignificant comma) sit on their own line.
- A cooked `\r` escape in a string value is re-emitted as `\r`
  (Prettier emits a raw CR byte, which the core `text()` builder forbids; the string VALUE is identical).
- Known divergence: a trailing comment on the same line as a description
  (`"desc" # comment`) moves to the next flush point (Prettier keeps it inline).
  Pre-existing behavior of the positional comment cursor, affects type-system descriptions too;
  no conformance test covers this shape.

## Verification

```sh
cargo c -p oxc_formatter_graphql
```

Run `clippy` and resolve all warnings.

### Fixtures tests

Snapshot tests driven by fixture files under `tests/fixtures/graphql/`,
covering what the Prettier conformance suite does not:
`# oxfmt-ignore` suppression, blank-line runs inside block strings,
string escape re-encoding (incl. the `\r` divergence), empty `[]` / `{}` values,
the full set of type-system extensions, insignificant-comma trivia,
trailing comments at various positions,
width-overflowing `implements` lists (which never break),
and executable descriptions + legacy fragment variables
(comment / blank-line / width edges beyond the conformance fixtures).

`build.rs` auto-generates a test case from every `.graphql` file using the core
`test_support` harness. Unit tests in `tests/fixtures/mod.rs` cover parse-error
`Err` semantics and BOM preservation; `src/comments.rs` has `classify_gap` tests
(CR / CRLF endings, which `.gitattributes` keeps out of fixture files).

```sh
cargo test -p oxc_formatter_graphql
# Review / accept snapshots after intentional changes
cargo insta review -p oxc_formatter_graphql
```

Add a case by dropping a new `.graphql` file into the directory, the build script picks it up.

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes);
results live in `tasks/prettier_conformance/snapshots/prettier.graphql.snap.md`.
The `graphql` language is part of the shared conformance binary.

```sh
cargo run -p oxc_prettier_conformance
# Debug a specific test
cargo run -p oxc_prettier_conformance -- --filter graphql/<dir>/<file>
```

### Embedded conformance (`apps/oxfmt`)

The embedded-language features (gql-in-js) are validated end-to-end through the Oxfmt.

Requires a dev build first.

```sh
pnpm --dir apps/oxfmt build-dev
pnpm --dir apps/oxfmt conformance
```

### Manual checks

```sh
cargo run -p oxc_formatter_graphql --example graphql_formatter [filename]
# Compare with Prettier
npx prettier --parser=graphql [filename]
```

A good large real-world stress input is GitHub's public GraphQL schema (~72k lines).
It is too large and third-party to commit as a fixture,
bug-catching shapes are distilled into `tests/fixtures/graphql/implements-width.graphql`):

```sh
curl -sL https://docs.github.com/public/fpt/schema.docs.graphql -o /tmp/github-schema.graphql
diff <(npx prettier --parser=graphql /tmp/github-schema.graphql) \
  <(cargo run -q -p oxc_formatter_graphql --example graphql_formatter /tmp/github-schema.graphql)
```

## Roadmap (TODO: Follow graphql-js 17 / Prettier main)

The guiding axis is Prettier compatibility, not spec compliance:
match the syntax that the graphql-js version Prettier depends on can format
(Prettier stable = graphql-js 16, Prettier main = graphql-js 17).

Directive applications like `@oneOf` / `@defer` need no work. `oxc-graphql-parser` (apollo-parser 0.8.6 base) already parses them.

These are in Prettier's unreleased changelog (main has them, next stable will).

- [#18582](https://github.com/prettier/prettier/blob/main/changelog_unreleased/graphql/18582.md):
  allow `implements` lists to break. We currently implement never break
  (see `tests/fixtures/graphql/implements-width.graphql`),
  so this is a layout divergence that will become incompatible, not a new-syntax item, lands sooner.
- [#19171](https://github.com/prettier/prettier/blob/main/changelog_unreleased/graphql/19171.md):
  directives on directive definitions (`directive @a @b on QUERY`) + `extend directive`.
  graphql-js 17 graduated this to default (no option).
  Need to update our fork first.
- [#19297](https://github.com/prettier/prettier/blob/main/changelog_unreleased/graphql/19297.md):
  fragment arguments (`...F(size: $size)`).
  graphql-js 17 still gates it behind `experimentalFragmentArguments`;
  it replaces the v16 `allowLegacyFragmentVariables` (definition-side, parser-only), which v17 removed.
  Need to update our fork first.
