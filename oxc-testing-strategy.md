# Oxc New Rules Testing Strategy

Generated: 2026-04-03

## Goal

Define how to test the new native Oxlint rule work so that:

- behavior is correct
- autofixes are safe
- performance stays aligned with Oxlint goals
- upstream PRs are easy to review and trust

This strategy covers both the near-term JS/TS rule ports and the later JSON, CSS, and i18n substrate work.

## Testing principles

1. Prefer small, local tests for each rule first.
2. Add fix tests for every rule that reports autofix capability.
3. Add graph-aware tests only when the rule truly depends on resolution or workspace state.
4. Reuse existing conformance infrastructure where it helps validate parity.
5. Add performance checks before and after each batch of rules, not only at the end.
6. Keep PRs narrow enough that test failures point to one logical change.

## Existing testing surface in Oxc

Relevant files:

- `crates/oxc_linter/src/tester.rs`
- `apps/oxlint/conformance/README.md`
- `justfile`

Important current facts:

- `Tester` is the standard native rule test harness.
- `Tester::test_and_snapshot()` runs pass/fail/fix logic and snapshot output.
- if a rule advertises fix capability but no fix cases are provided, `Tester` fails the test.
- cross-module tests can enable the import plugin with `with_import_plugin(true)`.
- Jest/Vitest rules can use `with_jest_plugin(true)` or `with_vitest_plugin(true)`.
- conformance infrastructure already exists for JS plugins and upstream rule fixtures.

## Layer 1: rule-local native tests

Every native rule PR should start here.

Use:

- `Tester::new(rule_name, plugin_name, pass, fail)`
- `.expect_fix(fix_cases)` when fixable
- `.test_and_snapshot()`

Required coverage:

- obvious valid cases
- obvious invalid cases
- edge cases from upstream fixture behavior
- configuration variants
- TS/JSX/test-file path variants when relevant

For whole-file rules using `run_once`, include:

- empty file
- one-hit case
- many-hit case
- cases where early exits should suppress diagnostics

## Layer 2: autofix tests

Autofix is a first-class goal, so fix testing must be explicit.

Each fixable rule should test:

- the happy path fix
- multiple fix candidates if the rule supports them
- comments and formatting preservation when relevant
- idempotence on already-fixed code
- cases where the rule should diagnose but intentionally not fix

Recommended fix acceptance rule:

- safe fixes should never produce syntactically invalid output
- safe fixes should not require a formatter pass to become valid

If a transformation may change behavior, prefer:

- suggestion-style fix metadata
- or diagnostic-only behavior

## Layer 3: graph-aware tests

Use graph-aware tests only for rules that need them.

Applies to:

- `import` graph rules
- `depend`
- `boundaries`
- barrel-file detection
- dependency ownership or package-edge checks

Mechanics:

- enable cross-module behavior with `with_import_plugin(true)`
- set deterministic fixture paths with `change_rule_path(...)`
- keep fixtures small and explicit so resolver behavior is obvious

Required graph test coverage:

- resolved import success
- missing import or unresolved path behavior
- path alias or package edge cases if supported
- cycle or transitive graph cases
- non-module files that should be ignored

## Layer 4: test-framework-specific tests

For Jest/Vitest-related rules:

- use the framework-specific plugin toggles
- use test-file extensions when the rule depends on test-file detection
- cover `describe`, `it`, `test`, `expect`, `.only`, `.skip`, and modifiers where relevant

This is especially important for replacing `no-only-tests` and focused-test checks.

## Layer 5: conformance and parity tests

Use conformance selectively when parity with existing ecosystems matters.

Useful commands and setup:

- `pnpm -C apps/oxlint run build-conformance`
- `pnpm -C apps/oxlint run init-conformance`
- `pnpm -C apps/oxlint run conformance`

Use conformance for:

- validating ports against upstream ESLint plugin behavior
- checking whether a native port intentionally diverges
- harvesting upstream fixtures before or during a port

Do not require full conformance runs for every tiny PR unless that PR directly affects the conformance harness.

## Layer 6: JSON and CSS substrate tests

When JSON and CSS work begins, testing must expand beyond rule-local coverage.

### JSON substrate tests

Required layers:

- parser golden tests
- span accuracy tests
- malformed input recovery tests
- duplicate-key behavior tests
- fixer round-trip tests
- file-kind loader tests for `.json` and, if supported, `.jsonc`

Rules built on JSON should then add normal pass/fail/fix rule tests on top.

### CSS substrate tests

Required layers:

- parser golden tests
- selector/declaration span tests
- malformed stylesheet recovery tests
- at-rule coverage
- fixer safety tests
- loader tests for `.css`

Do not start rule-level CSS work before these foundations are trustworthy.

## Layer 7: i18n testing

I18n should be tested in two groups.

### JS/TS usage rules

Test:

- configured translation functions
- hooks and components
- namespace handling
- forbidden raw strings
- allowed literal-text exceptions

### Catalog rules

Test:

- missing keys by locale
- extra keys
- placeholder mismatches
- nested object structure mismatches
- invalid translation values
- deterministic ordering of diagnostics across files

Cross-file catalog tests should use tiny, explicit locale fixture sets.

## Layer 8: performance testing

Because the goal is to replace ESLint, performance regression checks are mandatory.

Recommended checks:

- targeted benchmark before and after each major rule batch
- cold and warm CLI timings on:
  - `bunclaw`
  - `bunpm`
  - `abr-core`
- memory usage observation on representative monorepos

Useful commands:

- `cargo build -p oxlint --release --features allocator`
- `just benchmark-one linter`
- targeted repo timing runs with the release binary

Performance review questions for every PR batch:

- does the rule require `run_once` when `run` would be cheaper
- can AST node filtering via generated type info avoid unnecessary work
- does a fix or graph walk allocate more than needed
- does a config-heavy rule add expensive per-file setup

## Layer 9: integration and documentation checks

Before upstreaming a rule family batch, also verify:

- `cargo lintgen` was run if rule registration changed
- generated code is committed
- docs in `declare_oxc_lint!` are accurate
- config schema remains valid
- snapshots are updated intentionally

Repo-level commands:

- `just fmt`
- `cargo test -p oxc_linter`
- `just test`
- `just lint`
- `just ready` before a serious upstream push

## PR-level test checklist

Every rule PR should satisfy:

1. targeted native rule tests pass
2. fix tests exist for every fixable rule
3. snapshots are reviewed, not blindly updated
4. docs metadata matches actual behavior
5. performance impact is understood

Every architecture PR for JSON/CSS should satisfy:

1. parser and span tests exist
2. loader/file-kind tests exist
3. at least one rule proves the substrate is usable
4. fix behavior is tested end-to-end
5. runtime cost is measured

## Recommended cadence

### For small JS/TS rule ports

- run targeted `cargo test -p oxc_linter <filter>`
- run `just fmt`
- before opening PR, run `cargo test -p oxc_linter`

### For family batches or graph-aware work

- run `cargo test -p oxc_linter`
- run targeted conformance if parity matters
- run a release-binary smoke test on one real repo

### For JSON/CSS substrate work

- run parser tests
- run linter tests
- run targeted integration tests
- run performance checks on representative inputs

## Failure patterns to watch for

Common risks that tests must catch:

- fixes that overlap or apply in unstable order
- diagnostics that move when spans are converted incorrectly
- graph rules that depend on nondeterministic filesystem order
- plugin gating errors where a rule runs in the wrong file kind
- snapshot churn caused by incidental diagnostic wording changes
- performance cliffs from whole-file scans or repeated resolver work

## Bottom line

The testing strategy should mirror the delivery strategy:

- local native rule tests for fast progress
- fix tests for every safe autofix
- graph tests only where needed
- conformance as a parity backstop
- substrate tests before JSON/CSS rules
- performance checks throughout, not at the end

That gives us confidence that native Oxlint expansion stays both correct and fast enough to replace ESLint in the target monorepos.
