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
- Before matching a mismatch, always consider whether it is a Prettier bug or artifact (see "Known divergences")
- The oracle version is the `prettier` pinned in `apps/oxfmt/package.json`: the bundle, the conformance suite (via `oxc_formatter_tests`), and fixture verification all derive from that one version
  - The LATEST Prettier is still worth consulting as a forward-looking aid: whether a bug we diverged on has been fixed upstream, or a behavior is about to change
  - When the pin catches up to an upstream fix, converge and drop the divergence entry

### Known divergences

Admission reasons:

- (1) `semantics`: Prettier's output would change program semantics
  - Formatting must NEVER do that (this reason is mandatory to act on, the others are judgment calls)
  - Idempotency breakage is a symptom of this class: output that fails to re-parse, or re-parses to a different value/structure
  - Verify semantics claims with the reference compiler/parser (tsc, dart-sass, lessc, ...), not intuition
- (2) `prettier-bug`: Prettier's behavior is a bug
  - Acknowledged as an open issue, OR explainable as an artifact (of comment attachment, parser structure, token lexing, ...) rather than an intended layout rule
  - e.g. comment relocations that are attachment artifacts of Prettier's parser
  - When "looks wrong", find and understand an artifact explanation
- (3) `uniform-rule`: Prettier's behavior conflicts with a uniform rule across our formatter crates
  - The uniform rule wins, even where Prettier's behavior is normal and internally consistent
  - e.g. re-quoting SCSS `@warn "x"` per `singleQuote` where Prettier keeps the raw string verbatim
  - This also covers Prettier's internal inconsistencies (same construct, different output depending on node kind or context): one principle beats emulating the inconsistency
- (4) `cost`: The impact does not justify the matching cost
  - This is layout-only, rare trigger
- (5) `style-hold`: Prettier changed an intended style and we deliberately stay on the PREVIOUS Prettier version's output, pending an adoption decision
  - Requires a tracking issue recording the decision; never a novel style of our own (that would be taste)
  - Resolve by following the new style or re-classifying, then drop or rewrite the entry

Choosing the reason:

- Each reason judges a different thing: (1) the OUTPUT's effect, (2) the CAUSE on Prettier's side, (3) OUR principle, (4) the impact-vs-cost trade, (5) a tracked hold on an upstream style change
- Semantics takes precedence: output that changes meaning is (1) even when the mechanism is also an artifact; a bug label must not hide the one mandatory class
- (2) vs (3) litmus: would the divergence disappear once Prettier fixes itself and the pin catches up?
  - Yes → (2), and give the entry a `Drop when`
  - No → (3), the principle outlives any upstream fix
- When an entry has traits of both, the `Why` is the ground that DECIDED admission; the other trait goes in the prose
- Style debates (`status:needs discussion` issues) are still followed; do not "improve" on taste
  - Applying a rule already established across our crates (reason 3) is not taste; inventing a new style neither Prettier nor our crates have is

Rules:

- Every divergence is documented in the owning layer's `DIVERGENCES.md` and pinned by a fixture whose comments say which lines deviate from Prettier and why
  - Entry format: an H2 slug (the stable anchor external pointers use), required `Why:` (admission reason keyword + upstream issue if any) and `Pin:` lines, `Drop when:` only when a convergence condition exists; an input/ours/prettier example is the body, written with OUR behavior as the spec
  - `prettier-bug` entries drop by default when the upstream fix reaches the pin (that is what made them (2)); write `Drop when:` only when the condition is more specific than that default
  - No status, dates, or authors: listed = accepted, condition met = delete (git holds history)
  - The owning layer is where the behavior is decided: a language crate for single-language behavior, `apps/oxfmt` for embedding E2E behavior
  - A divergence discovered through a real-world conformance case (oxfmt externals, Prettier suite) is pinned by DISTILLING a minimal fixture into the owning layer's `tests/fixtures/`; the big source file stays in conformance as a regression net, never as the pin
  - `DIVERGENCES.md` holds entries only, opening with one back-reference line to this section; `AGENTS.md` keeps policy translations and mechanism prose;
    - oxfmt conformance notes never explain, one identifying line + `See <path>/DIVERGENCES.md#<slug>`
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
  - Terminator vs separator: a terminator cannot be replaced by another token (`;` after a JS statement); a separator can (`,` / `;` between TS interface members)
    - The replaceability test only separates these two
    - Comments may move behind a terminator (per-language compat tables decide when); they always stay before a separator
  - Grammar-fixed DELIMITER (braces, a head's parens) is neither: it bounds a region and stays user content, never crossed

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

Fixture tests and Prettier conformance re-format every output and record idempotency violations in their snapshots/reports.

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

The Prettier suite lives under `crates/oxc_formatter_tests/prettier/` and is self-provisioned on the first conformance run. It is gitignored, so use `rg --no-ignore` (or `-u`) when searching it.

JSDoc formatting is covered by plain fixture-pair tests in `oxc_formatter` (`--test jsdoc`, committed input/expected pairs — a mismatch is a failing test, not a tracked report entry).

Failures must be either fixed or classified under "Known divergences".

### E2E conformance (`apps/oxfmt`)

The embedded-language features (e.g. xxx-in-js / js-in-xxx) are validated end-to-end through Oxfmt. Requires a dev build first.

There are also conformance tests for each language that use real-world repositories.
