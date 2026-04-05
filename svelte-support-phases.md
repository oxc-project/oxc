# Svelte support phases

This file tracks the work completed so far to add proper Svelte support to Oxlint/Oxfmt.

## Current status snapshot

The current tree is in the **beta compatibility** stage:
- Oxlint has real-package Svelte coverage in CI for pinned packages, plus a latest-upstream canary.
- Oxfmt has the same pinned/canary structure for `prettier-plugin-svelte` + `svelte`.
- The implementation is still a hybrid interoperability path rather than native Rust Svelte parsing / formatting.

See [SVELTE_SUPPORT.md](./SVELTE_SUPPORT.md) for the current support framing, rollout guidance, and local commands.

For final rollout execution, use [SVELTE_BETA_SOAK.md](./SVELTE_BETA_SOAK.md). For external messaging, use [SVELTE_ANNOUNCEMENT_TEMPLATE.md](./SVELTE_ANNOUNCEMENT_TEMPLATE.md).

## Phase 0 — Native Svelte loader hardening


Changes:
- tightened `partial_loader/svelte.rs` so `<script>` handling uses real attribute parsing instead of a broad `"ts"` substring check
- detect both `module` and `context="module"`
- infer TypeScript from `lang="ts"`
- added unit tests for module scripts, TS scripts, combined module+TS scripts, and false positives like `data-language="ts"`
- added a CLI integration fixture for `.svelte` files with module + instance TypeScript scripts


## Phase 1 — Config blockers for Svelte migration


Changes:
- added string/package entries to `extends` for `oxlint.config.ts`
- added category-level `"recommended"` support so categories can enable Oxlint's built-in recommended subset instead of every rule in the category
- preserved explicit category settings when later plugin/override processing runs
- updated JS config parsing tests, config typings, and builder coverage


## Phase 2 — External parser foundation for JS plugins


Changes:
- added the generic external/custom parser foundation for JS plugins
- taught `RuleTester` to use `parseForESLint()` with `parse()` fallback
- plumbed parser-owned metadata for external ASTs (`parserServices`, `visitorKeys`, `scopeManager`)
- added normalization needed for external AST traversal
- added RuleTester coverage for direct external node visitors and selectors involving external-only ancestors


## Phase 3A — Preserve JS-config `languageOptions` through config resolution


Changes:
- added a JS-side `languageOptions` registry to preserve non-serializable parser objects and parser options loaded from `oxlint.config.ts`
- normalized JS config output so root configs, nested `extends`, and overrides send internal `_languageOptionsId` markers to Rust instead of attempting to JSON-serialize parser objects
- added internal `_languageOptionsHasParser` tracking so Rust can tell whether the resolved JS-side `languageOptions` select a custom parser
- extended Rust config parsing to accept the internal language-options IDs and parser-presence flag on root configs and overrides
- carried resolved language-options IDs and parser-presence state through `LintConfig`, override application, and the external linter callback boundary
- exposed the resolved parser/parserOptions back to JS rules through `context.languageOptions`
- added tests for JS-config parsing of `_languageOptionsId` / `_languageOptionsHasParser`, merge behavior for parser-presence state, and ordered accumulation of override language-options IDs
- added a lightweight shared parser type for config typing so `defineConfig(...)` no longer depends on the full plugin runtime type graph

Notes:
- this does **not** yet select `svelte-eslint-parser` for whole `.svelte` files
- this is the plumbing needed before the whole-file Svelte parser lane can be added


## Phase 3B — Whole-file `.svelte` parser lane

Status: in progress; narrow plumbing slice is close to PRable

Changes:
- added Rust-to-JS transport for whole-file source text so JS plugins can lint files without relying on the raw-transfer AST buffer path
- shared external-rule diagnostic/fix handling between the raw-transfer path and the new whole-file source-text path
- taught the JS plugin runtime to run the configured custom parser against whole-file source text and populate `sourceCode.text`, `parserServices`, `visitorKeys`, and `scopeManager` from parser output
- added `setupExternalSourceForFile(...)` and external-parser source setup in the JS runtime so rules can operate on a custom-parser AST without a raw-transfer buffer
- added a canary fixture `apps/oxlint/test/fixtures/js_config_svelte_whole_file` proving a `.svelte` file can expose original markup and parser services to a JS plugin rule
- tightened the surrounding config tests while landing this work, including regression coverage for the custom-parser flag propagation

Notes:
- this slice still does **not** yet route the real CLI/runtime to `svelte-eslint-parser`; the fixture uses an inline parser stub to prove the whole-file lane works
- CFG listeners are still unsupported on the whole-file custom-parser path

## Phase 3C — Whole-file custom-parser routing for `.svelte` files without scripts

Status: in progress; narrow runtime slice is close to PRable

Changes:
- added an external-only runtime path so files with a configured custom parser can still run JS plugin rules even when the native partial loader produces no script sections
- added `ContextHost::new_external_only(...)` and `Linter::run_external_only_on_source_text(...)` so resolved settings, globals, diagnostics, and fixes still flow through the existing external-rule machinery
- wired `Runtime::run`, `run_source`, and `run_test_source` to use that path instead of returning early when a file has no native script sections
- added a `.svelte` no-script canary fixture proving the whole-file parser lane works for markup-only Svelte components

Notes:
- disable-directive matching for this no-script whole-file path is still incomplete because Rust does not yet receive parser comments from the external framework parser
- CFG listeners are still unsupported on the whole-file custom-parser path

## Phase 4 — Parser metadata/runtime integration

Status: partially done in Phase 2 and Phase 3B

Done already:
- external parser metadata plumbing exists in the JS runtime foundation work
- whole-file custom-parser runs now carry parser-owned metadata through the JS plugin runtime

Still needed for Svelte:
- full CLI/runtime integration for `.svelte`
- remaining traversal/runtime gaps once real Svelte parser fixtures are wired in

## Phase 4A — External parser comment/token `SourceCode` APIs


Changes:
- added external-parser setup helpers in the JS runtime so whole-file custom-parser files populate `sourceCode` comment/token state without relying on the Rust raw-transfer buffer
- whole-file custom-parser runs now normalize and retain external `ast.comments` and `ast.tokens` (with fallback to parser-result-level `comments` / `tokens` metadata when present)
- existing `SourceCode` comment/token methods now work on externally parsed files through the same cached-object and merged-order machinery used by native files
- added a markup-only `.svelte` canary fixture proving `getAllComments`, `getCommentsBefore`, `getTokens`, `tokensAndComments`, and `ast.comments` / `ast.tokens` identity all work through the whole-file custom-parser lane

Notes:
- this improves JS-plugin compatibility for framework parsers that provide ESTree comments/tokens metadata
- disable-directive matching on the Rust side is still incomplete for whole-file framework parsers because Rust does not yet receive parser comments back from JS


## Phase 5A — Type-aware parserOptions / `svelteConfig` passthrough canary


Changes:
- added a new whole-file Svelte canary fixture `apps/oxlint/test/fixtures/js_config_svelte_type_aware_whole_file`
- the canary routes a `.svelte` file through the whole-file custom-parser lane with:
  - top-level Svelte parser on the override
  - nested TypeScript parser in `parserOptions.parser`
  - imported `svelteConfig` object in `parserOptions.svelteConfig`
  - `projectService` and `extraFileExtensions` on the matching override
- the fixture proves all of those values survive config loading and merge correctly across an object-style `extends` chain before reaching the parser and rule runtime
- the canary also proves non-serializable nested values are preserved, including parser functions and `svelteConfig.preprocess`
- added a direct JS unit test for `resolveLanguageOptionsIds(...)` covering deep parser-options merges and preservation of non-serializable nested values
- expanded `apps/oxlint/test/config.test-d.ts` with a more realistic type-aware Svelte config shape using object-style `extends`

Notes:
- this is still a canary/stub-parser slice; it does not yet run the real `svelte-eslint-parser` package end to end
- the runtime path now has explicit coverage for the data shape Svelte expects, which lowers the risk of the later real-parser integration work


## Phase 5B — Parser-provided scope methods for whole-file custom parsers


Changes:
- taught `SourceCode` scope helper methods to prefer the parser-provided `scopeManager` when whole-file custom-parser files supply one
- `sourceCode.getScope(...)`, `getDeclaredVariables(...)`, `markVariableAsUsed(...)`, and `isGlobalReference(...)` now use the active parser scope manager instead of always rebuilding fallback scope data from the Oxc AST
- preserved the existing fallback TS-ESLint scope analysis path for native files and files without an external parser scope manager
- added a whole-file `.svelte` canary fixture `apps/oxlint/test/fixtures/js_config_svelte_scope_methods_whole_file`
- the canary proves whole-file custom-parser runs can use parser-provided scope data through:
  - `sourceCode.scopeManager`
  - `sourceCode.getScope(...)`
  - `sourceCode.getDeclaredVariables(...)`
  - `sourceCode.isGlobalReference(...)`
  - `sourceCode.markVariableAsUsed(...)`

Notes:
- this closes an important compatibility gap for framework parsers such as `svelte-eslint-parser`, which return a custom `scopeManager` for virtual/template scopes
- this is still a stub-parser canary slice; it does not yet run the real `svelte-eslint-parser` package end to end


## Phase 5C — Whole-file disable directives from external parser comments


Changes:
- extended the JS external-linter payload so whole-file custom-parser runs can round-trip parser comments back to Rust alongside diagnostics
- added Rust-side reconstruction of directive comment spans from external parser comments, including HTML comments like `<!-- eslint-disable-next-line ... -->`
- taught `handle_external_linter_result(...)` to consult those reconstructed whole-file directives before reporting JS-plugin diagnostics
- generalized `DisableDirectivesBuilder` with `build_raw_comments(...)` so external parser comments can reuse the existing directive parser without needing native Oxc `Comment` structs
- added a unit test proving `eslint-disable-next-line` works from an HTML-style comment
- added a markup-only `.svelte` fixture `apps/oxlint/test/fixtures/js_config_svelte_disable_directives_whole_file` proving a whole-file custom-parser diagnostic is suppressed by an HTML disable directive

Notes:
- this closes the main runtime gap where `.svelte` whole-file parser diagnostics ignored disable directives in template comments
- it still does not add Rust-side unused-disable reporting/fixes for whole-file framework-parser comments


## Phase 5D — Package-shaped Svelte ecosystem whole-file canary


Changes:
- added a new whole-file Svelte canary fixture `apps/oxlint/test/fixtures/js_config_svelte_package_ecosystem_whole_file`
- the fixture uses package-shaped local `node_modules` entries for both `svelte-eslint-parser` and `eslint-plugin-svelte`, instead of inline stubs
- the config imports the parser from the package and extends a recommended config object exported by the plugin package
- the plugin rule proves package-loaded framework integrations can see the Svelte-specific parser surface Oxlint needs to preserve, including:
  - `parserServices.isSvelte`
  - `parserServices.svelteParseContext`
  - `parserServices.getStyleContext()`
  - `context.settings.svelte.compileOptions`
  - `context.settings.svelte.kit`
  - nested `parserOptions.parser`
  - `projectService`
  - `extraFileExtensions`
  - imported `svelteConfig.preprocess`
- the fixture also proves package-name normalization still yields the expected `svelte/valid-compile` rule ID on the whole-file custom-parser path

Notes:
- this is still a package-shaped canary, not the real upstream `svelte-eslint-parser` / `eslint-plugin-svelte` packages
- it is the first end-to-end fixture in this stack that exercises the same import and package-resolution shape real Svelte projects use


## Phase 5E — Report unused whole-file disable directives from external parser comments


Changes:
- extended raw directive comment handling so whole-file custom-parser comments can keep both the full outer comment span and the inner content span used for rule-name parsing
- whole-file external parser comments now build disable directives with the original HTML comment span, so unused-directive diagnostics and suggested removals target the full `<!-- ... -->` comment
- `run_external_rules_on_source_text(...)` now returns reconstructed whole-file disable directives after it uses them to suppress JS-plugin diagnostics
- whole-file custom-parser linting now reports unused disable/enable directives in both paths:
  - files with native script sections plus whole-file JS-plugin parsing
  - external-only files with no native script sections
- added disable-directive tests proving HTML comment directives preserve the outer comment span for unused-directive reporting
- added a new `.svelte` CLI canary fixture `apps/oxlint/test/fixtures/js_config_svelte_unused_disable_directives_whole_file`
  with `--report-unused-disable-directives` coverage for a markup-only whole-file parser run

Notes:
- this closes the remaining practical gap where whole-file `.svelte` parser comments could suppress diagnostics but could not themselves be reported as unused
- the fixture currently snapshots warning output; it does not yet add an end-to-end `--fix-suggestions` canary for removing the unused comment



## Phase 5F — Whole-file `.svelte` fixes and fix-suggestions canary


Changes:
- added a new package-shaped whole-file Svelte fixture `apps/oxlint/test/fixtures/js_config_svelte_fixes_suggestions_whole_file`
- the fixture uses local `svelte-eslint-parser` and `eslint-plugin-svelte` packages, mirroring the package import shape real Svelte projects use
- the parser returns external-only Svelte template nodes (`SvelteClassName` and `SvelteText`) through the whole-file custom-parser lane
- the plugin exposes a single rule that has both safe fixes and suggestions, so the fixture now snapshots all three modes on `.svelte` input:
  - normal lint output
  - `--fix`
  - `--fix-suggestions`
- the fix canary proves a whole-file custom-parser rule can safely rewrite original Svelte markup ranges
- the suggestion canary proves `--fix-suggestions` applies the first suggested edit on the original whole-file `.svelte` source text

Notes:
- this is still a package-shaped canary, not the real upstream `svelte-eslint-parser` / `eslint-plugin-svelte` packages
- the value of this slice is locking down edit application on the whole-file framework-parser lane, which had runtime coverage but no dedicated Svelte fix snapshots yet


## Phase 5G — Pass ESLint parser feature-detection flags on whole-file Svelte parser runs


Changes:
- taught whole-file custom-parser calls to always pass `eslintVisitorKeys: true` and `eslintScopeManager: true` alongside `filePath`
- mirrored the same behavior in `RuleTester`, so custom-parser tests see the same parser-call contract as the runtime
- added RuleTester coverage proving those flags are present even if user-supplied `parserOptions` tried to set them to `false`
- added a new package-shaped whole-file Svelte canary fixture `apps/oxlint/test/fixtures/js_config_svelte_parser_feature_flags_whole_file`
- the canary proves a package-loaded Svelte parser sees:
  - `eslintVisitorKeys: true`
  - `eslintScopeManager: true`
  - the expected `filePath`
- the canary also proves the file still reaches the whole-file parser lane and reports through a package-loaded Svelte rule

Notes:
- this is a small but important compatibility slice for ESLint custom parsers that use those parserOptions flags for feature detection
- the goal is to make the generic whole-file parser lane look more like normal ESLint, not to add another Svelte-specific special case


## Phase 5H — Pass core AST metadata flags to whole-file custom parsers


Changes:
- added a shared `createRequiredParserCallOptions(...)` helper so the runtime and `RuleTester` no longer drift on custom-parser call options
- whole-file custom-parser calls now always force the core AST metadata flags ESLint-style parsers commonly rely on:
  - `loc: true`
  - `range: true`
  - `raw: true`
  - `comment: true`
  - `tokens: true`
- mirrored the same behavior in `RuleTester`, including overriding user-supplied `false` values for those flags
- added `RuleTester` coverage proving those AST metadata flags are always passed to custom parsers
- added a new package-shaped whole-file Svelte canary fixture `apps/oxlint/test/fixtures/js_config_svelte_parser_ast_metadata_flags_whole_file`
- the canary proves a package-loaded Svelte parser sees all five flags and can return parser-generated comments/tokens that flow through `SourceCode` APIs on the whole-file lane

Notes:
- this is a generic custom-parser compatibility improvement, not a Svelte-only special case
- it makes the whole-file parser lane look more like normal ESLint for parsers that gate comment/token/raw/loc/range output on parser-call options


## Phase 5 — Type-aware Svelte support

Status: in progress

Remaining scope:
- wire the real `svelte-eslint-parser` package into end-to-end canaries
- carry any remaining parser-specific settings required by real Svelte ecosystem rules
- add stronger coverage once actual type-aware rules are exercised instead of stub-parser canaries
- finish any remaining whole-file runtime gaps that only surface with real Svelte parser output

## Phase 6 — Formatter support

Status: in progress

Planned scope:
- route `.svelte` formatting through formatter plugin language resolution
- integrate `prettier-plugin-svelte` style plugin discovery on the formatter side

## Phase 6A — Oxfmt plugin language discovery and plugin-option passthrough


Changes:
- added `ExternalPluginSupport` in Oxfmt so external formatter plugin languages can advertise parser names by extension and exact filename
- added `FormatFileStrategy::from_path_with_external_support(...)` and taught CLI/stdin/LSP/API entry points to use plugin language support when choosing how to format a file
- changed external formatter initialization so Rust asks JS to resolve a specific list of configured plugin specs and receive their serialized `languages` metadata back
- preserved top-level external formatter config fields when building Prettier options, instead of dropping unknown plugin-owned fields during `FormatConfig` serialization
- resolved relative plugin paths against the config directory (or API cwd) and preserved them in external formatter options so workers can load the actual plugins during formatting
- taught the JS formatter runtime to load configured plugins from `options.plugins` before formatting, while still separately resolving plugin language metadata for the Rust-side file walker
- added Rust tests for:
  - plugin-spec path resolution
  - preserving `plugins` and plugin-owned top-level options in external formatter options
  - routing `.svelte` files through plugin language metadata
- added new CLI and API canaries using a local fake `prettier-plugin-svelte` package/file:
  - the CLI canary proves `.svelte` files can be discovered and formatted from config-loaded plugin languages
  - the API canary proves direct `format("App.svelte", ...)` works with `plugins: [pluginPath]`
  - both canaries prove a plugin-owned option (`svelteSortOrder`) survives into the formatter call and affects output

Notes:
- this slice is formatter-side plumbing; it does not add a real upstream `prettier-plugin-svelte` dependency
- the fake Svelte plugin is intentionally minimal and exists only to lock down file-type discovery and plugin-option passthrough behavior



## Phase 6B — Package-name formatter plugin resolution from project `node_modules`


Changes:
- stopped treating every plugin spec containing `/` or `\` as a filesystem path during formatter config normalization
- relative and absolute filesystem plugin paths are still normalized, but package names, scoped packages, and package subpaths are now preserved as package specs
- encoded package-style plugin specs with an internal `resolveFrom` base directory so the JS formatter runtime can resolve them from the project/config directory instead of Oxfmt’s own module location
- taught the JS formatter runtime to decode those internal plugin specs and resolve them with `createRequire(...)` before importing the plugin module
- added Rust tests covering:
  - relative path plugin resolution
  - bare package name encoding
  - scoped package encoding
  - package-subpath encoding
  - preserving encoded package specs in external formatter options
- added a new CLI canary fixture `apps/oxfmt/test/cli/plugin_languages_package`
  using a package-shaped local `node_modules/prettier-plugin-svelte`
- added a matching API canary proving `format("App.svelte", ..., { plugins: ["prettier-plugin-svelte"] })` works when the package is installed in the current project

Notes:
- this closes a real ecosystem gap: package-name plugins were previously imported relative to Oxfmt’s own code, which could miss the target project’s local `node_modules`
- this is still a local package-shaped canary, not the real upstream `prettier-plugin-svelte` package



## Phase 6C — Formatter override-scoped plugin discovery and config-dir package-subpath canary


Changes:
- extended formatter plugin-spec extraction so Oxfmt now discovers plugin specs declared inside `.oxfmtrc` `overrides[].options.plugins`, not just top-level `plugins`
- deduplicated extracted plugin specs while preserving first-seen order, so the formatter-side language resolver can initialize every plugin language once even when the same package appears in multiple overrides
- preserved raw override-only external formatter options during per-file resolution by storing each override’s original `options` object alongside the typed `FormatConfig`
- when a file matches formatter overrides, Oxfmt now merges the matching raw override `options` into the external Prettier option object before applying typed merged options, so plugin-owned settings such as `plugins` and `svelteSortOrder` survive override resolution
- added Rust tests covering:
  - collecting package-style plugin specs from override options
  - merging raw override plugin options into resolved external formatter options with the correct `resolveFrom` base directory
- added a new CLI canary fixture `apps/oxfmt/test/cli/plugin_languages_override_package_subpath`
  that proves all of the following at once:
  - `.svelte` discovery works when the plugin is declared only inside an override
  - package subpath specs like `prettier-plugin-svelte/subpath` are preserved as package specs rather than being mistaken for file paths
  - the plugin is resolved from the nested config directory via local `node_modules`
  - override-only plugin options survive into the formatter call and affect output

Notes:
- this closes a real compatibility gap for Prettier-style configs that scope framework plugins and plugin-owned options to `*.svelte` overrides instead of placing them at the config root
- the canary still uses a local package-shaped fake Svelte plugin; it does not yet depend on the real upstream `prettier-plugin-svelte` package



## Phase 6D — Direct JS API support for imported formatter plugin objects


Changes:
- added a JS-side formatter plugin registry (`apps/oxfmt/src-js/plugin_registry.ts`) that can replace direct Prettier plugin objects with internal marker strings before options cross the Rust NAPI boundary
- taught the JS formatter runtime (`apps/oxfmt/src-js/libs/apis.ts`) to recognize those registered plugin markers and rehydrate them back into real plugin objects when resolving plugin languages and when preparing `options.plugins` for Prettier
- taught the public JS API entry point (`apps/oxfmt/src-js/index.ts`) to normalize direct `format(..., { plugins: [pluginObject] })` calls through that registry
- preserved registered plugin markers on the Rust side instead of accidentally rewriting them as package specs during external-plugin extraction / option normalization
- added Rust tests covering:
  - preserving registered plugin markers during plugin-spec extraction
  - preserving registered plugin markers in resolved external formatter options
- added JS coverage for:
  - direct API formatting of `.svelte` with an imported plugin object (`apps/oxfmt/test/api/plugin_object.test.ts`)
  - registry normalization / lookup behavior (`apps/oxfmt/test/plugin_registry.test.ts`)

Notes:
- this slice currently targets the direct JS API path (`format()`), where the same JS process owns both the plugin registry and the formatter runtime
- it does **not** extend `oxfmt.config.ts` / CLI worker-process formatting to imported plugin objects yet, because child-process formatter workers cannot safely receive arbitrary plugin objects over IPC
- package/path string plugin specs continue to be the supported route for CLI/LSP/stdin formatting



## Phase 6E — Preserve Svelte formatter plugins when migrating Prettier configs


Changes:
- updated `apps/oxfmt/src-js/cli/migration/migrate-prettier.ts` so `--migrate prettier` no longer drops all non-internal plugin strings
- `prettier-plugin-tailwindcss` is still migrated into `sortTailwindcss`
- `prettier-plugin-packagejson` is still migrated into `sortPackageJson`
- all other string plugin specs are now preserved in the generated `.oxfmtrc.json`, including:
  - package names like `prettier-plugin-svelte`
  - relative plugin paths
  - package subpaths / scoped package specs
- duplicate preserved plugin specs are deduplicated while keeping first-seen order
- switched the top-level `Options` import in the migration file to `import type` so the module can be loaded for syntax-only validation without requiring Prettier at module-evaluation time
- added CLI migration coverage proving a Svelte-style Prettier config with:
  - `prettier-plugin-svelte`
  - `prettier-plugin-tailwindcss`
  - extra preserved plugin specs
  migrates to an `.oxfmtrc.json` that keeps the Svelte/plugin entries while still translating Tailwind and package-json behavior

Notes:
- this closes a real migration gap for Svelte projects: Oxfmt now supports package/path formatter plugins, so dropping `prettier-plugin-svelte` during `--migrate prettier` would produce a broken config
- Tailwind is still handled via Oxfmt's internal `sortTailwindcss` path, which matches the existing migration behavior


## Phase 6F — Migrate JSON-based Prettier overrides for Svelte/plugin configs


Changes:
- taught `oxfmt --migrate prettier` to read raw JSON-like Prettier configs (`.prettierrc`, `.prettierrc.json*`, `prettier.config.json*`, and `package.json#prettier`) in addition to the already-resolved top-level config
- added override migration for JSON-based Prettier configs, so `overrides[].files`, `excludeFiles`, and `options` now survive into `.oxfmtrc.json`
- normalized single-string `files` / `excludeFiles` patterns into the array form Oxfmt expects
- reused the existing plugin migration logic inside override `options`, so override-scoped plugin behavior now migrates correctly:
  - `prettier-plugin-svelte` and other supported string plugin specs are preserved
  - `prettier-plugin-tailwindcss` is migrated into `sortTailwindcss`
  - `prettier-plugin-packagejson` is migrated into `sortPackageJson`
- added CLI migration coverage for:
  - a `.prettierrc` with Svelte override plugins and Tailwind/packagejson migration inside overrides
  - a `package.json#prettier` config with Svelte overrides

Notes:
- this targets JSON-like Prettier config formats only; JS/YAML Prettier configs still do not have automatic override migration
- this closes an important Svelte migration gap because many real projects scope `prettier-plugin-svelte` and related plugin options to `*.svelte` overrides instead of the config root



## Phase 6G — Migrate JS-based Prettier overrides for Svelte/plugin configs


Changes:
- taught `oxfmt --migrate prettier` to load raw JS-based Prettier config files when preserving overrides, covering:
  - `.prettierrc.js`
  - `.prettierrc.cjs`
  - `.prettierrc.mjs`
  - `prettier.config.js`
  - `prettier.config.cjs`
  - `prettier.config.mjs`
- added raw JS-config loading via dynamic module import so override arrays survive instead of being flattened away by the already-resolved top-level config
- supported both CommonJS and ESM object exports for raw override migration
- kept the existing override migration behavior once the raw config object is loaded, including:
  - preserving `prettier-plugin-svelte` and other supported string plugin specs
  - translating `prettier-plugin-tailwindcss` into `sortTailwindcss`
  - translating `prettier-plugin-packagejson` into `sortPackageJson`
- added CLI migration coverage for:
  - a CommonJS `.prettierrc.cjs` with Svelte override plugins and Tailwind option migration
  - an ESM `prettier.config.mjs` with Svelte override plugins, package-subpath preservation, and package-json migration inside the override

Notes:
- this closes the JS-config half of the remaining override migration gap; YAML-based Prettier overrides are still not migrated automatically
- the new raw JS loader currently targets config files that export object-shaped configs (the common Prettier config pattern)


## Phase 5I — Whole-file custom-parser correctness sweep

Status: in progress; narrow runtime fixes are ready to PR in the current working tree

Changes:
- fixed `context.languageOptions.parser.parse()` raw-transfer source placement in `apps/oxlint/src-js/plugins/parser.ts`
  so embedded source text is written inside `ACTIVE_SIZE` instead of the full buffer size
- added `apps/oxlint/src-js/plugins/external_parser_utils.ts` and taught the whole-file custom-parser lane to:
  - preserve `Program.sourceType` when the parser returns it
  - otherwise fall back to the parser call's requested `sourceType`
  - infer `"unambiguous"` / missing source types from top-level import/export nodes instead of always forcing `"module"`
  - derive `isJsx` / `isTs` from explicit parser options when present, with AST fallback through external-only nodes when hints are missing
- tightened external-AST normalization so it no longer walks `comments` / `tokens` payloads while wiring parent links and ranges
- updated `apps/oxlint/src-js/plugins/report.ts` so node-based reports prefer parser-provided `node.loc` when available, safely fall back to `node.range` when `loc` is malformed, and reject reversed / out-of-bounds ranges before they reach Rust
- added JS coverage for the new helper logic and the node-loc diagnostic path:
  - `apps/oxlint/test/external_parser_utils.test.ts`
  - `apps/oxlint/test/whole_file_custom_parser_diagnostics.test.ts`

Expected impact:
- removes one likely source of allocator corruption for `languageOptions.parser.parse()` sub-parses
- makes whole-file Svelte/custom-parser runs report `sourceType`, JSX, and TS flags closer to the parser's real output instead of defaulting to misleading values
- reduces remaining diagnostic-span drift for external parser nodes that already provide precise `loc`
- catches invalid external-parser ranges on the JS side before they can fail later in Rust/NAPI conversion

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus file-level syntax validation
- whole-file traversal compatibility with real upstream `svelte-eslint-parser` visitor keys still needs end-to-end confirmation
- multiline/stylish rendering differences may still remain on the Rust diagnostic formatting side even after the JS-side span fixups above

## Phase 5J — Whole-file custom-parser `sourceType` alignment

Status: in progress; targeted runtime fix is ready to PR in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/parser_call_options.ts` so required custom-parser call options can inherit a top-level `languageOptions.sourceType` when `parserOptions.sourceType` is absent
- wired the whole-file runtime in `apps/oxlint/src-js/plugins/lint.ts` to pass resolved `languageOptions.sourceType` through to external/custom parsers instead of only consulting `parserOptions.sourceType`
- simplified `RuleTester` parser-call option construction in `apps/oxlint/src-js/package/rule_tester.ts` so the runtime and test harness now share the same source-type merge logic
- added focused coverage for both the helper and the runtime path:
  - `apps/oxlint/test/parser_call_options.test.ts`
  - `apps/oxlint/test/whole_file_custom_parser_source_type.test.ts`

Expected impact:
- fixes a real whole-file parser drift where configs using `languageOptions: { sourceType: "script", parser: ... }` could still invoke the external parser in module mode unless `parserOptions.sourceType` was redundantly set
- keeps `context.languageOptions.sourceType` aligned with the mode actually requested from the custom parser when the parser omits `Program.sourceType`
- reduces another behavioral difference between CLI/runtime whole-file linting and `RuleTester`

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus file-level syntax validation
- there may still be remaining diagnostics rendering drift on the Rust/stylish side after the JS-side span and source-type fixes
- real upstream `svelte-eslint-parser` end-to-end validation is still the main missing publish-readiness check for the whole-file traversal/runtime path

## Phase 5K — Whole-file external traversal actually uses parser visitor keys

Status: in progress; targeted runtime fix is ready to PR in the current working tree

Changes:
- wired `apps/oxlint/src-js/plugins/lint.ts` so whole-file custom-parser runs no longer try to walk external ASTs with the generated Oxc ESTree walker
- whole-file runs now compile visitors through `apps/oxlint/src-js/plugins/external_traversal.ts` and walk the returned AST with parser-aware visitor keys instead
- taught `external_traversal.ts` to fall back to inferred child keys for custom node types when the parser omits a specific node type from `visitorKeys`
- added `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts` covering three behaviors on a Svelte-style external AST:
  - direct listeners on `SvelteElement`
  - selector listeners on `SvelteElement > SvelteText`
  - `*` listeners on external-only nodes

Expected impact:
- fixes the main remaining whole-file Svelte compatibility gap where external-only nodes were still invisible to runtime traversal even though parser `visitorKeys` had already been captured
- brings real runtime behavior closer to the earlier RuleTester/custom-parser foundation and to ESLint’s parser-driven traversal model
- makes the whole-file lane more resilient when a parser provides visitor keys for the entry node but omits them for some nested custom node types

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus focused regression additions
- traversal still depends on the parser exposing custom subtree entry points on known ESTree container nodes such as `Program`; the new fallback only helps once traversal reaches a custom node type
- real upstream `svelte-eslint-parser` end-to-end validation remains the biggest publish-readiness check still missing for the whole-file path


## Phase 5L — Whole-file custom-parser `ecmaVersion` alignment

Status: in progress; targeted parser-call fix is ready to PR in the current working tree

Changes:
- extended `apps/oxlint/src-js/plugins/parser_call_options.ts` so required custom-parser call options now inherit top-level `languageOptions.ecmaVersion` when `parserOptions.ecmaVersion` is absent
- wired the whole-file runtime in `apps/oxlint/src-js/plugins/lint.ts` to pass resolved `languageOptions.ecmaVersion` through to external/custom parsers alongside the already-aligned `sourceType`
- mirrored the same behavior in `apps/oxlint/src-js/package/rule_tester.ts`, keeping runtime and `RuleTester` parser-call contracts aligned
- expanded parser-call coverage in `apps/oxlint/test/parser_call_options.test.ts` for `ecmaVersion` merge and precedence behavior
- added `apps/oxlint/test/whole_file_custom_parser_ecma_version.test.ts` proving a whole-file custom parser receives top-level `languageOptions.ecmaVersion`

Expected impact:
- fixes a real compatibility gap where whole-file custom parsers could miss `ecmaVersion` unless users redundantly repeated it inside `parserOptions`
- reduces another behavioral difference between Oxlint's whole-file Svelte/custom-parser lane and ESLint's parser-call contract
- keeps `RuleTester` closer to runtime behavior for framework-parser fixtures that gate syntax support on `ecmaVersion`

Remaining risks / gaps:
- this pass intentionally does not change `context.languageOptions.ecmaVersion` in standard runtime builds; it only aligns the parser-call contract
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus targeted regression additions
- real upstream `svelte-eslint-parser` end-to-end validation remains the main missing publish-readiness check for the whole-file path


## Phase 5M — Whole-file custom-parser BOM handling matches native parser runs

Status: in progress; targeted runtime fix is ready to PR in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/lint.ts` so whole-file custom-parser runs strip a leading Unicode BOM before invoking the external parser and before exposing `sourceCode.text`, while still preserving `sourceCode.hasBOM === true`
- added coverage in `apps/oxlint/test/whole_file_custom_parser_diagnostics.test.ts` proving the whole-file parser receives BOM-stripped code and rules still observe the preserved BOM flag through `SourceCode`

Expected impact:
- aligns whole-file custom-parser behavior with the existing native/raw-transfer lane, which already strips the BOM from `sourceCode.text` but keeps `hasBOM` separately
- fixes a real offset mismatch risk for parser-provided ranges, locations, fixes, and directive-comment spans on BOM-prefixed `.svelte` files, because Rust span conversion already expects JS offsets to be relative to BOM-stripped text
- reduces another source of snapshot drift and edge-case fix corruption for external-parser diagnostics on files starting with `﻿`

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus file-level syntax validation
- real upstream `svelte-eslint-parser` end-to-end validation is still the main missing publish-readiness check for the whole-file traversal/runtime path
- there may still be remaining diagnostics rendering drift on the Rust/stylish side that is unrelated to BOM handling

## Phase 5N — External parser normalization only walks AST children and honors `lang` precedence

Status: in progress; targeted runtime fixes are ready to PR in the current working tree

Changes:
- tightened whole-file external AST normalization in `apps/oxlint/src-js/plugins/lint.ts` so it now walks only parser-declared or inferred AST child keys instead of recursively descending through every enumerable object property
- added cycle protection to that normalization pass with a `WeakSet`, preventing accidental recursion into parser metadata objects that point back to the AST
- kept start/end backfilling from `range`, but now only for actual AST nodes reached through traversal keys
- fixed `apps/oxlint/src-js/plugins/external_parser_utils.ts` so `parserOptions.lang` wins over conflicting `parserOptions.ecmaFeatures.jsx` hints when deriving whole-file JSX/TS flags
- added focused regressions for both issues:
  - `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts`
  - `apps/oxlint/test/external_parser_utils.test.ts`

Expected impact:
- removes a real stack-overflow/corruption risk for whole-file custom parsers that attach cyclic or non-AST metadata objects to the returned `Program`
- keeps external-node parent/range normalization aligned with the same child graph the runtime traversal will actually walk
- fixes incorrect `SourceCode` / `context.languageOptions.parserOptions.ecmaFeatures.jsx` behavior when `lang: "jsx"` or `lang: "tsx"` was paired with contradictory JSX hints

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus syntax validation of the touched files
- real upstream `svelte-eslint-parser` end-to-end validation is still the main missing publish-readiness check for the whole-file traversal/runtime path
- any remaining diagnostics snapshot drift is now more likely to be in Rust-side rendering than in JS-side external parser normalization

## Phase 5O — Runtime `languageOptions.ecmaVersion` alignment

Status: in progress; targeted runtime fix is ready to PR in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/context.ts` so `context.languageOptions.ecmaVersion` is now backed by mutable per-file runtime state instead of being pinned to Oxlint's default latest version in normal builds
- added shared normalization via `normalizeEcmaVersionForLanguageOptions(...)`, matching the existing RuleTester/conformance behavior for numeric legacy values while still defaulting to Oxlint's latest version when no explicit value is configured
- wired `apps/oxlint/src-js/plugins/lint.ts` to set the active ECMAScript version before rule `create(...)` runs and to reset it after each file; the runtime now also falls back to `parserOptions.ecmaVersion` when no top-level `languageOptions.ecmaVersion` is present so `context.languageOptions` stays aligned with the parser-call contract
- reused the shared normalization helper in `apps/oxlint/src-js/package/rule_tester.ts` to keep the test harness and runtime on the same code path
- extended `apps/oxlint/test/whole_file_custom_parser_ecma_version.test.ts` with a regression proving a whole-file custom-parser rule sees the configured `context.languageOptions.ecmaVersion` during rule creation/runtime, not just inside parser call options

Expected impact:
- fixes a real runtime drift where whole-file custom-parser rules could receive `ecmaVersion` in the parser call but still observe `context.languageOptions.ecmaVersion === 2026` regardless of configuration
- improves compatibility with framework rules that branch on `context.languageOptions.ecmaVersion` in `create(...)` before returning visitors
- reduces another remaining difference between runtime linting and RuleTester behavior on the Svelte/custom-parser lane

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus targeted regression additions and parse-level TypeScript validation of the touched files
- parser call options still preserve user-provided numeric `ecmaVersion` values as-is; this phase aligns the runtime-facing `languageOptions` value, not the external parser option normalization policy
- real upstream `svelte-eslint-parser` end-to-end validation remains the main missing publish-readiness check for the whole-file traversal/runtime path


## Phase 5N — External traversal reaches custom children on known container nodes

Status: in progress; targeted runtime and fixture cleanups are ready to PR in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/source_code.ts` so `getVisitorKeysForNode(...)` now merges parser-configured visitor keys with inferred AST child keys from the actual node shape instead of treating configured keys as complete
- updated `apps/oxlint/src-js/plugins/external_traversal.ts` to always use the merged visitor-key lookup for external nodes, which lets traversal continue through custom child properties hanging off known ESTree container nodes like `Program`
- aligned external-AST normalization in `apps/oxlint/src-js/plugins/lint.ts` with the same merged-key behavior, so parent/range normalization and runtime traversal now see the same child graph
- added a focused regression in `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts` proving traversal reaches `Program.templateBody` even when parser `visitorKeys.Program` only lists `body`
- completed the remaining Svelte fixture package metadata cleanup by adding explicit `main`/`exports` entries in:
  - `apps/oxlint/test/fixtures/js_config_svelte_parser_baseline_flags_whole_file/node_modules/eslint-plugin-svelte/package.json`
  - `apps/oxlint/test/fixtures/js_config_svelte_parser_baseline_flags_whole_file/node_modules/svelte-eslint-parser/package.json`
  - `apps/oxlint/test/fixtures/js_config_svelte_parser_feature_flags_whole_file/node_modules/eslint-plugin-svelte/package.json`
  - `apps/oxlint/test/fixtures/js_config_svelte_parser_feature_flags_whole_file/node_modules/svelte-eslint-parser/package.json`

Expected impact:
- closes the remaining traversal gap noted after Phase 5K: external parsers no longer need to redundantly override visitor keys for every known ESTree container just to expose custom Svelte subtrees
- keeps external-node parent/range normalization aligned with the same merged traversal graph used at runtime and by selector fallback
- removes the last obvious Node ESM package-shape fixtures that could still emit `[DEP0151]`-style warning noise during Svelte snapshot runs

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus targeted regression additions
- real upstream `svelte-eslint-parser` end-to-end validation is still the main missing publish-readiness check for the whole-file traversal/runtime path
- any remaining diagnostics snapshot drift is now more likely to be in Rust-side rendering than in JS-side external traversal or fixture package metadata

## Phase 5P — Override-scoped `settings` survive per-file resolution

Status: in progress; targeted Rust config fix is ready in the current working tree

Changes:
- extended `crates/oxc_linter/src/config/config_store.rs` so `ResolvedOxlintOverride` now carries override-scoped `settings`
- fixed `Config::apply_overrides(...)` to merge matching override settings into the per-file resolved `LintConfig` using `OxlintSettings::override_settings(...)`, instead of silently dropping them
- wired `crates/oxc_linter/src/config/config_builder.rs` to preserve parsed override `settings` when building resolved overrides
- added a focused regression in `crates/oxc_linter/src/config/config_store.rs` proving:
  - matching `.svelte` overrides contribute arbitrary nested `settings.svelte` data
  - later matching overrides win for overlapping keys
  - non-matching files keep the base settings unchanged

Expected impact:
- fixes a real config/runtime gap where override `settings` were accepted syntactically but never reached per-file lint execution
- makes package-shaped Svelte configs with override-scoped plugin settings materially work instead of only parsing successfully
- restores the intended deep-merge behavior for arbitrary plugin settings, including non-well-known keys such as `settings.svelte`

Remaining risks / gaps:
- I still have not run the Rust test suite in this environment, so this phase is based on code inspection plus targeted regression additions
- `extends`/override interactions should now compose through the existing `OxlintSettings::override_settings(...)` logic, but end-to-end confirmation with real Svelte fixture configs is still pending
- the remaining publish-readiness uncertainty is now more concentrated in end-to-end fixture validation and any leftover diagnostics rendering drift

## Phase 5Q — `extends` merge parity and honest rule-count reporting

Status: in progress; targeted Rust config fixes are ready in the current working tree

Changes:
- fixed `crates/oxc_linter/src/config/oxlintrc.rs` so `Oxlintrc::merge(...)` now merges inherited `settings`, `env`, and `globals` instead of replacing parent values wholesale
- preserved child precedence while keeping parent-only entries, using the existing deep-merge / override helpers:
  - `OxlintSettings::override_settings(...)`
  - `OxlintEnv::override_envs(...)`
  - `OxlintGlobals::override_globals(...)`
- fixed `crates/oxc_linter/src/config/config_store.rs` so `ConfigStore::number_of_rules(...)` now returns `None` whenever overrides exist, not just when nested configs exist
- added focused regressions proving:
  - `extends` deep-merges plugin settings instead of dropping inherited keys
  - `extends` merges `env` and `globals` with child precedence
  - rule-count reporting becomes unknown when overrides can change the active rule set per file

Expected impact:
- fixes a real config bug where realistic shared Svelte configs could lose inherited `settings.svelte`, `env`, or `globals` once a child config added its own values
- restores more ESLint-like `extends` behavior for package-shaped and object-style config composition
- prevents misleading CLI/report metadata such as a single global rule count when overrides can materially change the enabled rules on matching `.svelte` files

Remaining risks / gaps:
- I validated the touched Rust files with `rustfmt +stable --check`, but full Rust test execution is still blocked in this container by offline dependency resolution against crates.io
- real upstream `svelte-eslint-parser` end-to-end validation is still the main missing publish-readiness check for the whole-file traversal/runtime path
- any remaining uncertainty is now more concentrated in fixture-level runtime behavior and residual diagnostics rendering drift rather than config merge plumbing

## Phase 5Q — External AST traversal now ignores comments, tokens, and comment attachments

Status: in progress; targeted runtime hardening is ready in the current working tree

Changes:
- added `apps/oxlint/src-js/plugins/external_ast_utils.ts` to centralize external-AST child-key inference and visitor-key sanitization
- updated `apps/oxlint/src-js/plugins/source_code.ts` to:
  - infer external child keys through the shared helper
  - sanitize parser-provided `visitorKeys` before exposing them through `SourceCode` / traversal
- updated `apps/oxlint/src-js/plugins/lint.ts` so whole-file external AST normalization now uses the shared child-key inference and sanitized visitor keys, preventing parser `comments`, `tokens`, `leadingComments`, `trailingComments`, and `innerComments` entries from being treated as visitable AST children
- updated `apps/oxlint/src-js/plugins/external_parser_utils.ts` so JSX/TS flag detection now:
  - walks only inferred AST child keys instead of every enumerable object property
  - uses a `WeakSet` to avoid recursion on cyclic parser metadata
- added focused regressions for both issues:
  - `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts`
  - `apps/oxlint/test/external_parser_utils.test.ts`

Expected impact:
- prevents whole-file custom-parser traversal from visiting parser comments, tokens, or attached comment arrays as if they were AST nodes
- removes another stack-overflow risk when external parsers attach cyclic metadata objects to returned programs
- keeps normalization, runtime traversal, and syntax-flag detection aligned on the same AST-only child graph

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus targeted regression additions and TypeScript parse checks
- real upstream `svelte-eslint-parser` end-to-end validation remains the main publish-readiness check still missing for the whole-file path
- any remaining diagnostics snapshot drift is now more likely to be outside external AST child-key handling


## Phase 5R — Empty whole-file custom-parser files still run JS plugin rules

Status: in progress; targeted Rust runtime fix is ready in the current working tree

Changes:
- removed the `source_text.is_empty()` early return in `crates/oxc_linter/src/lib.rs::run_external_only_on_source_text(...)`
- added a focused Rust regression proving the external-only whole-file path still calls the JS external linter for an empty `App.svelte` source string and surfaces its diagnostic

Expected impact:
- fixes a real runtime gap where empty `.svelte` or other framework-component files could skip JS plugin linting entirely when they relied on the external-only whole-file custom-parser path
- keeps empty-file behavior aligned with the normal whole-file parser lane, which already allows empty source text through to the parser/runtime
- avoids silently missing file-level Svelte rules that report on `Program`, parser services, or framework metadata even when the component body is empty

Remaining risks / gaps:
- I still have not run the full Rust or fixture suite in this environment, so this phase is based on code inspection plus a targeted unit regression
- real upstream `svelte-eslint-parser` end-to-end validation is still the main publish-readiness check missing for the whole-file traversal/runtime path
- any remaining snapshot/rendering drift is likely unrelated to this empty-file runtime guard now that the external-only path no longer skips empty sources


## Phase 5S — Whole-file external ASTs set `Program.parent = null`

Status: in progress; targeted runtime fix is ready in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/lint.ts` so whole-file external AST normalization now always writes the `parent` property, including `parent: null` on the root `Program`
- added a focused regression in `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts` proving `context.sourceCode.getAncestors(...)` works on a Svelte-style external node and yields `Program,SvelteElement` instead of crashing

Expected impact:
- fixes a real whole-file custom-parser runtime bug where external AST nodes could have a complete parent chain except for the root `Program`, causing `SourceCode.getAncestors(...)` to walk past the root and throw
- brings external AST normalization back in line with Oxlint's native ESTree shape, where `Program.parent` is explicitly `null`
- improves compatibility for Svelte/framework rules that use ancestor lookups on external-only template nodes

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus a focused regression and TypeScript syntax checks
- real upstream `svelte-eslint-parser` end-to-end validation remains the main publish-readiness check still missing for the whole-file traversal/runtime path
- any remaining diagnostics rendering drift is likely separate from this external parent-chain fix

## Phase 5T — Harden malformed JS-plugin diagnostics in whole-file runs

Status: in progress; targeted runtime hardening is ready in the current working tree

Changes:
- hardened `crates/oxc_linter/src/lib.rs` so external JS-plugin diagnostics no longer blindly trust `rule_index` or diagnostic spans returned from JS
- whole-file / external-only runs now:
  - report an internal plugin error when `rule_index` is out of bounds instead of panicking on `external_rules[rule_index]`
  - validate converted diagnostic spans against the original source text before building labeled diagnostics
- added focused Rust regressions proving malformed external diagnostics now surface ordinary error messages instead of crashing:
  - invalid `rule_index`
  - out-of-bounds diagnostic range
- tightened `apps/oxlint/src-js/plugins/report.ts` so explicit `context.report({ loc })` calls now reject reversed ranges where `loc.end` is before `loc.start`
- added a JS regression in `apps/oxlint/test/whole_file_custom_parser_diagnostics.test.ts` covering reversed explicit `loc` ranges

Expected impact:
- removes another real crash path in the whole-file custom-parser / JS-plugin bridge
- keeps malformed third-party plugin diagnostics from taking down the linter process even if JS-side validation is bypassed or regresses
- closes a remaining reporting correctness gap where explicit `loc` diagnostics could still produce reversed spans

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus targeted regression additions and syntax/format checks of the touched files
- real upstream `svelte-eslint-parser` end-to-end validation remains the main publish-readiness check still missing for the whole-file traversal/runtime path
- any remaining diagnostics rendering drift is now more likely to be in final Rust-side formatter rendering than in JS-to-Rust diagnostic validation

## Phase 5U — Preserve top-level parser metadata when `ast.comments` / `ast.tokens` are empty

Status: in progress; targeted whole-file parser compatibility fixes are ready in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/lint.ts` so whole-file `parseForESLint()` runs no longer discard parser-provided top-level `comments` / `tokens` metadata merely because the returned `ast` contains empty `comments: []` or `tokens: []` placeholders
- added a focused JS regression in `apps/oxlint/test/whole_file_custom_parser_metadata_fallback.test.ts` proving Oxlint now falls back to top-level parser metadata when `ast.comments` / `ast.tokens` are empty while the parser result still provides real entries
- cleaned the minor unused-variable warning in `crates/oxc_linter/src/lib.rs` by renaming the ignored BOM-stripped source-text binding in the external-only whole-file path
- added explicit `exports` entries to the remaining package-shaped Svelte fixture packages that still only relied on `main`, reducing `[DEP0151]` ESM warning noise in Node-based fixture runs

Expected impact:
- fixes a real whole-file custom-parser compatibility bug where `SourceCode` comment/token APIs and whole-file directive round-tripping could silently lose parser metadata if a framework parser returned top-level `comments` / `tokens` alongside empty AST placeholders
- improves compatibility with parsers that follow `parseForESLint()`'s top-level metadata shape instead of mutating the returned `Program` in place
- reduces fixture stderr noise so any remaining snapshot drift is more likely to reflect real diagnostic/rendering differences instead of Node package-resolution warnings

Remaining risks / gaps:
- I still have not run the full Vitest/Rust/fixture suite in this environment, so this phase is based on code inspection plus a targeted regression and syntax checks of the touched TypeScript files
- real upstream `svelte-eslint-parser` end-to-end validation remains the main publish-readiness check still missing for the whole-file traversal/runtime path
- diagnostics rendering/stylish snapshot drift may still need a separate Rust-side pass even after this metadata fallback fix


## Phase 5V — Preserve parser visitor-key traversal order on whole-file external ASTs


Changes:
- added `mergeExternalChildKeys(...)` so parser-provided `visitorKeys` stay authoritative for child traversal order
- still append inferred-only external child keys when parsers omit a custom subtree entry, preserving the earlier fallback behavior
- switched both whole-file external AST normalization and runtime traversal key lookup to the shared merge helper
- added a regression proving parser `visitorKeys.Program = ["templateBody", "body"]` visits Svelte template nodes before script nodes even when the object property order is `body` then `templateBody`

Notes:
- before this pass, traversal merged parser keys with inferred keys but preserved object insertion order whenever both sets overlapped, which could silently reorder whole-file Svelte traversal relative to ESLint/parser expectations
- this is a correctness fix, not just a perf cleanup; listener order can affect rule state and diagnostics when template and script subtrees interact


## Phase 5V — Respect explicit empty external `visitorKeys`

Status: in progress; targeted whole-file external-parser fix is ready in the current working tree

Changes:
- updated `apps/oxlint/src-js/plugins/external_ast_utils.ts` so explicit empty parser `visitorKeys` entries now stay authoritative instead of being expanded with inferred child keys
- updated `apps/oxlint/src-js/plugins/external_parser_utils.ts` so JSX/TS flag detection walks the same sanitized external child graph as normalization/traversal, including respect for explicit empty visitor keys
- updated `apps/oxlint/src-js/plugins/lint.ts` to derive whole-file external source flags after sanitizing parser `visitorKeys`, keeping parser metadata, traversal, and syntax-flag detection aligned on one graph
- added focused regressions in:
  - `apps/oxlint/test/whole_file_custom_parser_external_traversal.test.ts`
  - `apps/oxlint/test/external_parser_utils.test.ts`

Expected impact:
- fixes a real traversal correctness gap where a custom parser could explicitly mark a node type as a leaf with `visitorKeys: []`, but Oxlint would still recurse into inferred node-like children under that node
- prevents whole-file external runs from visiting hidden parser metadata subtrees or firing listeners on nodes the parser intentionally kept out of traversal
- keeps JSX/TS feature-flag inference aligned with the same external AST graph used for parent normalization and listener traversal

Remaining risks / gaps:
- I validated the touched TypeScript files with `typescript.transpileModule(...)` parse checks, but I still have not run the full Vitest/Rust/fixture suite in this environment
- real upstream `svelte-eslint-parser` end-to-end validation remains the main publish-readiness check still missing for the whole-file traversal/runtime path
- any remaining diagnostics rendering/stylish snapshot drift is likely separate from this visitor-key authority fix

## Phase 5W — Surface whole-file custom-parser parse errors as parser diagnostics


Changes:
- taught the JS whole-file custom-parser lane to recognize parser-style thrown errors that include an `index` or `lineNumber` / `column` and serialize them as parse errors instead of generic “Error running JS plugin” failures
- added a Rust-side `parse_error` payload path for external linter results so whole-file custom-parser parse failures become ordinary diagnostics with source labels rather than rule-indexed plugin diagnostics
- added focused JS and Rust regression tests covering:
  - parser errors reported via `index`
  - fallback to `lineNumber` / `column`
  - invalid whole-file parse-error range validation on the Rust side
- added a new package-shaped Svelte fixture `apps/oxlint/test/fixtures/js_config_svelte_template_parse_error_whole_file`
- the fixture uses a package-shaped `svelte-eslint-parser` stub that mimics upstream `ParseError` shape for a malformed template expression in `{#if page.data.user && }`
- expected CLI behavior is now locked down as a parser diagnostic at the template location instead of a generic JS-plugin runtime failure

Notes:
- this is still a package-shaped regression fixture, not a real upstream npm install of `svelte-eslint-parser`
- it closes the runtime/coverage gap for how Oxlint handles whole-file custom-parser parse failures, which was the practical blocker behind the malformed-template smoke failure


## Real-upstream parity plan — 2026-03-31

This is the next multi-phase publish-readiness plan now that the real parser package is available for verification work.

### Phase 1 — Direct real-parser integration coverage


Changes:
- added `apps/oxlint/test/real_svelte_parser_whole_file.test.ts`
- the new test file imports the real installed `svelte-eslint-parser` package instead of using a stub/package-shaped fixture parser
- added a happy-path whole-file regression proving a real `SvelteElement` listener fires through the external parser lane and that real parser services expose the expected Svelte surface
- added a malformed-template regression for `{#if page.data.user && }` proving the real parser path now reaches Oxlint's structured whole-file parse-error handling instead of falling back to a generic JS-plugin failure

Notes:
- this phase intentionally stays parser-only; it validates the real parser package before widening the matrix to real `eslint-plugin-svelte` package coverage
- the tests assume the workspace has the real `svelte-eslint-parser` package and its `svelte` peer available during JS test runs
- the existing package-shaped CLI fixture from Phase 5W stays in place for deterministic snapshot coverage until the real-package CLI canary is added

### Phase 2 — CLI canaries on the real parser


Changes:
- switched `apps/oxlint/test/fixtures/js_config_svelte_template_parse_error_whole_file` from the package-shaped `svelte-eslint-parser` stub to the real installed parser package
- removed the fixture-local `node_modules/svelte-eslint-parser` shim so the config now resolves the workspace-installed parser exactly like a real project would
- added `apps/oxlint/test/fixtures/js_config_svelte_real_parser_whole_file`
- the new fixture uses the real `svelte-eslint-parser` package with a local JS plugin rule that listens to `SvelteElement` and reports parser-service signals from the real whole-file path
- added CLI snapshots for both the malformed-template parse error and the happy-path traversal canary

Notes:
- this phase still keeps real `eslint-plugin-svelte` package parity out of scope; the new happy-path fixture uses a local JS plugin so the snapshot isolates parser behavior
- the malformed-template snapshot now intentionally locks down the user-facing parser failure emitted by the real parser path rather than the package-shaped stub error text
- these fixtures assume the workspace test install provides `svelte-eslint-parser` and its `svelte` peer

### Phase 3 — Real `eslint-plugin-svelte` direct-rule baseline


Changes:
- added `apps/oxlint/test/real_svelte_plugin_whole_file.test.ts`
- the new test imports the real installed `eslint-plugin-svelte` package together with the real `svelte-eslint-parser`
- registers the upstream `no-useless-mustaches` rule directly with Oxlint's JS-plugin runtime instead of using a local wrapper/stub rule
- proves a real upstream Svelte rule reports the expected whole-file template diagnostic and autofix on `.svelte` source text
- verifies plugin-name normalization from `eslint-plugin-svelte` to `svelte` still holds for the real package path

Notes:
- this phase intentionally validates the real rule runtime before config-level parity, because the current upstream package exports flat `configs.recommended` arrays whose base config wires both parser setup and `processor: "svelte/svelte"`
- Oxlint does not yet ingest that config/processor shape directly from imported config objects, so recommended-config loading moves into the next phase instead of being bundled into this one
- the tests assume the workspace has the real `eslint-plugin-svelte`, `svelte-eslint-parser`, and `svelte` packages available during JS test runs

### Phase 4 — Real plugin config-array / processor compatibility


Changes:
- taught `apps/oxlint/src-js/js_config.ts` to normalize ESLint-style flat config fragments when they appear inside `oxlint.config.ts` `extends`, including:
  - flattening nested config arrays such as `sveltePlugin.configs.recommended`
  - ignoring flat-config `name` metadata entries
  - translating flat `plugins: { svelte: pluginObject }` maps into `jsPlugins` entries using the plugin package `meta.name`
  - wrapping file-scoped flat fragments into Oxlint `overrides`
  - treating the upstream Svelte `processor: "svelte/svelte"` marker as redundant on Oxlint's whole-file parser lane instead of failing config parsing
- widened the public config typings so `defineConfig({ extends: [sveltePlugin.configs.recommended] })` is type-checkable without `any` casts
- added `apps/oxlint/test/js_config_flat_compat.test.ts` covering:
  - flat-config array normalization for a real-package-shaped Svelte config surface
  - explicit rejection of unsupported processors so non-Svelte cases still fail honestly
- added Rust-side parsing coverage for the normalized flat-config compatibility output in `apps/oxlint/src/js_config.rs`
- added a new CLI canary fixture `apps/oxlint/test/fixtures/js_config_svelte_real_recommended_whole_file`
  proving the real installed `eslint-plugin-svelte` recommended config can load end to end and report `svelte/no-useless-mustaches` on `.svelte`
- simplified `apps/oxlint/test/fixtures/js_config_svelte_template_parse_error_whole_file/oxlint.config.ts`
  so the malformed-template parser regression now relies on the real recommended config alone, without a fixture-local parser override

Notes:
- this phase intentionally treats the upstream Svelte processor declaration as a no-op compatibility marker on Oxlint's whole-file parser lane; it does **not** claim full generic ESLint processor support
- unsupported flat-config processors still fail during config loading so non-Svelte configs do not silently degrade
- the recommended-config canaries assume the workspace test install provides `eslint-plugin-svelte`, `svelte-eslint-parser`, and `svelte`

### Phase 5 — Type-aware and config-heavy real-project coverage

Status: in progress; Phase 5A is ready to PR in the current working tree

Delivered in Phase 5A:
- converted `apps/oxlint/test/fixtures/js_config_svelte_type_aware_whole_file` from a stub parser canary to the real parser stack:
  - real `svelte-eslint-parser` as the whole-file parser
  - real `@typescript-eslint/parser` as the nested script parser
  - imported `svelteConfig` with a real function-valued `preprocess`
  - object-style `extends` merge carrying the nested parser and `svelteConfig` from the base config while the matching override adds `projectService`, `extraFileExtensions`, and `tsconfigRootDir`
- added a fixture-local `tsconfig.json` plus a real typed helper module so the canary uses an actual project-shaped layout instead of synthetic parser stubs
- widened the fixture to cover both:
  - a mixed module-script + instance-script `.svelte` component with `lang="ts"`
  - a real no-script `.svelte` component under the same config
- updated the local canary rule so it now proves, on the real parser path, that Oxlint preserves and exposes:
  - `context.languageOptions.parser`
  - nested `parserOptions.parser`
  - `projectService`
  - `extraFileExtensions`
  - imported `svelteConfig.preprocess`
  - merged base parser options from `extends`
  - real parser services such as `isSvelte`, `svelteParseContext`, and TS parser service maps
  - traversal over real Svelte root/style/template/script nodes and nested TS nodes
- extended `apps/oxlint/test/real_svelte_parser_whole_file.test.ts` with a direct runtime regression for the real nested TypeScript parser path, covering mixed script layouts and preserved TS parser services on whole-file `.svelte` input

Remaining planned scope:
- add a dedicated real-package publish-readiness sweep for comments/tokens, disable directives, scope helpers, fixes/suggestions, and visitor-key edge cases where package-shaped fixtures still remain
- confirm the later whole-file compatibility fixes still hold on real parser ASTs, metadata, and scope data across a broader fixture matrix

### Phase 6 — Whole-file runtime publish-readiness

Status: in progress; Phase 6A is ready to PR in the current working tree

Planned scope:
- re-run the comments/tokens, disable-directive, scope, fixes/suggestions, and visitor-key cases against real packages where practical
- stabilize the JS/Vitest and CLI fixture suite, not only the Rust workspace suite
- document any remaining deliberate gaps, including CFG/code-path listeners, if they still differ on the whole-file lane

### Phase 6A — Real-parser comments/directives/scope publish-readiness


Changes:
- extended `apps/oxlint/test/real_svelte_parser_whole_file.test.ts` with direct real-parser regressions covering:
  - `SourceCode` comment/token APIs on real Svelte HTML comments and markup tokens
  - round-tripping real HTML comment spans through `lintFile(...)` for Rust-side disable-directive handling
  - parser-provided scope helpers (`getScope`, `getDeclaredVariables`, `isGlobalReference`, and `markVariableAsUsed`) on real `.svelte` script/template input
- added `apps/oxlint/test/fixtures/js_config_svelte_real_comments_tokens_whole_file`
- added `apps/oxlint/test/fixtures/js_config_svelte_real_disable_directives_whole_file`
- added `apps/oxlint/test/fixtures/js_config_svelte_real_unused_disable_directives_whole_file`
- all three new fixtures use the real installed `svelte-eslint-parser` package with local JS plugin rules, instead of inline/package-shaped parser stubs
- the new CLI canaries lock down:
  - `SourceCode` comment/token behavior on a real HTML comment + markup `.svelte` file
  - `eslint-disable-next-line` suppression from a real whole-file parser run
  - `--report-unused-disable-directives` reporting on real HTML comment spans

Notes:
- the earlier stub-parser fixtures remain useful as narrower generic whole-file-lane canaries, so this phase adds real-parser coverage without deleting the deterministic stub cases yet
- the remaining Phase 6 scope still includes real-package fixes/suggestions coverage and any visitor-key/package-shaped edge cases that only show up outside comments/directives/scope APIs

### Phase 6B — Real-parser fixes/suggestions and traversal/call-contract publish-readiness


Changes:
- extended `apps/oxlint/test/real_svelte_parser_whole_file.test.ts` with direct real-parser regressions covering:
  - required parser-call flags and top-level `sourceType` / `ecmaVersion` passthrough on the real `svelte-eslint-parser` path
  - selector and wildcard traversal on real `SvelteElement` / `SvelteText` nodes
  - visitor-key order across real script and template nodes in `Program.body`
- added `apps/oxlint/test/fixtures/js_config_svelte_real_fixes_suggestions_whole_file`
- the new CLI canary uses the real installed `svelte-eslint-parser` package together with a local JS plugin rule that emits both:
  - a safe whole-file fix on a real `SvelteLiteral` node
  - whole-file suggestions on a real `SvelteText` node
- the fixture snapshots all three whole-file edit modes on real `.svelte` input:
  - normal lint output
  - `--fix`
  - `--fix-suggestions`

Notes:
- this phase keeps the earlier package-shaped fixes/suggestions and parser-flag fixtures as narrow generic canaries, while adding publish-readiness coverage on top of the real parser package
- the remaining Phase 6 scope is mostly suite stabilization and any final real-package gaps that only show up once the broader JS/CLI matrix is run end to end

### Phase 7 — Formatter parity

Status: in progress; Phase 7D is ready to PR in the current working tree

Planned scope:
- swap in the real `prettier-plugin-svelte` for the formatter-side canaries
- validate CLI/API/override-scoped plugin resolution on real `.svelte` formatting setups
- keep the earlier fake plugin fixtures as narrow plumbing canaries until the real-package matrix is stable

### Phase 7A — Real `prettier-plugin-svelte` package/object/override canaries


Changes:
- added `apps/oxfmt/test/cli/plugin_languages_real_package`
- the new fixture vendors a CommonJS build of the real upstream `prettier-plugin-svelte` source under fixture-local `node_modules`, instead of using the earlier fake formatter plugin
- the top-level fixture config proves CLI package-name plugin resolution formats `.svelte` files through the real plugin and preserves the real plugin-owned `svelteSortOrder` option
- the nested `subdir/config/.oxfmtrc.json` fixture proves override-scoped plugin discovery still works when the real plugin is declared only inside `overrides[].options.plugins`
- added `apps/oxfmt/test/api/plugin_languages_real_package.test.ts` covering both:
  - package-name plugin resolution from `cwd` with the real plugin
  - direct API formatting with an imported real plugin object
- all new formatter parity checks lock down real `.svelte` formatting output rather than the simplified reorder-only fake plugin behavior

Notes:
- this phase keeps the earlier fake/plugin-shaped formatter fixtures in place as narrower plumbing canaries, mirroring the linter-side parity strategy
- the vendored real plugin package still assumes the workspace/runtime has the `svelte` peer available, because upstream `prettier-plugin-svelte` loads `svelte/compiler` at runtime
- a follow-up formatter parity slice can widen this to any remaining CLI/LSP/migration cases that still only exercise the fake plugin path

### Phase 7B — Real `prettier-plugin-svelte` LSP parity and in-memory Svelte support


Changes:
- added `apps/oxfmt/test/lsp/plugin_languages_real_package.test.ts`
- reused the Phase 7A real-package fixture to prove the LSP path formats `.svelte` files through the real vendored `prettier-plugin-svelte` package
- added LSP coverage for four real-package scenarios that were previously unverified:
  - normal file-based `.svelte` formatting from local `node_modules`
  - override-scoped plugin discovery from a nested `.oxfmtrc.json`
  - untitled/in-memory `.svelte` formatting through `languageId: "svelte"`
  - formatter restart after `workspace/didChangeConfiguration`, switching from an empty config to the real Svelte plugin config
- taught the LSP in-memory file mapper in `apps/oxfmt/src/lsp/mod.rs` to recognize `languageId: "svelte"` and synthesize a `.svelte` filename, which allows untitled/in-memory Svelte documents to hit external plugin language resolution
- added a small Rust unit test for the new Svelte language-id mapping and a fixture-local `empty.json` used by the config-change canary

Notes:
- before this phase, file-backed `.svelte` LSP formatting could work when a real plugin was configured, but untitled/in-memory Svelte documents were still blocked because the LSP layer had no `svelte` → `.svelte` mapping
- this phase reuses the existing real-package fixture rather than vendoring the plugin a second time
- the main remaining formatter parity work is broader publish-readiness/stabilization, not another obvious missing Svelte-specific runtime hook

### Phase 7C — Migrate JS-based Prettier configs that import real Svelte plugin objects


Changes:
- taught `apps/oxfmt/src-js/cli/migration/migrate-prettier.ts` to recognize real Svelte formatter plugin objects when they appear in JS/CJS/MJS Prettier configs
- recognized Svelte plugin objects are now migrated back to the package spec string `prettier-plugin-svelte`, because `.oxfmtrc.json` cannot store non-serializable plugin objects directly
- added `apps/oxfmt/test/cli/migrate_prettier/migrate_prettier.test.ts` regressions covering both:
  - top-level `plugins: [sveltePlugin]` in `prettier.config.mjs`
  - override-scoped `plugins: [sveltePlugin]` in `.prettierrc.cjs`
- the new migration tests reuse the Phase 7A vendored real package and install only the minimal peer stubs needed for config import resolution, so the migration path now exercises the real plugin object shape instead of a fake formatter plugin

Notes:
- this closes the main remaining migration/config-conversion gap for real Svelte formatter setups using imported plugin objects instead of package-spec strings
- the migrated spec is normalized to `prettier-plugin-svelte`; the exact original module spec cannot be recovered from a plugin object alone
- the remaining formatter parity scope is now mostly suite stabilization and any broader publish-readiness coverage that still only exercises fake/plugin-shaped fixtures

### Phase 7D — Real `prettier-plugin-svelte` stdin parity


Changes:
- extended `apps/oxfmt/test/cli/plugin_languages_real_package/plugin_languages_real_package.test.ts` with real-package stdin coverage
- added stdin assertions for both:
  - top-level `.oxfmtrc.json` plugin discovery from fixture-local `node_modules`
  - override-scoped `plugins: ["prettier-plugin-svelte"]` loaded via `-c ./subdir/config/.oxfmtrc.json`
- widened `apps/oxfmt/test/cli/utils.ts` so stdin helpers can run from a fixture `cwd`, pass extra CLI args such as `-c`, and still support the existing pipe-based smoke case

Notes:
- before this phase, the formatter parity matrix covered the real Svelte plugin through file-backed CLI, API, LSP, and config migration, but not through the dedicated `--stdin-filepath` runtime
- this phase reuses the existing vendored real-package fixture rather than introducing another copy of the plugin package
- the main remaining formatter parity work is broader stabilization/documentation, not an obvious missing Svelte-specific runtime entry point

## Current status update — 2026-03-31

Status: updated after the latest VPS verification pass.

Verified in the current working tree:
- `mise -C /srv/apps/oxc x -- cargo test -p oxc_linter --lib` passes
- `mise -C /srv/apps/oxc x -- cargo test -p oxlint --lib` passes
- `mise -C /srv/apps/oxc x -- cargo test -p website_linter --bin website_linter` passes
- `mise -C /srv/apps/oxc x -- cargo test --all-features` passes

Notes:
- this iteration did **not** land a new core Svelte runtime/parser slice; it primarily cleared config/fixture blockers, restored package-shaped test fixtures, and refreshed snapshots/schema output so the current tree is green again
- the Svelte whole-file/custom-parser stack already in this file is still the relevant roadmap; the main remaining work is publish-readiness and real-upstream validation, not another large missing plumbing layer

## Stabilization follow-up — 2026-04-01


Changes:
- updated the Rust flat-config compatibility regression in `apps/oxlint/src/js_config.rs` so it asserts Oxlint's canonical serialized severity (`"deny"`) after parsing instead of expecting the JS-side alias (`"error"`)
- documented the layer split in `apps/oxlint/test/js_config_flat_compat.test.ts`: the JS loader still emits ESLint-style aliases such as `"error"`, while Rust canonicalizes them through `AllowWarnDeny` during deserialization

Notes:
- the reported cargo failure was narrowly isolated to this canonicalization mismatch; the intent of the compatibility layer is unchanged
- no runtime behavior changed here, only the regression expectation and accompanying test note

## Stabilization pass — 2026-04-01 (build/raw-source/formatter fixtures)


Changes:
- fixed the `apps/oxlint/src-js/js_config.ts` build-only duplicate import trap by using the existing `JSONStringify` helper directly for flat-config processor errors; this avoids `replace-globals` prepending a second `JSONStringify` import during `tsdown`
- switched raw-transfer deserialization imports in both `tasks/ast_tools/src/generators/raw_transfer.rs` and the checked-in `apps/oxlint/src-js/generated/deserialize.js` file from `../plugins/*.js` to `../plugins/*.ts`, so raw source execution can resolve the source tree under Node's strip-types path instead of looking for non-existent source-side `.js` files
- added `code`-condition package `imports` entries in `apps/oxlint/package.json` for `#oxlint`, `#oxlint/plugins`, and `#oxlint/plugins-dev`, keeping raw-source self-imports pointed at `src-js/*` while built usage still resolves to `dist/*`
- patched the vendored `prettier-plugin-svelte` fixture used by `apps/oxfmt/test/cli/plugin_languages_real_package` so top-level `<style>` blocks preserve their snipped-content attribute in the real-package fixture path; this is intended to stabilize the remaining CLI/LSP formatter parity tests without changing the healthy realworld formatter path
- added `apps/oxlint/test/raw_source_runtime.test.ts` to lock down the new raw-source self-import and deserialization import expectations

Notes:
- this is a stabilization slice driven by the 2026-04-01 host verification report: the remaining `oxlint` runtime blockers were the JS build failure and raw-source plugin loading, while the main remaining `oxfmt` red was fixture/LSP style-body loss despite the realworld app formatting cleanly
- the formatter change here is intentionally scoped to the vendored real-package fixture used by the parity tests; the report already showed that the real Svelte app formatter path itself was healthy

## What is still left for full Svelte support?

### Lint/runtime side
- replace the remaining stub/package-shaped Svelte canaries with end-to-end coverage against the real upstream `svelte-eslint-parser` and `eslint-plugin-svelte` packages
- run and stabilize the remaining JS/Vitest whole-file parser coverage, not just the Rust workspace suite
- confirm real parser behavior for the later whole-file fixes in Phase 5I–5V, especially:
  - traversal/visitor-key authority and ordering
  - parser metadata fallback (`comments`, `tokens`, `scopeManager`, `parserServices`)
  - parser call contract alignment (`sourceType`, `ecmaVersion`, BOM handling, feature flags)
  - diagnostic/range validation on real Svelte AST output
- close any remaining runtime gaps that only surface with real Svelte parser output rather than synthetic fixture ASTs
- validate whether CFG/code-path listener compatibility is still needed on the whole-file custom-parser lane for any real Svelte ecosystem rules

### Config / migration / ecosystem side
- verify that real Svelte project configs work end to end with imported parser/plugin packages, nested parser options, `svelteConfig`, and override-scoped settings in ordinary project layouts
- keep generated schema/docs in sync as the Svelte/config story settles

### Formatter side
- complete the formatter-side equivalent of the linter publish-readiness pass using the real upstream `prettier-plugin-svelte`, not only local fake/package-shaped canaries
- finish any remaining CLI/LSP/config migration validation for real Svelte formatter plugin setups

### Practical definition of “full support” still missing
To call Svelte support “full”, the repo still needs confidence that a normal Svelte project using the real ecosystem packages works in all three modes:
- linting regular `.svelte` files
- linting type-aware `.svelte` files
- formatting `.svelte` files
without depending on stub parsers/plugins or package-shaped fake fixtures.


## Stabilization pass — 2026-04-02 (build hardening / flat ignores / formatter fallback)


Changes:
- removed the last direct `utils/globals.ts` import from `apps/oxlint/src-js/js_config.ts` and switched it back to native `Date.now` / `JSON.stringify`, so the `replace-globals` tsdown plugin can rewrite those accesses without ever colliding with a hand-written `JSONStringify` binding
- hardened `apps/oxlint/tsdown_plugins/replace_globals.ts` to detect existing imports from `utils/globals.ts` and skip prepending duplicate bindings; added `apps/oxlint/test/replace_globals.test.ts` to lock down the narrowed `JSONStringify` redeclaration case
- added root-fragment flat-config `ignores` compatibility in `apps/oxlint/src-js/js_config.ts`, mapping it to Oxlint `ignorePatterns` while still rejecting override-like fragments that mix `files` and `ignores`; covered in `apps/oxlint/test/js_config_flat_compat.test.ts`, `apps/oxlint/test/config.test-d.ts`, and `apps/oxlint/src/js_config.rs`
- added raw-source `apps/oxlint/src-js/plugins/tokens.js` / `comments.js` shim modules plus a regression in `apps/oxlint/test/raw_source_runtime.test.ts`, so generated `.js` plugin imports keep working under raw source execution even if a stale/generated path still points at the source tree
- hardened the vendored real `prettier-plugin-svelte` fixture in `apps/oxfmt/test/cli/plugin_languages_real_package/fixtures/node_modules/prettier-plugin-svelte/src/embed.js` with a fallback that extracts unsnipped tag bodies from `options.originalText` when the normalized Svelte AST drops the synthetic content attribute, which specifically targets the remaining `<style></style>` parity failure in CLI/LSP tests

Notes:
- this pass was driven by the latest host verification report: the main remaining `oxlint` blocker was the unresolved `JSONStringify` build failure, while the formatter side was narrowed to style-body loss in the vendored real-package parity fixture rather than in real Svelte apps
- the raw-source shims intentionally overlap with the earlier `.ts` import fix in the generated deserializer; the goal is to make raw-source execution resilient regardless of whether the checked-in generator output or a stale generated file uses `.ts` or `.js` plugin specifiers


## Stabilization pass — 2026-04-02 (raw-source runtime flags)


Changes:
- added `apps/oxlint/src-js/runtime_flags.ts` to seed `DEBUG` / `CONFORMANCE` globals from environment variables when source `.ts` entrypoints are executed directly without `tsdown` define replacement
- imported that runtime-flag bootstrap from the raw-source `apps/oxlint/src-js/cli.ts` and `apps/oxlint/src-js/plugins-dev.ts` entrypoints, covering the two main source-mode workflows that were still using build-only globals
- updated `apps/oxlint/package.json` so the raw-source `pnpm conformance` lane explicitly runs with `CONFORMANCE=true`, matching the existing conformance-build semantics when `#oxlint/plugins-dev` resolves to source files through the `code` condition
- added `apps/oxlint/test/runtime_flags.test.ts` plus extra assertions in `apps/oxlint/test/raw_source_runtime.test.ts` to lock down the new runtime-flag parsing/bootstrap expectations and the source-conformance script wiring

Notes:
- this pass specifically targets the remaining raw-source blocker from the latest host report: direct source execution still crashed on `ReferenceError: DEBUG is not defined` before real Svelte plugin loading could complete
- the built JS bundles keep their existing compile-time flag replacement; the new runtime bootstrap is only there to make source-mode execution behave predictably when the build step is intentionally bypassed


## Stabilization pass — 2026-04-02 (optional real-package test gating / RuleTester parser propagation)


Changes:
- fixed `RuleTester` custom-parser propagation by registering each test case's merged `languageOptions` and passing the resulting IDs into `lintFileImpl(...)`, so `context.languageOptions.parser` is no longer reset back to Oxlint's default parser during the actual lint run
- added optional-package test gating helpers in `apps/oxlint/test/utils.ts` and regression coverage in `apps/oxlint/test/utils.test.ts`
- extended fixture `options.json` support with `requiredPackages`, then marked the real-package Svelte CLI fixtures as requiring the real workspace installs they actually depend on
- updated `apps/oxlint/test/e2e.test.ts` and `apps/oxlint/test/eslint_compat.test.ts` to skip optional-package fixtures cleanly when those packages are not resolvable from the fixture cwd
- updated `apps/oxlint/test/real_svelte_parser_whole_file.test.ts` and `apps/oxlint/test/real_svelte_plugin_whole_file.test.ts` to skip their real-package coverage cleanly when `svelte`, `svelte-eslint-parser`, `eslint-plugin-svelte`, or `@typescript-eslint/parser` are not installed

Notes:
- this does not remove the real-package coverage; it keeps those tests active whenever the optional Svelte packages are present, but stops turning package-light workspace installs red for what are effectively environment/setup gaps
- the `RuleTester` change is a real runtime fix: custom parser objects now stay visible through `context.languageOptions` during the lint pass itself, instead of only during pre-lint metadata setup



## Phase 6H — Self-contained real `prettier-plugin-svelte` test fixture


Changes:
- added a local `svelte` fixture package under `apps/oxfmt/test/cli/plugin_languages_real_package/fixtures/node_modules/svelte`
- the fixture exports a narrow CommonJS `svelte/compiler` implementation so the real-package Oxfmt tests no longer depend on a workspace-wide or globally installed Svelte compiler
- the compiler shim intentionally supports only the document shapes exercised by the real-package formatter fixtures:
  - top-level `<script>`
  - top-level `<style>`
  - simple markup with text plus `{identifier}` mustache tags
- kept the existing real upstream `prettier-plugin-svelte` package fixture unchanged; the new shim only supplies the missing compiler dependency it expects at runtime
- added a narrow API-side regression in `apps/oxfmt/test/api/plugin_languages_real_package.test.ts` to assert the real plugin fixture resolves `svelte/compiler` from its own fixture-local `node_modules`

Notes:
- this is test-fixture hardening, not production formatter logic
- the goal is to make the real-package CLI/API/LSP formatter matrix self-contained and deterministic in CI
- the shim is intentionally narrow and should only be used by the fixture inputs in this test directory


## Stabilization pass — 2026-04-02 (raw-source runtime bindings / whole-file RuleTester / real-package self-containment follow-up)


Changes:
- upgraded `apps/oxlint/src-js/runtime_flags.ts` so raw-source entrypoints do not just seed `globalThis.DEBUG` / `globalThis.CONFORMANCE`, but also install bare runtime flag bindings via global eval; this matches the way bundled replacements expose those identifiers to flag-guarded modules such as `src-js/utils/asserts.ts`
- added a runtime-flags regression in `apps/oxlint/test/runtime_flags.test.ts` that verifies bare `DEBUG` / `CONFORMANCE` identifiers are visible inside a later raw-source execution context without polluting the current test worker's globals
- fixed `RuleTester` whole-file custom-parser execution so test cases that pass `languageOptions.parser` now lint through the same whole-file parser lane as real runtime usage, instead of parsing through the raw-transfer lane and only copying metadata afterwards
- updated the real Svelte whole-file parser/plugin tests to gate on actual dynamic importability of `svelte`, `svelte-eslint-parser`, `eslint-plugin-svelte`, and `@typescript-eslint/parser`, which more closely matches the failure mode observed in the narrowed Vitest red lanes
- strengthened the vendored `svelte/compiler` fixture used by the real `prettier-plugin-svelte` tests so it returns a root node with stable source spans, and added a narrow canary test that validates the fixture AST shape after `snipScriptAndStyleTagContent(...)`

Notes:
- the runtime-flags change directly targets the remaining raw-source `ReferenceError: DEBUG is not defined` failure shape reported during real-project verification
- the `RuleTester` change is intentionally aligned with Oxlint's runtime whole-file parser path so parser identity, parser services, visitor keys, and scope manager now come from a single parser result instead of a hybrid of two different lanes
- the formatter-side change is still fixture hardening rather than production formatter logic; the goal is to make the real `prettier-plugin-svelte` parity matrix behave like the already-green built real-project path


## Stabilization pass — 2026-04-02 (raw-source JSON import / optional-package importability / formatter stdin + migration parity)


Changes:
- fixed the raw-source `oxlint` plugin context bootstrap so source execution now reads `apps/oxlint/package.json` via a default JSON import instead of a named `version` export; this matches Node's JSON module semantics and removes the raw-source `does not provide an export named 'version'` failure shape from the latest verification bundle
- changed `apps/oxlint/test/utils.ts` optional-package gating from `require.resolve(...)` to actual `import(...)` smoke checks executed from the fixture cwd, then updated the helper tests to reflect importability rather than bare resolution
- added `requiredPackages` to the Svelte whole-file type-aware fixture so it skips cleanly when `svelte`, `svelte-eslint-parser`, or `@typescript-eslint/parser` are absent from the runtime environment
- made the `js_config_extends_package_string` fixture self-contained by vendoring a tiny `oxlint-config-base` package that enables `eqeqeq`, removing the last dependency on a host-installed package for that lane
- fixed root `Oxlintrc` deserialization so public unknown-field errors no longer expose the internal `_languageOptionsId` / `_languageOptionsHasParser` transport fields, while still preserving those fields for JS-config loader internals
- hardened the Svelte template-parse-error fixture by embedding a parser object directly in the fixture-local `eslint-plugin-svelte` recommended config, restoring the intended parse-error coverage without depending on an external parser package
- fixed Oxfmt stdin test helpers to preserve final trailing newlines instead of letting `execa` strip them, and updated the affected stdin snapshots accordingly
- fixed `--migrate prettier` top-level plugin preservation so supported string plugin specs now come from the raw config when available, avoiding accidental conversion of relative plugin paths into absolute temp-directory paths during migration
- updated the affected `oxlint` e2e snapshots whose remaining diffs were due to the now-intentional omission of global rule counts when configs contain overrides / nested configs, plus the narrowed whole-file diagnostic span formatting changes already reflected in the latest verification logs

Notes:
- this pass is driven directly by the uploaded VPS verification bundle: it targets the remaining raw-source `oxlint` crash, the package-availability false positives in `oxlint` e2e, and the three remaining `oxfmt` Vitest reds
- the snapshot updates are limited to fixtures whose logs now differ only because rule counts are intentionally omitted for override-dependent configs or because the whole-file diagnostic formatter now emits the smaller spans already shown in the verification logs
- one red lane from the VPS bundle came from an untracked local `apps/oxlint/test/fixtures/runtime_flags/` fixture that is not present in the repository snapshot used for this patch, so it is not modified here


## Stabilization pass — 2026-04-02 (raw-source package version lookup / formatter stdin newline / migrate-prettier local plugin spec normalization)


Changes:
- replaced `apps/oxlint/src-js/plugins/context.ts` package-version bootstrap with a filesystem-based `resolveNearestPackageVersion(import.meta.url)` helper so raw-source and built plugin contexts no longer depend on ESM JSON-module export semantics when reading `apps/oxlint/package.json`
- added `apps/oxlint/src-js/utils/package_version.ts`, `apps/oxlint/test/package_version.test.ts`, and a raw-source regression in `apps/oxlint/test/raw_source_runtime.test.ts` so both `src-js/plugins/*` and `dist-pkg-plugins/*` layouts are covered explicitly
- added `apps/oxfmt/src-js/cli/migration/plugin_specs.ts` and wired `migrate-prettier` through it so project-local absolute plugin file paths are re-relativized back to `./...` while `node_modules` paths and out-of-project paths are left untouched
- added `apps/oxfmt/test/plugin_specs.test.ts` to pin that plugin-spec normalization behavior directly
- hardened `apps/oxfmt/src/core/format.rs` so successful formatter output now always ends with a final newline when `insert_final_newline` is enabled, instead of assuming every upstream formatter already guarantees it
- added focused Rust unit coverage for the newline helper so stdin/external-formatter parity regressions are caught closer to the formatting boundary

Notes:
- this pass directly targets the remaining items from the latest verification pass report: the raw-source `oxlint` package-version load failure, the `migrate-prettier` temp absolute-path leak, and the real-package stdin newline mismatch in `oxfmt`
- the raw-source fix intentionally avoids JSON import semantics entirely; it should behave the same whether the module is executed from `src-js`, from `dist-pkg-plugins`, or from a direct `node --experimental-strip-types` workflow
- the formatter newline change is deliberately narrow: it only enforces the configured final-newline contract at the end of formatting, and does not otherwise change formatter output selection or parser routing


## Stabilization pass — 2026-04-02 (real-package oxlint fixture gating + external formatter EOF newline parity)


Changes:
- added `requiredPackages` declarations to the remaining real-package Svelte `apps/oxlint/test/fixtures/*` e2e fixtures that import `svelte-eslint-parser` or `eslint-plugin-svelte` directly but were still missing optional-package gating, so package-light installs now skip those lanes cleanly instead of snapshotting fixture-local config import failures
- preserved external-formatter output when the *only* change would be appending a trailing newline to an otherwise already-formatted file, avoiding noisy `--check` / `--list-different` failures on real Svelte projects while still keeping substantive formatting edits intact
- added focused Rust coverage for the external-formatter EOF-newline preservation helper
- added real `prettier-plugin-svelte` API and CLI regressions using a self-contained fixture file that is already formatted except for a missing final newline

Notes:
- the Oxlint side here is test-environment stabilization rather than a core runtime fix; the goal is to stop optional real-package fixture lanes from turning red when those packages are not installed from the fixture cwd
- the Oxfmt change is intentionally narrow: it only preserves the source EOF newline state for external formatter outputs when no other text changes are required


## Stabilization pass — 2026-04-03 (real-package config-import gating + empty-file EOF newline parity)


Changes:
- hardened `apps/oxlint/test/utils.ts` optional-package gating so real-package fixtures now try the fixture’s own config import path (for `oxlint.config.ts` / `vite.config.ts`) and only skip when that import fails due to one of the declared `requiredPackages`; this closes the gap where a bare package probe could pass while the real config import still crashed in `test/e2e.test.ts`
- added focused coverage in `apps/oxlint/test/utils.test.ts` for missing-package error parsing and for the new “config-local import failure becomes a clean skip” behavior
- broadened `apps/oxfmt/src/core/format.rs` newline-only preservation for external formatters so it also keeps the original missing final newline when the external formatter makes no other textual changes at all (including completely empty files)
- added API and CLI regressions for otherwise empty real-package `.svelte` inputs so newline-only EOF changes on empty external-plugin files stay green in both direct formatting and `--check` / `--list-different`

Notes:
- this pass is driven by the latest verification report: the remaining `oxlint` reds were the 4 real-package Svelte e2e fixtures still failing during config import, while the remaining `oxfmt` realworld diff was a newline-only change on `src/routes/profile/+page.svelte`
- the Oxlint change is intentionally conservative: it still allows real fixture failures to surface normally, and only converts errors into skips when the config import failure can be traced back to one of the fixture’s declared optional packages
- the Oxfmt change stays limited to external formatter entries, so Oxc-native formatter files continue to obey the existing final-newline setting exactly as before


## Stabilization pass — 2026-04-03 (external formatter empty-file EOF panic follow-up)


Changes:
- replaced the `apps/oxfmt/src/core/format.rs` empty-EOF preservation fast path with a no-op-safe helper so external formatter results that already match the source text no longer assume a trailing newline exists before truncation
- kept the existing newline-only preservation behavior for real external formatter outputs that *do* append a single trailing linebreak, including `\n`, `\r\n`, and `\r`
- added focused Rust unit coverage for both sides of that branch: same-text preservation without panic (including empty files) and single-linebreak trimming when the external formatter added one

Notes:
- this pass is driven directly by the newest verification report, where `oxlint_vitest` is already green and the remaining real failures are the empty-file EOF panic in `apps/oxfmt/src/core/format.rs` plus the two corresponding CLI tests in `plugin_languages_real_package.test.ts`
- the fix is intentionally minimal: it changes only the final external EOF-preservation step and leaves the broader formatter selection, parser routing, and newline policy untouched


## Phase 6C — Real-package Svelte lint CI manifest + direct runtime coverage


Changes:
- added `apps/oxlint/scripts/svelte-real-package-metadata.ts` as the single source of truth for the dedicated real-package Svelte lint lane, including the pinned upstream package specs and the exact fixture set the lane is expected to cover
- rewired `apps/oxlint/scripts/install-real-svelte-packages.ts` to use that explicit manifest instead of discovering fixtures heuristically from `requiredPackages`, and to link `apps/oxlint/test/node_modules` in addition to the fixture-local `node_modules` directories so the direct real-package parser/plugin unit tests can resolve upstream packages too
- expanded `apps/oxlint/package.json`'s `test:svelte-real-packages` script to run the direct `real_svelte_parser_whole_file` and `real_svelte_plugin_whole_file` tests alongside the focused CLI fixture suite and built-vs-raw smoke checks
- added `apps/oxlint/test/svelte_real_package_manifest.test.ts` so future real-package Svelte fixtures must be added to the CI manifest deliberately instead of being silently skipped or picked up by accident

Notes:
- this phase makes the lane more deterministic: adding a new real-package Svelte fixture now requires touching one explicit manifest, and the CI lane will fail if the fixture metadata drifts from that manifest
- linking `apps/oxlint/test/node_modules` is intentionally scoped to the helper setup step used by the dedicated lane; it exists so direct unit tests that import `svelte-eslint-parser` or `eslint-plugin-svelte` from the test directory resolve the same pinned packages as the CLI fixtures
- the lane now covers both levels of confidence the Svelte work needs: end-to-end CLI fixtures and direct whole-file parser/plugin runtime tests against the upstream packages


## Phase 6D — Real-package Svelte lint lane preflight + cleanup ergonomics


Changes:
- added `apps/oxlint/scripts/check-real-svelte-packages.ts` to verify the dedicated helper install is present, that every pinned upstream package version matches the manifest, and that both the test-root and fixture-local `node_modules` links resolve to the shared install directory before the lane runs
- added `apps/oxlint/scripts/cleanup-real-svelte-packages.ts` so developers can remove the shared install directory and every symlink the helper created without manually cleaning up `apps/oxlint/test/node_modules` and the fixture directories
- expanded `apps/oxlint/package.json` with `check:svelte-real-packages` and `clean:svelte-real-packages`, and made `test:svelte-real-packages` run the preflight check automatically so half-wired local runs fail with a clear setup error instead of later package-resolution noise

Notes:
- this phase is mostly ergonomics and determinism: the CI lane now has an explicit preflight gate, and local developers get a supported cleanup path for the extra test-directory symlink introduced in Phase 6C
- the cleanup script intentionally refuses to delete a non-symlink `node_modules` path in the test tree, so it will not wipe a manually created directory by accident
- the preflight script checks the pinned package versions from the manifest rather than just package presence, which keeps the helper lane aligned with the intended upstream compatibility targets


## Phase 6E — Real-package Svelte profile support for pinned CI vs latest-upstream canaries


Changes:
- extended `apps/oxlint/scripts/svelte-real-package-metadata.ts` with explicit real-package profiles so the existing CI lane stays pinned by default while a separate `latest-svelte` profile can float only the Svelte ecosystem packages (`svelte`, `svelte-eslint-parser`, `eslint-plugin-svelte`)
- taught `install-real-svelte-packages.ts`, `check-real-svelte-packages.ts`, `report-real-svelte-packages.ts`, and `run-real-svelte-package-tests.ts` to accept `--profile`, propagate that profile through the managed runner, and render profile-aware diagnostics and report paths
- added convenience scripts in `apps/oxlint/package.json` for the `latest-svelte` profile and updated the manifest test so the default profile remains pinned while the canary profile's floating package set stays narrow and explicit
- generalized report cleanup/ignore patterns so both the pinned lane and any future profile-specific canary reports are cleaned up and ignored consistently

Notes:
- the important product decision here is that the mandatory CI lane remains deterministic and pinned; the new profile support exists so maintainers can add an upstream-drift canary without turning every PR into a moving-target compatibility test
- the `latest-svelte` profile intentionally floats only the Svelte-facing packages; `eslint`, `typescript`, and `@typescript-eslint/parser` stay pinned so canary failures are easier to attribute to Svelte ecosystem changes instead of unrelated upstream churn
- floating profiles always reinstall instead of reusing an existing helper install, so a local or scheduled canary run picks up fresh upstream releases rather than silently reusing an older cached helper directory


## Phase 6F — Scheduled latest-upstream Svelte canary workflow


Changes:
- added `.github/workflows/oxlint-svelte-upstream-canary.yml` as a dedicated scheduled/manual workflow that runs the real-package Svelte lane against the `latest-svelte` profile instead of the pinned mandatory CI profile
- the new workflow builds `apps/oxlint`, installs and verifies the `latest-svelte` helper profile, runs runtime/fixture/smoke/LSP suites in one canary job, then always collects and uploads a profile-specific markdown/json diagnostics report
- the workflow summary now records the resolved latest Svelte package versions for each canary run, which makes upstream breakage easier to attribute when a new `svelte`, `svelte-eslint-parser`, or `eslint-plugin-svelte` release lands

Notes:
- this keeps the mandatory PR lane deterministic while still giving maintainers continuous signal about upstream drift in the Svelte ecosystem
- the canary is intentionally Linux-only for now; the goal is early warning when upstream packages change, not duplicating the full pinned matrix across every platform
- the canary report is always uploaded, not just on failure, so maintainers can see exactly which latest upstream versions were exercised on a green run too

