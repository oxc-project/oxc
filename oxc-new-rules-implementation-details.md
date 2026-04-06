# Oxc New Rules Implementation Details

Generated: 2026-04-03

## Goal

Implement the missing lint surface from `bunclaw`, `bunpm`, and `abr-core` directly in native Oxlint, then upstream it as a sequence of small PRs on top of `oxlint-new-rules`.

This document focuses on implementation mechanics: where native rules live today, how to add more of them, how to group the researched plugin families, and how to sequence the work so we ship value early without blocking on CSS/JSON architecture.

## Ground truth in the current codebase

Relevant files:

- `crates/oxc_linter/src/rule.rs`
- `crates/oxc_linter/src/rules.rs`
- `crates/oxc_linter/src/config/plugins.rs`
- `crates/oxc_linter/src/config/oxlintrc.rs`
- `crates/oxc_linter/src/loader/mod.rs`
- `crates/oxc_linter/src/loader/partial_loader/mod.rs`
- `crates/oxc_linter/src/service/runtime.rs`
- `crates/oxc_linter/src/tester.rs`
- `crates/oxc_linter/src/table.rs`
- `tasks/rulegen`

Current implementation facts:

- Native rules are compiled into Oxlint and registered in `crates/oxc_linter/src/rules.rs`.
- Generated glue is rebuilt with `cargo lintgen`.
- Rules implement the `Rule` trait and usually use either:
  - `run` for AST-local checks
  - `run_once` for whole-file or graph-aware checks
  - `run_on_jest_node` for test-framework-specific checks
  - `should_run` for file-level gating
- Native plugin enablement is a `LintPlugins` bitflag in `config/plugins.rs`.
- `LintPlugins` is currently a `u16`; only two free bits remain, so new plugin families are not cheap.
- Cross-module analysis already exists for import-aware rules through `module_record`, `resolver`, and runtime `cross_module`.
- The loader is JS/TS-centric and only has partial loaders for `vue`, `astro`, and `svelte`.
- The native test harness is `Tester` in `crates/oxc_linter/src/tester.rs`, with first-class support for pass/fail/fix snapshots.

## Primary implementation modes

### 1. Add a rule to an existing native plugin family

This is the default path for most of the researched surface.

Mechanics:

1. Scaffold with `just new-rule <name> <plugin>` when possible.
2. Implement the rule under `crates/oxc_linter/src/rules/<plugin>/<rule>.rs`.
3. Register the module in `crates/oxc_linter/src/rules.rs`.
4. Run `cargo lintgen`.
5. Add rule tests with `Tester::new(...).test_and_snapshot()`.
6. Add fix tests with `.expect_fix(...)` if the rule advertises autofix.
7. Run `just fmt` and targeted `cargo test -p oxc_linter`.

Use this path for:

- `import/*`
- `promise/*`
- `node/*`
- `jsx-a11y/*`
- `react/*`
- `react-perf/*`
- `@typescript-eslint/*`
- `unicorn/*`

### 2. Add an Oxc-owned JS/TS rule under the `oxc` plugin

This is the best path for project-specific rules and ecosystem rules that do not justify a whole native plugin family yet.

Use this path for:

- `no-inline-types/no-inline-type-annotations`
- `require-schema-validation/json-parse-validation`
- `no-block-in-inline/no-block-in-inline`
- `check-file`-style file/path naming rules
- `no-secrets`
- selected `security` rules
- selected `depend` and `boundaries` rules
- selected `i18next` JS usage rules

Why this is the right default:

- the `oxc` plugin already exists as the home for Oxc-owned policy rules
- we avoid burning scarce native plugin bits
- upstream review is easier when the rule is clearly Oxc-owned rather than a partial reimplementation of an entire external plugin

### 3. Add a cross-module or graph-aware rule

Rules that need import resolution, package boundaries, or dependency ownership should use the existing import-aware runtime before any new subsystem is invented.

Implementation shape:

- prefer `run_once`
- read from `ctx.module_record()`
- enable cross-module tests with `Tester::with_import_plugin(true)`
- use resolver-backed absolute paths instead of raw import text whenever the semantics depend on the real module graph

Likely candidates:

- `import/no-unused-modules`
- `depend/*`
- `boundaries/*`
- stronger `barrel-files` coverage
- some `package-json` dependency checks

### 4. Add a test-framework rule

Rules that reason about `describe`, `it`, `test`, `expect`, or `.only` should use the existing Jest/Vitest hooks rather than live under a generic plugin.

Implementation shape:

- prefer `run_on_jest_node` when the rule is really about test-call semantics
- gate by plugin and test file handling already present in runtime
- add coverage with `Tester::with_jest_plugin(true)` or `with_vitest_plugin(true)`

This is the likely home for:

- `no-only-tests`
- missing focused-test restrictions
- some test-only naming/structure rules

### 5. Add a non-JS file rule

This is not a normal rule port. It requires new document parsing and new rule context.

Do not start here.

These belong to later waves:

- `json/*`
- `package-json/*`
- `i18n-json/*`
- `css/*`

## Mapping of researched families to native Oxc destinations

| Researched family    | Native destination                 | First implementation move                           | Notes                                                         |
| -------------------- | ---------------------------------- | --------------------------------------------------- | ------------------------------------------------------------- |
| `import`             | existing `import` plugin           | direct native rule ports                            | lowest-risk upstream work                                     |
| `promise`            | existing `promise` plugin          | direct native rule ports                            | many are AST-local                                            |
| `node` / `n`         | existing `node` plugin             | direct native rule ports                            | import/runtime-aware subset likely needs graph context        |
| `jsx-a11y`           | existing `jsx_a11y` plugin         | direct native rule ports                            | good early upstream candidates                                |
| `react`              | existing `react` plugin            | direct native rule ports                            | keep React-specific semantics together                        |
| `react-perf`         | existing `react_perf` plugin       | direct native rule ports                            | good fit for current structure                                |
| `@typescript-eslint` | existing `typescript` plugin       | direct native rule ports                            | prefer current TS semantic helpers                            |
| `unicorn`            | existing `unicorn` plugin          | direct native rule ports                            | many autofix opportunities                                    |
| `no-only-tests`      | existing `jest` / `vitest` plugins | add missing focused-test/native equivalents         | avoid separate plugin family                                  |
| `check-file`         | `oxc` plugin                       | add Oxc-owned path/file rules                       | path-based, repo-policy oriented                              |
| `depend`             | `oxc` plugin first                 | add graph-aware Oxc-owned rules                     | may later deserve a dedicated family only if it becomes broad |
| `no-secrets`         | `oxc` plugin                       | add high-signal secret patterns first               | avoid noisy regex-only shotgun ports                          |
| `security`           | `oxc` plugin first                 | port high-confidence correctness rules              | only split later if a coherent family emerges                 |
| `optimize-regex`     | `oxc` plugin                       | start with rules that have clear safe fixes         | regex analysis likely reusable across rules                   |
| `es-x`               | defer                              | decide target/runtime model first                   | not worth early PRs without target semantics                  |
| `barrel-files`       | existing `oxc` plugin              | extend `oxc/no_barrel_file` and related graph rules | already partially covered                                     |
| `boundaries`         | `oxc` plugin first                 | implement minimal architectural boundary checks     | likely settings-heavy                                         |
| `compat`             | defer                              | requires browser/runtime target data model          | architecture-first problem                                    |
| `package-json`       | later `json` substrate             | begin after native JSON pipeline exists             | not a JS rule                                                 |
| `perfectionist`      | defer                              | evaluate limited high-value subset                  | style-heavy and fixer-heavy                                   |
| `i18next`            | `oxc` plugin first                 | implement JS/TS usage rules first                   | no new loader needed for usage rules                          |
| `i18n-json`          | JSON + i18n layer                  | begin after JSON substrate exists                   | cross-file catalog rules                                      |
| `json`               | native JSON pipeline               | architecture work first                             | not a normal rule port                                        |
| `css`                | native CSS pipeline                | architecture work first                             | likely separate parser + context                              |
| `react-dom`          | `react` or `oxc` plugin            | evaluate rule by rule                               | only create a new family if upstream maintainers want it      |
| `react-rsc`          | `react` / `nextjs` / `oxc`         | place by semantics, not package name                | many rules are framework-targeted                             |
| `react-x`            | `react` or `oxc`                   | fold into existing React families where natural     | avoid plugin explosion                                        |
| local custom rules   | `oxc` plugin                       | port directly                                       | fastest route to zero-ESLint for JS/TS                        |

## Recommended waves

### Wave 1: direct ports into existing native families

This is the fastest path to useful upstream PRs.

Start with:

- missing `import/*`
- missing `promise/*`
- missing `node/*`
- missing `jsx-a11y/*`
- missing `react/*`
- missing `react-perf/*`
- missing `typescript/*`
- missing `unicorn/*`

Selection criteria:

- clear one-to-one mapping from an existing ESLint rule
- AST-local or modest `run_once` logic
- no new config schema beyond normal rule config
- no new loader or document kind
- ideally autofixable

### Wave 2: Oxc-owned JS/TS rules required by our repos

Target:

- `json-parse-validation`
- `no-inline-type-annotations`
- `no-block-in-inline`
- missing `.only` / test-focus protection if current Jest/Vitest coverage is insufficient
- high-value `check-file`, `no-secrets`, and `security` rules

These rules close the local migration gap without waiting for architecture work.

### Wave 3: graph-aware repository policy rules

Target:

- `depend`
- `boundaries`
- stronger barrel-file detection
- dependency ownership checks

These should reuse current module graph and resolver infrastructure before any larger runtime redesign.

### Wave 4: native JSON substrate

Target:

- `json`
- `package-json`
- `i18n-json`

This wave should start only after the JS/TS backlog above is moving, because it is architecture work first and rule work second.

### Wave 5: native CSS substrate

Target:

- plain `.css` rule support first
- no SCSS/LESS/styled-components expansion in the first milestone

This is the most expensive wave and should only start after maintainers agree with the direction.

## Autofix policy

Goal: everything autofixable that can be made safe and deterministic.

Use safe autofix by default for:

- import sorting and merging where semantics are preserved
- trivial syntax rewrites
- focused test removal when transformation is exact
- filename/path normalization diagnostics with exact rename suggestions only when the runtime can support them safely
- obvious React/JSX spelling or structural rewrites

Prefer suggestion-only or diagnostic-only for:

- security-sensitive rewrites
- rules that may change runtime behavior
- large import graph rewrites
- JSON or CSS edits before formatting preservation is robust
- repository-structure rules that imply file moves

Implementation rule:

- if a rule reports fix capability, `Tester` requires `.expect_fix(...)`
- fixes must be idempotent
- safe fixes should not depend on formatter cleanup to become valid

## What to touch for each new native rule

Minimum checklist:

1. add module file under `crates/oxc_linter/src/rules/<plugin>/`
2. register module in `crates/oxc_linter/src/rules.rs`
3. regenerate glue with `cargo lintgen`
4. add pass/fail tests
5. add fix tests if fixable
6. verify docs metadata in the `declare_oxc_lint!` block
7. run formatting and tests

If a new plugin family is ever added, also expect changes in:

- `crates/oxc_linter/src/config/plugins.rs`
- `crates/oxc_linter/src/config/oxlintrc.rs`
- generated schema/docs
- rule table/docs generation
- website/plugin documentation

Because `LintPlugins` is currently `u16`, any move beyond two additional native plugin families should first widen the bitflag type.

## Upstream PR slicing

Do not upstream this as one mega-branch.

Recommended slicing:

1. small one-rule or small related-rule PRs in existing native families
2. local custom rules in `oxc`
3. graph-aware Oxc rules
4. JSON architecture RFC-style PRs
5. JSON rule PRs
6. CSS architecture only after maintainer alignment

Each PR should include:

- native docs in the rule comment block
- rule tests and fix tests
- AI usage disclosure
- explicit notes on whether behavior matches upstream ESLint exactly or intentionally differs
