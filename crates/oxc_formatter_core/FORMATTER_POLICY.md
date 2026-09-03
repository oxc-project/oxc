# Formatter policy

Shared policy for every formatter crate in the oxc ecosystem:

- `oxc_formatter` (JS/TS)
- `oxc_formatter_json`
- `oxc_formatter_css`
- `oxc_formatter_graphql`
- `oxc_formatter_yaml`

using `oxc_formatter_core`, integrated by `apps/oxfmt`.

Each crate's `AGENTS.md` holds only language-specific rules and the crate-local translation of these policies.
A conflict between the two is a bug in one of them: until it is fixed, the crate file governs that crate, and the crate file records the conflict as a known violation.

Vocabulary used below:

- "taste": a layout no admission reason grounds (see "Known divergences"). Never admitted
- "the pin": the Prettier version pinned in `apps/oxfmt/package.json`, the single oracle for every crate

`apps/oxfmt` is bound by this policy as the integration side of its contracts (error handling, embedded conformance); everything about tier dispatch, configuration, and delegation to Prettier stays in its own `AGENTS.md`.

## Prettier compatibility

- What we provide above all is CONSISTENT, PREDICTABLE, EXPLAINABLE behavior; the higher the conformance coverage the better, but 100% parity is not what we promise
- The canonical reference is Prettier's OUTPUT
  - Its source code is an analysis aid, not a porting target
  - Match its layout decisions; the only way to differ is an admitted divergence (see "Known divergences"), never taste
  - Never mirror its internal logic 1:1; pin behavior with fixtures instead
  - Before matching a mismatch, always check whether an admission reason already decides it (see "Known divergences")
- The pin is the oracle: the bundle, the conformance suite (via `oxc_formatter_tests`), and fixture verification all derive from that one version
  - Prettier `main` is consulted for every mismatch, as evidence, not as an oracle: it shows whether the pin's output is Prettier's current intent or something upstream already considers a bug, but `main` is unreleased and may still change
  - An upstream fix changes nothing in admission: our output follows from the reasons, whether or not `main` agrees
    - A mismatch some reason grounds is entered like any other, citing the upstream PR as a reference; our shape may differ from `main`'s
    - A mismatch no reason grounds is a STYLE change on `main`: never adopted ahead of the pin, we print the pin's layout until the pin moves

### Known divergences

Admission reasons (1)-(4) are checked in order: the first that applies is the `Why`, the other traits go in the prose.
(5) is outside that order: a temporary hold on a pin change that no reason grounds, pending an adoption decision.
Every reason is OUR decision factor; Prettier's mechanism (comment attachment, `lineSuffix`, token lexing, ...) is explanation for the prose, never a reason.

- (1) `semantics`: Prettier's output would change program semantics (fails to re-parse, or re-parses to a different value/structure)
  - Mandatory: Formatting must NEVER do that
  - Verify semantics claims with the reference compiler/parser (tsc, dart-sass, lessc, ...), not intuition
- (2) `invariant`: Prettier's output breaks one of our formatter contract invariants; that set is closed, extending it is a policy change
  - The comment placement invariants (see "Comment placement invariants" below): the output moves content the user owns
  - Idempotency: a second pass must reproduce the first, layout included; a formatter with no fixpoint has no defined output
  - Mandatory: the output is inadmissible whatever its layout merits
  - Only the comment itself moving is (2); when the comment stays and the tokens around it change layout, that is (3) `uniform-rule (comment presence never changes layout)`
  - No reason parenthetical (an upstream issue reference is still fine): the prose names which invariant in one sentence (crosses content, changes line, loses a suppression target, is not a fixpoint)
- (3) `uniform-rule`: our output follows a rule we apply uniformly; Prettier's output is admissible, we chose otherwise
  - The only judgment call among (1)-(3); the `Why` names the rule
  - The usual trigger is Prettier's internal inconsistency, one construct printed differently by context:
    - `uniform-rule (same construct, same output: <sibling>)`, we print the sibling's shape everywhere
    - The sibling is a construct Prettier ITSELF prints that way (measured), never a shape of our own; an entry that cannot name it is not admitted under this rule
  - The rarer trigger is Prettier being consistent but sidestepping one of our rules (an option, the `line_suffix` width rule, ...):
    - `uniform-rule (<rule>)`, and the entry body cites where the rule already applies (a section of this document, another crate, another construct)
    - A rule with no other application is not uniform; it is taste until a second application exists
  - When both a named rule and a sibling apply, the `Why` names the rule: it holds regardless of what Prettier prints for the sibling
- (4) `cost`: The impact does not justify the matching cost
  - Admissible only when the difference is layout-only and rarely triggered; the entry states what matching would require
  - The usual cost is structural (our IR, printer or AST differs from Prettier's): the entry names the difference, which is its natural `Drop when`
- (5) `style-hold`: Prettier changed an intended style and we deliberately stay on the PREVIOUS Prettier version's output, pending an adoption decision
  - The `Why` cites the tracking issue that records the decision: `style-hold (oxc#NNNN)`; never a novel style of our own
  - Resolve by following the new style or re-classifying, then drop or rewrite the entry
  - An open upstream style debate (`status:needs discussion`) is not a hold: the pin's output is still matched

Rules:

- Every divergence is documented in the owning layer's `DIVERGENCES.md` and pinned by a fixture whose comments say which lines deviate from Prettier and why
  - Entry format: an H2 slug (the stable anchor external pointers use), required `Why:` (reason keyword; its parenthetical holds what the reason defines, then any upstream issue as a bare reference: `uniform-rule (same construct, same output: X; prettier/prettier#NNNN)`) and `Pin:` lines; an input/ours/prettier example is the body, written with OUR behavior as the spec
  - At a pin bump, re-run conformance: an entry no longer observed is deleted, the rest are re-checked against their reason; `Drop when:` is written only for a condition more specific than that default
  - No status, dates, or authors: listed = accepted, condition met = delete (git holds history)
  - The owning layer is where the behavior is decided: a language crate for single-language behavior, `apps/oxfmt` for embedding E2E behavior
  - A divergence discovered through a real-world conformance case (oxfmt externals, Prettier suite) is pinned by DISTILLING a minimal fixture into the owning layer's `tests/fixtures/`; the big source file stays in conformance as a regression net, never as the pin
  - `DIVERGENCES.md` holds entries only, opening with one back-reference line to this section; `AGENTS.md` keeps policy translations and mechanism prose;
    - oxfmt conformance notes never explain, one identifying line + `See <path>/DIVERGENCES.md#<slug>`
- Affected conformance fixtures stay counted as failures; a new conformance failure is acceptable only under this policy, and the crate's `AGENTS.md` conformance section lists the file with the entry slug it belongs to
- The rule cuts both ways: never "fix" a conformance failure by following Prettier into a documented divergence
  - Check the crate's divergence list and open oxc issues for intent before treating a diff as a plain bug

## Comment placement invariants

Two layers of rules; know which one you are editing:

- Invariants hold uniformly: violating one is a bug even where Prettier disagrees
- Compat tables record measured Prettier behavior that is not derivable from principle: extend them by measuring Prettier, never by analogy, and pin every entry in a fixture

The invariants:

- A comment never crosses user content (code, other comments, other tokens): it stays on its source side of every token
  - When Prettier relocates a comment across tokens, that is a divergence under reason (2) `invariant` (see "Known divergences"), not a rule to emulate
- A comment never crosses a line boundary: line-based directives (`eslint-disable-line`, ...) must keep their meaning
  - Line comments print via `line_suffix`; own-line comments stay own-line
- A suppression comment (`prettier-ignore` / `oxfmt-ignore`) never loses its target, and its original text is preserved
- Repositioning is allowed only relative to formatter-OWNED punctuation: the formatter owns terminators (e.g. a statement's `;`) and the trivia up to them; the user owns content
  - Terminator vs separator: a terminator cannot be replaced by another token (`;` after a JS statement); a separator can (`,` / `;` between TS interface members)
    - The replaceability test only separates these two
    - Comments may move behind a terminator (per-language compat tables decide when); they always stay before a separator
  - Grammar-fixed DELIMITER (braces, a head's parens) is neither: it bounds a region and stays user content, never crossed
  - (JS/TS) Redundant expression parentheses are NOT delimiters: the formatter drops them and re-derives parens by its own rules, so any paren in the output is formatter-owned
    - Trailing comment inside the dropped pair moves behind the terminator, even across a re-printed pair
    - The source pair stays user content only where a sub-printer claims it and prints the comment inside (per-language keeps tables)
    - The line-boundary invariant still gates the move: never across a re-printed paren that ends up on its own line

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

Every expected output must be verified against the pin; in a fixture pinning a "Known divergence", every line except the ones its comments mark as deviating.

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes).

Every language crate owns its conformance as a `tests/conformance.rs` target (via `oxc_formatter_tests::conformance`, report pinned with `insta`), part of the crate's regular `cargo test`:

```sh
cargo test -p <crate> --test conformance
# Debug a specific test
PRETTIER_FILTER=<path> cargo test -p <crate> --test conformance -- --nocapture
```

The Prettier suite lives under `crates/oxc_formatter_tests/prettier/` and is self-provisioned on the first conformance run. It is gitignored, so use `rg --no-ignore` (or `-u`) when searching it.

Failures must be either fixed or classified under "Known divergences".

### E2E conformance (`apps/oxfmt`)

The embedded-language features (e.g. xxx-in-js / js-in-xxx) are validated end-to-end through Oxfmt. Requires a dev build first.

There are also conformance tests for each language that use real-world repositories.
