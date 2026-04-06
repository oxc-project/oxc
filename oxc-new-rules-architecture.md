# Oxc New Rules Architecture

Generated: 2026-04-03

## Goal

Define the native Oxlint architecture needed to absorb the researched lint surface without relying on ESLint or JS plugins as the end state.

The architecture should optimize for:

- Oxlint-native speed
- small, reviewable upstream PRs
- reuse of existing rule/runtime infrastructure
- a clean path to JSON, CSS, and i18n support without destabilizing JS/TS linting

## Existing architecture constraints

Relevant code:

- `crates/oxc_linter/src/lib.rs`
- `crates/oxc_linter/src/rule.rs`
- `crates/oxc_linter/src/config/plugins.rs`
- `crates/oxc_linter/src/loader/mod.rs`
- `crates/oxc_linter/src/loader/partial_loader/mod.rs`
- `crates/oxc_linter/src/service/runtime.rs`

Current facts that matter:

- the core runtime assumes JS/TS parsing and semantic analysis
- partial loaders only extract JS/TS from `vue`, `astro`, and `svelte`
- `ProcessedModule`, `ModuleContent`, `SectionContent`, parser token flow, and semantic flow are all JS-specific
- built-in plugins are static bitflags, not dynamically discovered native plugins
- `LintPlugins` is `u16`, which strongly discourages plugin-family sprawl
- JS plugins already exist, but they are a compatibility layer, not the architecture we want to deepen

## Architectural principles

### 1. Keep the current JS/TS pipeline intact for the first waves

Most missing coverage does not require new architecture.

That means:

- port rules into existing native families first
- place local or policy-heavy JS/TS rules in `oxc`
- reuse current resolver and module graph for graph-aware rules

This preserves performance and lets upstream work begin immediately.

### 2. Add new native plugin families only when the document kind truly changes

Because plugin bits are scarce and each new family expands config/docs surface, the default should be:

- existing family if semantics match
- `oxc` family if the rule is Oxc-owned or cross-cutting

New families should be reserved for cases where the language substrate itself is new:

- likely `json`
- likely `css`

Avoid early native families for:

- `i18n`
- `security`
- `check-file`
- `depend`
- `boundaries`

Those can live under `oxc` first.

### 3. Treat JSON and CSS as document-pipeline work, not as rule-port work

These require more than rules:

- file detection
- parsing with spans
- diagnostics
- fixes
- test harness support
- config and docs surface

### 4. Split i18n into JS usage rules and catalog rules

I18n support is really two different problems:

- JS/TS usage rules:
  - translation-call patterns
  - forbidden literal text
  - namespace usage
  - required key patterns
- catalog rules:
  - missing keys
  - mismatched placeholders
  - inconsistent locale structure
  - invalid translation values

The first fits today’s JS pipeline.
The second needs JSON support and likely workspace indexing.

## Proposed native architecture by phase

## Phase A: expand the JS/TS-native surface

No new architecture required.

Use the current stack:

- `Rule` trait
- generated `RuleEnum`
- `LintContext`
- `module_record`
- current resolver/runtime
- `Tester`

This phase covers:

- `import`
- `promise`
- `node`
- `jsx-a11y`
- `react`
- `react-perf`
- `typescript`
- `unicorn`
- custom Oxc-owned JS/TS rules

## Phase B: formalize Oxc-owned policy rules under `oxc`

This is a policy and maintainability decision more than a runtime change.

Recommended placement:

- `check-file`
- `no-secrets`
- selected `security`
- selected `depend`
- selected `boundaries`
- local custom rules
- JS-facing `i18next` rules

Benefits:

- avoids a proliferation of mostly-empty native families
- keeps Oxc-owned semantics clearly separated from pure upstream ESLint parity ports
- buys time before widening `LintPlugins`

## Phase C: introduce a native JSON linting substrate

This is the first real architectural expansion.

### Recommended scope for JSON v1

Support:

- `.json`
- optionally `.jsonc` if maintainers agree
- `package.json`
- translation catalogs used by `i18n-json`

Do not attempt YAML or TOML in the first milestone.

### Recommended JSON architecture

Create a JSON-native pipeline with:

1. file kind recognition
2. parser with span-preserving AST
3. JSON-specific lint context
4. fixer support on JSON spans
5. rule registration under a native family, most likely `json`

Minimum AST shape:

- document
- object
- property
- array
- string
- number
- boolean
- null

Capabilities the parser must preserve:

- exact key/value spans
- stable source slicing for diagnostics
- enough trivia handling to support minimal safe fixes

### Why `serde_json` alone is not enough

For linting and fixing we need:

- source spans
- key spans
- value spans
- duplicate-key handling with diagnostics on both entries
- controlled source rewriting

That points to a dedicated JSON parser or a new Oxc JSON crate, not just config deserialization.

### JSON runtime integration recommendation

Do not immediately refactor the entire JS runtime into a generic multi-language engine.

Instead:

- add document-kind dispatch near loader/runtime entry
- keep JS/TS processing unchanged
- route JSON files through a parallel JSON-specific lint path that still returns the same top-level diagnostic/fix result shape

This gives us a smaller upstream diff and less risk to JS performance.

## Phase D: add i18n catalog support on top of JSON

After JSON exists, add an i18n layer with two pieces.

### 1. JS/TS usage rules

These can be added earlier, before JSON.

Needed settings likely include:

- translation function names
- hook names
- component names
- namespace conventions
- allowed literal-text zones

### 2. Catalog rules

These depend on JSON plus workspace indexing.

Likely architecture:

- discover locale files
- normalize locale identifiers and namespaces
- index key paths by locale
- compare key sets and placeholder signatures

This likely belongs in `oxc` or `json` first, not a separate `i18n` family.

## Phase E: introduce a native CSS linting substrate

CSS is the largest architecture step.

### Recommended CSS v1 scope

Support only:

- plain `.css`

Defer:

- SCSS
- LESS
- CSS-in-JS
- framework-embedded style sections

### Recommended CSS architecture

As with JSON, treat CSS as a new document kind with:

1. file kind recognition
2. CSS parser with span-preserving AST
3. CSS-specific rule context
4. fixer support
5. native plugin family, most likely `css`

Likely AST needs:

- stylesheet
- rule
- selector
- declaration
- at-rule
- value fragments

### Why CSS should be later than JSON

- there is no current CSS linter substrate in `oxc_linter`
- CSS semantics are much broader than JSON
- selector/value parsing and fix safety are harder
- maintainers may prefer a dedicated design discussion before accepting the substrate

## Cross-module and workspace services

Several researched families need workspace-level knowledge even within JS/TS:

- `depend`
- `boundaries`
- `package-json`
- `i18n-json`

Recommended architectural rule:

- centralize workspace indexes in reusable services
- do not let each rule crawl the filesystem ad hoc

Near-term recommendation:

- reuse `module_record` and resolver for JS import graphs
- add a separate lightweight JSON/i18n catalog index later
- keep indexes deterministic and path-normalized for stable tests

## Plugin-family strategy

Recommended family strategy:

- existing families absorb parity ports
- `oxc` absorbs Oxc-owned policy and custom rules
- `json` becomes the first new family, if JSON support lands
- `css` becomes the second new family, if CSS support lands
- widen `LintPlugins` from `u16` to `u32` only when this becomes necessary

This is intentionally conservative. It minimizes config churn and keeps the upstream story coherent.

## Configuration strategy

For JS/TS phases, use existing `plugins`, `rules`, and `settings`.

For JSON/CSS phases:

- add native plugin names only when the substrate is ready
- prefer rule-level config first
- add top-level settings only when several rules truly share them

Likely future settings buckets:

- `i18n`
- `boundaries`
- `depend`
- `json`
- `css`

But these should not be added speculatively.

## Documentation and generation implications

Any architecture change that adds a new family or document kind will likely require follow-up in:

- `config/plugins.rs`
- `config/oxlintrc.rs`
- rule table generation
- configuration schema
- website docs
- migration docs

If the work stays inside existing families or `oxc`, the architecture cost is much lower.

## Recommended sequence

1. Land missing rules in existing native families.
2. Land local custom rules and high-value policy rules in `oxc`.
3. Reuse module graph support for `depend`, `boundaries`, and barrel-file work.
4. Write a focused JSON architecture proposal and get maintainer agreement.
5. Land JSON substrate in small foundational PRs.
6. Add `package-json` and `i18n-json` rules on top of JSON.
7. Only then propose CSS substrate.

## Non-goals for the first milestone

Do not do these early:

- fork the entire runtime into a generic all-languages framework
- create many new native plugin families just to mirror npm package names
- attempt CSS-in-JS and plain CSS at the same time
- mix JSON architecture work into the first wave of JS/TS rule ports
- hold back easy upstream rule PRs until JSON/CSS design is complete

## Bottom line

The native architecture should stay conservative:

- JS/TS rule growth happens in the current runtime
- Oxc-owned policy rules go into `oxc`
- JSON and CSS are explicit substrate projects
- i18n is split into JS usage and JSON catalog layers

That gives us the fastest path to upstream value without painting Oxlint into an architectural corner.
