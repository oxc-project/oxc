# Formatter policy

Shared policy for every formatter crate whose `AGENTS.md` links here, built on `oxc_formatter_core` and integrated by `apps/oxfmt`.

Each crate's `AGENTS.md` holds only language-specific rules and the crate-local translation of these policies.
A conflict between the two is a bug in one of them: until it is fixed, the crate file governs that crate, and the crate file records the conflict as a known violation.

## Fixing

- Fix the cause, never stack patches: a special case added only to make one failing input pass (a context check, a flag, a magic number) is a patch
  - A root fix removes the patches it makes redundant
- Look for the bug's relatives (sites with the same check, assumption, or root cause) and fix them in the same change, or report the ones out of scope
- Fix in the layer that owns the cause (parser, core, another crate), never compensate downstream
  - The parsers are ours to keep the formatters small and fast: a parser bug is fixed in the parser, even when the fix waits for a release

## Prettier compatibility

- The goal is CONSISTENT, PREDICTABLE, EXPLAINABLE behavior, not 100% parity with Prettier
  - Consistency and a sound rationale win over parity: the `DIVERGENCES.md` files record where they already did
  - Conformance is a measure, not a target: a deliberate divergence lowers it and is still correct
- The single oracle is Prettier's OUTPUT at the pin, the version in `apps/oxfmt/package.json`: the bundle, the conformance suite (`oxc_formatter_tests`) and fixture verification all use it
  - Its source code is an analysis aid, never a porting target: pin behavior with fixtures, do not mirror its logic
  - A mismatch is fixed to match the pin, unless an admission reason decides otherwise (see "Known divergences")
- Prettier `main` and its issues are evidence, not an oracle: they show whether the pin's output is Prettier's intent, a known bug, or a bug already fixed there
  - Before fixing or admitting a mismatch, check them along with the `DIVERGENCES.md` entries and open oxc issues
  - Following a `main` fix is a judgment call: follow clear bug fixes, stay on the pin for unstable changes or ones the community pushes back on
  - A followed fix is a divergence from the pin, entered under (1)-(3) citing the PR

### Known divergences

Admission reasons (1)-(4) are checked in order: the first that applies is the `Why`, the other applicable reasons go in the prose.
(5) is outside that order: a temporary hold, only when none of (1)-(4) applies.
Every reason is OUR decision factor; Prettier's mechanism (comment attachment, `lineSuffix`, token lexing, ...) is explanation for the prose, never a reason.

- (1) `semantics`: Prettier's output would change program semantics (fails to re-parse, or re-parses to a different value/structure)
  - Verify semantics claims with the reference compiler/parser (tsc, dart-sass, lessc, ...), not intuition
- (2) `invariant`: Prettier's output breaks one of our formatter contract invariants; that set is closed, extending it is a policy change
  - Lossless: every comment is printed, exactly once; comments are the user's, and losing one outranks moving one
    (whatever the compiler makes of it: a comment may carry tool meaning, `/*! */`, `-disable`, etc)
  - Comment placement: the output moves a comment in a way "Comment placement invariants" below forbids
  - Idempotency: a second pass must reproduce the first, layout included; a formatter with no fixpoint has no defined output
  - Only the comment itself moving is (2); when the comment stays and the tokens around it change layout, that is (3) `uniform-rule (comment presence never changes layout)`
  - The `Why` takes no parenthetical except upstream issues; the prose names the broken invariant in one sentence (crosses content, changes line, loses a suppression target, is not a fixpoint)
- (3) `uniform-rule`: our output follows a rule we apply uniformly; Prettier's output is admissible, we chose otherwise
  - The only judgment call among (1)-(3); the `Why` names the rule
  - The usual trigger is Prettier's internal inconsistency, one construct printed differently by context:
    - `uniform-rule (same construct, same output: <sibling>)`, we print the sibling's shape everywhere
    - The sibling is a construct Prettier ITSELF prints that way (measured), never a shape of our own; an entry that cannot name it is not admitted under this rule
  - The rarer trigger is Prettier being consistent but sidestepping one of our rules (a user option governs everywhere, the `line_suffix` width rule, ...):
    - `uniform-rule (<rule>)`, and the entry body cites where the rule already applies (a section of this document, another crate, another construct)
    - A rule with no other application is not uniform; it needs a second application first
  - When both a named rule and a sibling apply, the `Why` names the rule: it holds regardless of what Prettier prints for the sibling
- (4) `cost`: The impact does not justify the matching cost
  - Admissible only when the difference is layout-only and rarely triggered; the entry states what matching would require
  - The usual cost is structural (our IR, printer or AST differs from Prettier's): the entry names the difference, which is its natural `Drop when`
- (5) `style-hold`: Prettier changed an intended style and we deliberately stay on the PREVIOUS Prettier version's output, pending an adoption decision
  - The `Why` cites the tracking issue that records the decision: `style-hold (oxc#NNNN)`; never a novel style of our own
  - Resolve by following the new style or re-classifying
  - An open upstream style debate (`status:needs discussion`) is not a hold: the pin's output is still matched

Rules:

- Every divergence has an entry in the owning layer's `DIVERGENCES.md` and a pin fixture
  - The owning layer decides the behavior: a language crate for single-language behavior, `apps/oxfmt` for embedding
    - A behavior core decides for every language (see "Text width") is a rule in this document;
      each language crate where it shows has its own entry and pin fixture, citing that section
  - The pin fixture is minimal and lives in the owning layer, distilled from whatever exposed the divergence; a conformance file is never the pin fixture, it stays a regression net
  - The entry's `Pin:` names the fixture, which needs no link back; the why and the deviating lines live in the entry
- `DIVERGENCES.md#<slug>` is the one form for referring to an entry (docs, source, fixtures); never a quoted or bare slug, never an upstream issue alone
- Entry format, with OUR behavior as the spec
  - H2 slug: the stable anchor
  - `Why:` required; the reason keyword, its parenthetical, then upstream issues as bare references: `uniform-rule (same construct, same output: X; prettier/prettier#NNNN)`
  - `Pin:` required; the fixture path
  - `Conformance:` optional; the failing files of the owner's own suite
  - `Oxfmt:` optional, language crates only; the oxfmt externals failing on the entry
  - `Drop when:` optional; only for a condition more specific than the pin-bump default
  - Body: an input/ours/prettier code example
  - No status, dates, or authors: listed = accepted, condition met = delete (git holds history)
- Lifecycle
  - A new conformance failure is acceptable only under this policy; it stays counted as a failure
  - A failing file no entry lists is unclassified; the crate's `AGENTS.md` conformance section holds only those
  - At a pin bump, re-run conformance: an entry no longer observed is deleted, the rest are re-checked against their reason
- A documented divergence is never "fixed" by following Prettier
- `DIVERGENCES.md` holds entries only, opening with one back-reference to this section; `AGENTS.md` keeps policy translations and mechanism prose

## Comment placement invariants

Two layers of rules; know which one you are editing:

- Invariants hold uniformly: violating one is a bug even where Prettier disagrees
- Compat tables record measured Prettier behavior that is not derivable from principle: extend them by measuring Prettier, never by analogy, and pin every entry in a fixture

The invariants (placement only; losing a comment is the lossless contract under reason (2), above all of these):

- A comment never crosses user content (tokens and other comments): it stays on its source side of every token
- A comment never crosses a line boundary: line-based directives (`eslint-disable-line`, ...) must keep their meaning
  - Line comments print via `line_suffix`; own-line comments stay own-line
  - One exception, Prettier's stance since prettier/prettier#1267:
    - A same-line comment right after an opening delimiter (`{ // c`, `( # c`, `[ // c`) is the body's first leading comment and moves to its own line;
    - A `-line` directive there loses its target (`-next-line` variants do not)
    - Applies to every opener uniformly; where Prettier's attachment does something else for a particular opener, that is an artifact to pin as a divergence, not a table to extend
- A suppression comment (`prettier-ignore` / `oxfmt-ignore`) never loses its target, and its original text is preserved
- Repositioning is allowed only relative to formatter-OWNED punctuation: the formatter owns terminators (e.g. a statement's `;`) and the trivia up to them; the user owns content
  - Terminator vs separator: a terminator cannot be replaced by another token (`;` after a JS statement); a separator can (`,` / `;` between TS interface members)
    - Comments may move behind a terminator (per-language compat tables decide when); they always stay before a separator
      - Except a same-line line comment: it rides a `line_suffix` and lands just past the separator (`a // c\n, b` -> `a, // c`), the separator cannot follow it on the line;
      - an own-line comment leads what follows the separator instead
  - Grammar-fixed DELIMITER (braces, a head's parens, an argument list's parens) is neither: it bounds a region and stays user content, never crossed
  - (JS/TS) Redundant expression parentheses are NOT delimiters: the formatter drops them and re-derives parens by its own rules, so any paren in the output is formatter-owned
    - Trailing comment inside the dropped pair moves behind the terminator, even across a re-printed pair
    - The source pair stays user content only where a sub-printer claims it and prints the comment inside (per-language tables)
    - The line-boundary invariant still gates the move: never across a re-printed paren that ends up on its own line

Per-language translations (which tokens are terminators, the compat tables, cursor bounds disciplines) live in each crate's AGENTS.md.

## Text width

Printed text is measured by its display width (`TextWidth` / `text_width`, via `unicode-width`):

- East Asian wide and fullwidth characters, and emoji presentation sequences count as 2
- Zero-width characters count as 0: nonspacing and enclosing marks (`Mn` / `Me`), and format characters (`U+200B..U+200F`, `U+FEFF`, ...)
- A tab counts as `tabWidth` wherever it sits, not up to the next tab stop (Prettier counts it as 0, prettier/prettier#20381)
- Everything else counts as 1, ambiguous width included

This holds for the printer's line width and for every heuristic comparing text with a column budget (`tabWidth`, a share of `printWidth`, a short object key, ...),
never a byte or UTF-16 length, even where Prettier uses a JS `.length`.

## Error semantics

`format()` / `format_to_ir()` return `Err` whenever they cannot produce output they can stand behind:

- Any parse error bails out, even from an error-tolerant parser: never format a broken AST
- Print-stage internal errors are also `Err`
- The caller (oxfmt) decides what happens next

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
Fixture tests also assert the lossless contract through `FixtureFormatter::fingerprint` (`oxc_formatter_tests`): a loss fails the test instead of being pinned.

### Fixture tests

Snapshot tests driven by fixture files under `tests/fixtures/`; they cover what the Prettier conformance suite does not (suppression, divergence pins, embedded shapes, ...).
`build.rs` generates a test per fixture file: add a case by dropping a file in, no registration needed.
Options are resolved from the nearest `options.json` up the directory tree.

```sh
cargo test -p <crate>
# Review / accept snapshots after intentional changes
cargo insta test --accept -p <crate>
```

Every expected output must be verified against the oracle; in a pin fixture, every line except the ones its entry shows as deviating.

### Prettier conformance

Compares output against Prettier's snapshots and tracks failures (not passes).

Every language crate owns its conformance as a `tests/conformance.rs` target (via `oxc_formatter_tests::conformance`, report pinned with `insta`), part of the crate's regular `cargo test`:

```sh
cargo test -p <crate> --test conformance
# Debug a specific test
PRETTIER_FILTER=<path> cargo test -p <crate> --test conformance -- --nocapture
```

The Prettier suite lives under `crates/oxc_formatter_tests/prettier/` and is self-provisioned on the first conformance run.
It is gitignored, so use `rg --no-ignore` (or `-u`) when searching it.

### E2E conformance (`apps/oxfmt`)

The embedded-language features (e.g. xxx-in-js / js-in-xxx) are validated end-to-end through Oxfmt.
Requires a dev build first.
A language crate's fixtures cover only its own side (e.g. `format_to_ir` on a fragment): they run without a dispatcher, so an embedded body stays verbatim.

Some categories also run on real-world repositories; see `apps/oxfmt`'s `AGENTS.md`.
