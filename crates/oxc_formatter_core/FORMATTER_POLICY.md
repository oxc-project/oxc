# Formatter policy

Shared policy for every formatter crate in the oxc ecosystem:

- `oxc_formatter` (JS/TS)
- `oxc_formatter_json`
- `oxc_formatter_css`
- `oxc_formatter_graphql`
- `oxc_formatter_yaml`

using `oxc_formatter_core`, integrated by `apps/oxfmt`.

Each crate's `AGENTS.md` holds only language-specific rules and the crate-local translation of these policies; when they conflict, the crate file wins for that crate (and the conflict should be fixed).

`apps/oxfmt` is bound by this policy as the integration side of its contracts (error handling, embedded conformance); everything about tier dispatch, configuration, and delegation to Prettier stays in its own `AGENTS.md`.

## Prettier compatibility

- Matching Prettier's output is the GOAL, not an absolute rule
  - The higher the conformance coverage the better, but 100% parity is not what we promise
  - What we provide above all is CONSISTENT, PREDICTABLE, EXPLAINABLE behavior
  - When parity conflicts with that, "Known divergences" below arbitrates
- The canonical reference is Prettier's OUTPUT
  - Its source code is an analysis aid, not a porting target
  - Match its layout decisions, do not invent new ones
  - Never mirror its internal logic 1:1; pin behavior with fixtures instead
  - Comments explain BEHAVIOR and mechanism, never Prettier's code structure
    - Naming an upstream function as a search key is fine (greppable in any Prettier version) — most valuable when pinning an artifact explanation or a compat-table entry
    - File names / line numbers are not (not reproducible without a commit hash)
- Implementation strategies legitimately differ (e.g. Prettier pre-classifies comments per context; we decide on the spot with positional cursors)
  - Compatibility is judged on bytes out, not code shape
- Before matching a mismatch, always consider whether it is a Prettier bug or artifact (see "Known divergences")
- The oracle version is the `prettier` pinned in `apps/oxfmt/package.json`: the bundle, the conformance suite (via `oxc_formatter_tests`), and fixture verification all derive from that one version
  - The LATEST Prettier is still worth consulting as a forward-looking aid: whether a bug we diverged on has been fixed upstream, or a behavior is about to change
  - When the pin catches up to an upstream fix, converge and drop the divergence entry

### Known divergences

Deliberate divergences from Prettier's output. Admission reasons:

- (1) Prettier's output would change program semantics: Formatting must NEVER do that (this reason is mandatory to act on, the others are judgment calls)
  - Idempotency breakage is a symptom of this class: output that fails to re-parse, or re-parses to a different value/structure
  - Verify semantics claims with the reference compiler/parser (tsc, dart-sass, lessc, ...), not intuition
- (2) Prettier's behavior is a bug: acknowledged as an open issue, OR explainable as an artifact (of comment attachment, parser structure, token lexing, ...) rather than an intended layout rule
  - Typical case: comment relocations that are attachment artifacts of Prettier's parser
  - "Looks wrong" without an artifact explanation is NOT enough; follow Prettier until the mechanism is understood
- (3) Prettier's behavior conflicts with a uniform rule across our formatter crates
  - The uniform rule wins, even where Prettier's behavior is normal and internally consistent
    - e.g. re-quoting SCSS `@warn "x"` per `singleQuote` where Prettier keeps the raw string verbatim
  - This also covers Prettier's internal inconsistencies (same construct, different output depending on node kind or context): one principle beats emulating the inconsistency
- (4) The impact does not justify the matching cost (layout-only, rare trigger)

Rules:

- Style debates (`status:needs discussion` issues) are still followed; do not "improve" on taste
  - Applying a rule already established across our crates (reason 3) is not taste; inventing a new style neither Prettier nor our crates have is
- Every divergence is documented in the owning crate's AGENTS.md "Known divergences" section, with the reason
- Every divergence is pinned by a fixture whose comments say which lines deviate from Prettier and why
- Affected conformance fixtures stay counted as failures; a new conformance failure is acceptable only under this policy and must be documented
- The rule cuts both ways: never "fix" a conformance failure by following Prettier into a documented divergence
  - Check the crate's divergence list and open oxc issues for intent before treating a diff as a plain bug

## Comment placement invariants

Two layers of rules; know which one you are editing:

- Invariants hold uniformly: violating one is a bug even where Prettier disagrees
- Compat tables record measured Prettier behavior that is not derivable from principle: extend them by measuring Prettier, never by analogy, and pin every entry in a fixture

The invariants:

- A comment never crosses user content (code, other comments, other tokens): it stays on its source side of every token
  - When Prettier relocates a comment across tokens, that is an attachment artifact to diverge from (see "Known divergences"), not a rule to emulate
- A comment never crosses a line boundary: line-based directives (`eslint-disable-line`, ...) must keep their meaning
  - Line comments print via `line_suffix`; own-line comments stay own-line
- A suppression comment (`prettier-ignore` / `oxfmt-ignore`) never loses its target, and its original text is preserved
- Repositioning is allowed only relative to formatter-OWNED punctuation: the formatter owns terminators (e.g. a statement's `;`) and the trivia up to them; the user owns content
  - Terminator vs separator: a terminator cannot be replaced by another token (`;` after a JS statement); a separator can (`,`/`;` between interface members)
    - Comments may move behind a terminator (per-language compat tables decide when); they always stay before a separator

Per-language translations (which tokens are terminators, the compat tables, cursor bounds disciplines) live in each crate's AGENTS.md.

## Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- Any parse error bails out, even from an error-tolerant parser: never format a broken AST
- Print-stage internal errors are also `Err`
- The caller (oxfmt) decides what happens next: diagnostics for standalone files, template-as-is for embedded

A parse error is a SAFE failure (the input is left as-is); silent token corruption is the UNSAFE one.
When triaging a report: don't corrupt → then accept → then pretty-print.

## Verification

Every crate:

```sh
cargo c -p <crate>        # plus any crate-specific feature configs listed in its AGENTS.md
```

Run `clippy` for the same configurations and resolve all warnings.

Whenever comparing output against Prettier (fixture verification, manual checks), use the oxfmt-bundled one:

```sh
# A bare `npx prettier` is NOT installed at the repo root; the bundled one is the same version as the oracle
node apps/oxfmt/node_modules/prettier/bin/prettier.cjs --parser <parser> --print-width=100 <file>
```

NOTE: Prettier's default `printWidth` is `80`, but Oxfmt is `100`.

### Fixture tests

Snapshot tests driven by fixture files under `tests/fixtures/`; they cover what the Prettier conformance suite does not (suppression, divergence pins, embedded shapes, ...).
`build.rs` auto-discovers every fixture file and generates a test per file — add a case by dropping a file in, no registration needed. Options are resolved from the nearest `options.json` up the directory tree.

```sh
cargo test -p <crate>
# Review / accept snapshots after intentional changes
cargo insta test --accept -p <crate>
```

Every expected output must be verified against Prettier, except fixtures pinning a "Known divergence".

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes).

Every language crate owns its conformance as a `tests/conformance.rs` target (via `oxc_formatter_tests::conformance`, report pinned with `insta`), part of the crate's regular `cargo test`:

```sh
cargo test -p <crate> --test conformance
# Debug a specific test
PRETTIER_FILTER=<path> cargo test -p <crate> --test conformance -- --nocapture
```

JSDoc formatting is covered by plain fixture-pair tests in `oxc_formatter` (`--test jsdoc`, committed input/expected pairs — a mismatch is a failing test, not a tracked report entry).

Failures must be either fixed or classified under "Known divergences".

### Embedded conformance (`apps/oxfmt`)

The embedded-language features (e.g. xxx-in-js / js-in-xxx) are validated end-to-end through Oxfmt. Requires a dev build first.
