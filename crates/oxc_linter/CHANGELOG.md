# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.11.0] - 2025-08-07

### 🚀 Features

- ac46347 oxlint: Add `tsgolint` integration (#12485) (camchenry)

### 🐛 Bug Fixes

- 9c4bd42 linter/jest/expect-expect: Add support for expect in array expressions (#12877) (camc314)
- 6b4a7a7 linter: Prevent category rules from being reapplied to already-configured plugins in overrides (#12875) (camc314)

### ⚡ Performance

- 2f8fc31 linter/new-cap: Use iterator chaining instead of cloning (#12879) (camc314)

### 🧪 Testing

- e15093c linter/new-cap: Add tests for `Intl.DateTimeFormat` (#12878) (camc314)


## [1.10.0] - 2025-08-06

### 🚀 Features

- 44ac5a1 linter: Add eslint/no-unassigned-vars rule (#11365) (yefan)
- ce6eeee linter: Add `eslint/prefer-destructuring` rule (#12721) (yefan)
- 9b35600 linter/jsx-a11y: Add support for mapped attributes in label association checks (#12805) (camc314)
- a754f7a linter: Support `countVoidThis` option in `max-params` rule (#12604) (yefan)

### 🐛 Bug Fixes

- 2c1dab6 linter/no-unassigned-vars: False positive with variables in for loop (#12833) (camc314)
- 5a24574 linter/func-style: Fix more false positives (#12828) (camc314)
- 33a7320 linter/no-throw-literal: Fix unconditional recursion in `could_be_error` (#12819) (camc314)
- a3aec6a linter/explicit-module-boundary-types: Debug assertion fail with top level return (#12820) (camc314)
- 6efe457 linter/no-empty-function: Respect allow options for functions and arrow functions (#12814) (camc314)
- 1c21c46 linter/new-cap: Fix panic with computed member expr (#12804) (camc314)
- 45206dd linter: Apply fix span offset after fixing the section source text (#12758) (Sysix)
- 1e97e35 linter/unicorn/prefer-structured-clone: Update Default implementation for `PreferStructuredCloneConfig` (#12791) (camc314)
- d382159 linter/unicorn/prefer-object-from-entries: Update Default implementation for `PreferObjectFromEntriesConfig` (#12790) (camc314)
- b07d29c linter/typescript/no-this-alias: Update Default implementation for `NoThisAliasConfig` (#12789) (camc314)
- 0db34ab linter/react/jsx-filename-case: Update Default implementation for `JsxFilenameExtensionConfig` (#12788) (camc314)
- ff84eff linter/jest/prefer-lowercase-title: Update Default implementation for `PreferLowercaseTitleConfig` (#12787) (camc314)
- 5175c6d linter/jest/no-large-snapshots: Update Default implementation for `NoLargeSnapshotsConfig` (#12786) (camc314)
- 0eaebcd linter/jest/no-deprecated-functions: Update Default implementation for `JestConfig` (#12785) (camc314)
- 4265db7 linter/import/no-anonymous-default-export: Update Default implementation for `NoAnonymousDefaultExport` config (#12784) (camc314)
- 6a360e3 linter/import/extensions: Update Default implementation for ExtensionsConfig (#12783) (camc314)
- 42c8f29 linter: Default options for `eslint/no-else-return` (#12762) (Sysix)
- 4eac511 linter: Default options for `eslint/no-unneeded-ternary` (#12761) (Sysix)
- 9c01dbf linter: Default options for `eslint/new-cap` (#12760) (Sysix)
- b25406f linter/explicit-function-return-types: Update default values in ExplicitFunctionReturnTypeConfig (#12718) (camc314)
- ce5876d parser: Validate inner expression of type assertions in assignment targets (#12614) (camc314)
- 5383331 linter/explicit-mod-boundary-types: False positive with jsx elements (#12678) (camc314)
- d0e99b5 linter/explicit-mod-boundary-types: False positive with call expressions (#12677) (camc314)
- 525137e linter: Add missing options to no-inner-declarations (#12661) (camc314)
- fc4a327 linter: No-unused-vars false positive with class property initializers (#12660) (camc314)
- 6af8631 linter/no-unused-vars: False positive with chain expression (#12609) (camc314)
- 744ef52 linter: Correct `array-type` handling of `default: 'array-simple'` (#12607) (yefan)

### 🚜 Refactor

- 3f37ed1 linter: Replace `lazy_static` with `std::sync::LazyLock` (#12822) (Copilot)
- 69fd08d semantic: Improve unused label tracking and add debug assertions (#12812) (camc314)
- c0e224a linter: Store `ExternalRuleId` in `OxlintOverrides` not raw names (#12502) (camc314)
- 61587e4 linter: Correct comment (#12792) (overlookmotel)
- 5adcb98 linter: Use `u32` to keep track of last fixed source text position (#12696) (Sysix)
- 77acc11 linter, transformer: Use `Scoping::symbol_is_unused` (#12666) (overlookmotel)
- ecf1cff language_server: Simplify offset adjustment by using `Message.move_offset` (#12647) (Sysix)
- 7695393 linter: Simplify offset adjustment by using `Message.move_offset` (#12595) (Sysix)
- b36dc92 linter: Refactor large arrays to reduce binary size (#12603) (Boshen)
- 3b9f1f0 linter: Update iter_outer_expressions to take AstNodes reference (#12583) (camc314)

### 📚 Documentation

- e760fd4 linter: Complete linter rules documentation with missing "Why is this bad?" sections (#12757) (Copilot)
- 514322c rust: Add minimal documentation to example files in crates directory (#12731) (Copilot)
- 1d910d5 linter: Fix typescript/consistent-type-imports rule options to match TypeScript ESLint (#12707) (Copilot)
- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)
- 7660a88 linter: Improve linter rule documentation with "Why is this bad?" sections and enhanced examples (#12711) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)

### ⚡ Performance

- 09ae2a9 linter: Eliminate unnecessary Iterator::collect() allocations (#12776) (Copilot)

### 🎨 Styling

- cacbd1e linter/no-empty-function: Order match arms consistently (#12815) (camc314)
- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)

### 🧪 Testing

- 3957fcc linter/no-undef: Add test case for `TSImportType` (#12800) (camc314)
- c6bfb8a linter: Add rule configuration consistency test (#12744) (camc314)
- 2ceb835 linter: Fix offset for partical source texts (#12594) (Sysix)


## [1.9.0] - 2025-07-29

### 💥 BREAKING CHANGES

- 5a7e72a semantic: [**BREAKING**] `AstNodes::program` return `&Program` not `Option<&Program>` (#12515) (overlookmotel)

### 🚀 Features

- 3489ce0 linter: Add `typescript-eslint/explicit-module-boundary-types` (#12402) (Don Isaac)

### 🐛 Bug Fixes

- 0fd3e87 linter: Default options for `eslint/yoda` (#12540) (Sysix)
- 724776f linter: Default options for `unicorn/switch-case-braces` (#12539) (Sysix)
- fda45ea linter/promise/prefer-await-to-callbacks: False positive for `addEventListener` (#12537) (Copilot)
- 1a710e3 linter/array-type: Fix more false negatives (#12501) (camc314)
- 2b5bf98 linter: Consistent-function-scoping false positive with hoisted var declarations (#12523) (camc314)
- 209d006 linter: Parse vue lang attribute without quotes (#12517) (Sysix)
- 85a34ce linter/array-type: False negative with arrays in generic args (#12500) (camc314)
- 98c1fbb linter/require-await: Improve async keyword detection in get_delete_span function (#12494) (camc314)
- 7c75dba linter/require-await: Improve span calculation for object properties (#12490) (camc314)
- 2b261cf linter/exhaustive-deps: False positive in exhaustive deps (#12471) (camc314)

### 🚜 Refactor

- a696227 linter: Remove AstKind for SimpleAssignmentTarget (#12401) (Tyler Earls)
- 7af38e1 napi/oxlint: Simplify `ExternalLinterLintFileCb` type (#12572) (overlookmotel)
- 543fd53 napi/oxlint: Rename `run` to `lintFile` (#12567) (overlookmotel)
- 0179c86 napi/oxlint: Reverse args of `ExternalLinter::new` (#12566) (overlookmotel)
- 491c401 linter: Remove `#[must_use]` from `LintService::with_*` methods (#12560) (overlookmotel)
- bea652f linter: Add `vue` and `regex` to `BuiltinLintPlugins` (#12542) (Sysix)
- aa9dd21 linter/no-eval: Get source type from `Semantic` (#12514) (overlookmotel)
- 8c8c8bc napi/oxlint: Diagnostics communicate which rule via rule index, not rule ID (#12482) (overlookmotel)
- e2d9b4d fixer: Add Debug trait to PossibleFixes and Message structs (#12493) (camc314)
- f0b1f0d napi/oxlint, napi/parser: Remove source length from `RawTransferMetadata` (#12483) (overlookmotel)
- 7e4959a napi/oxlint: Rename `rules` to `ruleNames` (#12477) (overlookmotel)
- 7a0da04 diagnostics: Remove Option wrapper from MPSC channel and sender field (#12467) (camc314)

### 🧪 Testing

- 56468c7 linter/no-unused-private-class-members: Add more test cases (#12569) (camc314)
- 191a164 linter/no-unused-private-class-members: Add more test cases (#12563) (camc314)


## [1.8.0] - 2025-07-22

### 🚀 Features

- 6d2b549 napi/oxlint: Pass AST in buffer to JS (#12350) (overlookmotel)
- 14f0159 linter/exhaustive-deps: Add auto-fixer (#12354) (Don Isaac)

### 🐛 Bug Fixes

- 99e105f linter: Correct autofix in `unicorn/prefer-number-properties` for Infinity (#12445) (yefan)
- 0b539e3 linter: `unicorn/catch-error-name` wasn't using the ignore property (#12446) (Parbez)
- 05fba9b linter: Don't panic on `TSNonNullExpression` in `unicorn/prefer-array-find` (#12400) (Sysix)
- 4621872 linter: Parse second script block for `svelte` files (#12390) (Sysix)
- fbe7eb4 linter/filename-case: Fix default config when no config is provided (#12389) (camc314)
- fea9df4 linter: Report errors with the correct severity for custom plugins (#12362) (camc314)
- 652c038 linter: Mark correctly enabled default rules for `--rules` (#12163) (Sysix)
- eadc359 linter: Correct source text for vue files having script attributes containig ">" char inside (#12375) (Sysix)
- 54d143a linter/exhaustive-deps: More descriptive messages for always-rerender violations (#12336) (Don Isaac)
- dac4db9 linter/exhaustive-deps: Better diagnostics for missing dependencies (#12337) (Don Isaac)
- 119d23a linter/prefer-array-flat-map: Error for `.flat(1.0)` (#12360) (overlookmotel)

### 🚜 Refactor

- 2d9291c linter/prefer-number-properties: Simplify fixer logic (#12451) (camc314)
- c5dff1e linter, napi/parser: Add `source_len` field to `RawTransferMetadata` (#12383) (overlookmotel)
- 5e3b415 linter: Duplicate `RawTransferMetadata` in `oxc_linter` crate (#12382) (overlookmotel)
- 773fd88 linter: Pass `&Allocator` into `Linter::run_external_rules` (#12374) (overlookmotel)
- b10ed11 linter: Make unwrap unconditional (#12371) (overlookmotel)
- a0631d4 linter: Move running external rules into feature-gated function (#12370) (overlookmotel)
- 4fc4e7c linter: Make feature gates for `oxlint2` feature consistent (#12369) (overlookmotel)
- 50b1786 linter: Clarify usage of `Allocator` and `AllocatorGuard` (#12332) (overlookmotel)
- 26d3a39 linter: Remove `ModuleContentOwner` abstraction (#12331) (overlookmotel)

### 📚 Documentation

- 3c21d94 linter: Correct comment on `RawTransferMetadata2` type (#12428) (overlookmotel)


## [1.7.0] - 2025-07-16

### 🚀 Features

- 5e428a4 linter/eslint-plugin-next: No-html-link-for-pages rule addition (#12194) (Gabriel Díaz Aguilera)
- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)
- c551b8f linter: Report diagnostics from custom plugins (#12219) (camc314)
- d387729 linter: JS custom rules config (#12160) (camc314)
- bde1ef1 linter: Load custom JS plugins (#11980) (camc314)
- d4ebd14 linter: Add `oxlint2`/`disable_oxlint2` feature flags (#12130) (camc314)
- a4dae73 linter: Introduce `LintPlugins` to store builtin + custom plugins (#12117) (camc314)

### 🐛 Bug Fixes

- 3f9a1f0 linter/no-unused-private-class-members: Fix false positive with nullish coalescing assignments (#12317) (camc314)
- 47fad0e linter/no-empty-file: False positive with empty file with triple slash directive (#12293) (camc314)
- 633ba30 linter: False positive with unknown plugins when unmatched eslint rule (#12285) (camc314)
- 98708eb linter: Fix inconsistent behavior in `no-duplicate-imports` rule (#12192) (yefan)

### 🚜 Refactor

- ee761de ast: Remove `AstKind` for `AssignmentTarget` (#12252) (Tyler Earls)
- c68b607 ast: Rename `TemplateLiteral::quasi` to `TemplateLiteral::single_quasi` (#12266) (Dunqing)
- 32c32af ast: Check whether there is a single `quasi` in `TemplateLiteral::quasi` (#12265) (Dunqing)
- 8f6a1da linter/js-plugins: Use `u32` for IDs (#12243) (overlookmotel)
- 36cd364 linter/js-plugins: Clean up code (#12242) (overlookmotel)
- 8c02ebd linter/js-plugins: Rename `specifiers` to `paths` (#12241) (overlookmotel)
- 3adaf98 linter: Simplify getting nodes count (#12239) (overlookmotel)
- 6e54645 language_server: Store `LintService` instead of `Linter` (#12016) (Sysix)
- 113cf8c linter: Move `LintServiceOptions.paths` to `LintService.with_paths` (#12015) (Sysix)
- 729b82b linter: Rename `plugin_name` to `plugin_specifier` (#12148) (overlookmotel)
- 532b816 linter: Use `to_string` instead of `into` (#12147) (overlookmotel)
- 89f2a69 linter: TODO comment (#12146) (overlookmotel)
- f90d3e1 linter: Feature gate `load_external_plugin` by both `oxlint2` and `disable_oxlint2` features (#12141) (overlookmotel)
- 12e4ec7 linter: Make `tokio` dependency optional (#12140) (overlookmotel)
- 1d2eaca oxlint2: Introduce `force_test_reporter` feature for consistent graphical outputs (#12133) (camc314)
- 8814c53 ast: Remove `AstKind` for `PropertyKey` (#12108) (camchenry)
- 228cff5 semantic,linter: Assert that Program is always the first node (#12123) (Ulrich Stark)
- e8e2a25 ast: Remove `AstKind` for `AssignmentTargetPattern` (#12105) (camchenry)
- f7c675d linter: Rename `LintPlugins` to `BuiltinLintPlugins` (#12116) (camc314)
- a9e5ec0 linter: Access plugins through config instead of storing directly (#12115) (camc314)
- 9736a7f linter: Simplify `unicorn/require-post-message-target-origin` (#12110) (shulaoda)

### 📚 Documentation

- 2e3db46 linter: Add missing backtick preventing website from building (#12113) (camc314)

### ⚡ Performance

- d0f8b88 linter/js-plugins: Do not copy `Vec` (#12248) (overlookmotel)
- 4284d19 linter/js-plugins: Use hashmap `Entry` API + remove temp `Vec` (#12247) (overlookmotel)
- c7889c3 semantic,linter: Simplify implementation and uses of ancestors iterators (#12164) (Ulrich Stark)
- f99959c linter: Move work out of loop (#12145) (overlookmotel)
- 514d40c linter: Do not create `Resolver` unless required (#12142) (overlookmotel)
- 7103527 linter/no-constructor-return: Optimize loop (#12138) (overlookmotel)


## [1.6.0] - 2025-07-07

### 🚀 Features

- f81d336 linter: Introduce `ExternalLinter` struct (#12052) (camc314)

### 🐛 Bug Fixes

- 3f0e03e linter: Fix inconsistent behavior in `no-duplicate-imports` rule (#12051) (yefan)
- 6dbacea linter/no-barrel-file: No diagnostic tag when some modules arent resolved (#12049) (camc314)
- dd6b1ee linter/extensions: False positives with non configured extensions (#11872) (camc314)
- eb1c596 linter/consistent-index-object-style: Fix default impl for rule config (#12031) (camc314)

### 🚜 Refactor

- 54cf5cb semantic: Remove Option from parent_* methods (#12087) (Ulrich Stark)
- 8d1be94 language_server: Fix todo by avoiding allocation (#12096) (Ulrich Stark)
- 72418ca linter: `RuntimeFileSystem::write_file` take `&str` (#12075) (overlookmotel)
- 2f7cbda linter: Move napi bindings out of oxc_linter (#12072) (camc314)
- 2319710 linter: Shorten code (#12070) (overlookmotel)
- 387762d linter/no-unused-vars: Simplify check for export nodes (#12044) (Dunqing)
- f1d4086 ast: Remove `AstKind` for `ModuleDeclaration` (#12022) (camchenry)
- 754c05a ast: Remove `AstKind` for `TSTypeName` (#11990) (camchenry)
- 0c7f9e8 linter: Remove a branch (#12032) (overlookmotel)

### 📚 Documentation

- 85ec382 linter: Add good/bad example for `nextjs/no-page-custom-font` (#12092) (Sysix)
- 9240342 linter: Add docs for `nextjs/no-script-component-in-head` (#12091) (Sysix)
- 0878414 linter: Add good/bad example for `nextjs/no-head-import-in-document` (#12061) (Sysix)
- 222bc73 linter: Add bad/good example for `nextjs/no-head-element` (#12059) (Sysix)
- a7e9f50 linter: Add good/bad example for `nextjs/no-title-in-document-head` (#12065) (Sysix)
- 51c6818 linter: Add good/bad example for `nextjs/no-document-import-in-pages` (#12064) (Sysix)
- c7b38f9 consistent-indexed-object-style: Clarify docs (#12019) (Luca Ban)

### ⚡ Performance

- 04e2de5 linter: Avoid iteration when checking import is `AllButDefault` or `SideEffect` in `eslint/no-duplicate-imports` (#12093) (Sysix)
- e2a7d6a linter: Check filepath before running `nextjs/no-head-element` (#12062) (Sysix)
- 00a9fd9 linter: Check for filename before running `nextjs/no-head-import-in-document` rule (#12060) (Sysix)
- 62a3ce0 linter: Replace `unicode-segmentation` crate with `icu_segmenter` (#12063) (Sysix)


## [1.5.0] - 2025-07-02

### 🚀 Features

- 899b8b4 linter: Allow namespace re-export in `import/no-cycle` (#11995) (Boshen)

### 🐛 Bug Fixes

- f732589 linter: Panic in `consistent-type-imports` when the source contains a `{` (#12002) (camc314)

### 🚜 Refactor

- f7a2ae4 ast: Add `AstKind` for `AssignmentTargetPropertyIdentifier`, `AssignmentTargetPropertyProperty` (#11985) (camc314)
- cfa52c2 ast: Add `AstKind` for `AssignmentTargetRest` (#11984) (camc314)
- 3f91f24 linter: Remove `RulesCache` (#11981) (camc314)
- 54582cb ast: Add `AstKind` for `BindingProperty` (#11974) (camc314)


## [1.4.0] - 2025-06-30

### 🚀 Features

- 8e1573d linter: Add id-length rule from eslint (#11887) (Nicholas Rayburn)
- 5dfcac5 linter: Add eslint/arrow-body-style rule (#11937) (yefan)
- f102cb1 linter: Add `import/prefer-default-export` rule (#11891) (yefan)

### 🐛 Bug Fixes

- 114c4fb linter/no-useless-spread: Panic with multi byte char (#11964) (camc314)
- c2e5439 linter: Fix default values for `unicorn/consistent-function-scoping` (#11960) (Sysix)
- 214c8e7 linter: Fix default values for `import/no-absolute-path` (#11959) (Sysix)
- b4cc222 linter: Fix `typescript/no-namespace` default values (#11958) (Sysix)
- 11e0a43 linter/prefer-dom-node-remove: Panic when callee is ts non null expression (#11949) (camc314)
- 4903e39 linter/no-standalone-expect: False positive in callback fn (#11940) (camc314)
- 1e88dba oxc_linter: Make linter file paths clickable within JetBrains terminals (#11619) (Nicholas Rayburn)
- fe4006b linter/jsx-key: False positive in react/jsx-key (#11918) (camc314)
- d32cb4b linter: Fix default values for `eslint/no-redeclare` (#11911) (Sysix)
- d80c19d linter: Fix default values for `eslint/max-lines-per-function` (#11910) (Sysix)
- 41a5342 linter: Fix default values for `eslint/max-nested-callbacks` (#11909) (Sysix)
- 2e3db4e linter: Fix default values for `eslint/max-depth` (#11908) (Sysix)
- a358797 linter: Remove false positives for `no-extend-native` (#11888) (camchenry)
- 6f67b52 linter: Revert prefer-promise-reject-errors to old behavior (#11889) (camchenry)
- d991fed linter: Fix `jsx-a11y/label-has-associated-control` default values (#11832) (Sysix)
- a0a4aa1 linter: Count own indirect export entries to the threshold for `oxc/no-barrel-file` (#11838) (Sysix)

### 🚜 Refactor

- 344f3f9 linter: Minor refactors to `eslint/id-length` (#11976) (camc314)
- 46b59d8 linter: Remove unused `ContextHost::with_config` (#11970) (camc314)
- 17e0898 linter: Move `import/no-duplicates` to style category (#11929) (Sysix)
- dd2e196 linter: Move `unicorn/no-nested-ternary` to style category (#11928) (Sysix)
- 8404da4 linter: Remove unused `LintPluginOptions` (#11919) (camc314)
- 87b8496 ast: Remove `AstKind` for `MemberExpression` and replace with `StaticMemberExpression` and `PrivateFieldExpression` (#11767) (camchenry)
- e840680 linter/no-named-as-default-members: Remove needless lambda (#11896) (camc314)
- 2760591 linter/no-console: Early return if ident does not match (#11895) (camc314)
- 190e390 ast: Add `AstKind` for `ComputedMemberExpression` (#11766) (camchenry)

### 📚 Documentation

- 940b98f linter: Fix docs for `typescript/no-this-alias` (#11963) (Sysix)
- c4a95a2 linter: Move `jest/valid-title` options into a codeblock (#11961) (Sysix)
- 0d3e8e7 linter: Fix prefer-string-replace-all example (#11921) (Bruno Henriques)
- 06781ab linter: Fix doc formatting for perfer-logical-op-over-ternary (#11920) (camc314)

### ⚡ Performance

- 2cf63ea linter: `nextjs/no-document-import-in-page` check the filepath before running the rule (#11962) (Sysix)
- 19cee8c linter/no-extend-native: Do not create unnecessary `CompactStr` (#11885) (overlookmotel)
- 66dbf9d linter/no-console: Get static property name only once (#11880) (overlookmotel)

### 🧪 Testing

- 168f776 linter/no-console: Add more tests (#11878) (overlookmotel)


## [1.3.0] - 2025-06-23

### 🚀 Features

- 1a54184 linter: Add fix for unused disable directive (#11708) (Sysix)
- 816ff03 linter: Read source text into the arena (#11825) (camc314)
- dc6f584 linter: Add `read_to_arena_str` function (#11823) (overlookmotel)

### 🐛 Bug Fixes

- 76891da linter/exhaustive-deps: False positive with ident used in type param (#11812) (camc314)

### 🚜 Refactor

- b39d1fa linter: Output smaller spans for unused disable directives with multiple rules (#11781) (Sysix)

### 📚 Documentation

- faab3ee linter: Improve docs for typescript/no-this-alias (#11845) (camc314)


## [1.2.0] - 2025-06-19

### 🚀 Features

- 8c341a2 sema/check: Ts setters cannot have initializers (#11695) (Don Isaac)
- 38dc614 oxc_linter: Reuse allocators (#11736) (camc314)
- bf8263d playground: Allow specifying a JSON string as the linter config (#11710) (Nicholas Rayburn)
- 52ecc87 linter: Implement import/extensions (#11548) (Tyler Earls)

### 🐛 Bug Fixes

- 3d88eeb linter/no-console: False negative when `console.*` methods are used as args to functions (#11790) (camc314)
- c80e405 linter/no-new-wrappers: Fix panic in fixer with multi byte chars (#11773) (camc314)
- e58a0b0 linter: Panic in unicorn/consistent-function-scoping (#11772) (camc314)
- 80c87d4 linter: Typo in typescript/consistent-index-object-style (#11744) (camc314)
- ff775e9 linter/consistent-function-scoping: Descriptive diagnostic labels (#11682) (Don Isaac)
- 989634a linter/no-inner-declaration: False negative with for loops (#11692) (camc314)
- b272b91 linter/no-undef: False negative with unresolved ref after type ref (#11721) (camc314)
- 6252275 linter: Panic in import/extensions with empty file names (#11720) (camc314)
- f34e432 linter: Use fixer::noop in dangerous cases for eslint/no-var (#11693) (camc314)
- 6c2b41c linter/consistent-function-scoping: Allow functions in TS modules/namespaces (#11681) (Don Isaac)
- 2ca1c70 linter/exhaustive-deps: False positive with TS Non null assertion operator (#11690) (camc314)
- ee15f7d linter: False negative in typescript/prefer-function-type (#11674) (camc314)
- abd0441 linter: Add missing menuitemradio and menutitemcheckbox roles (#11651) (Daniel Flynn)
- 8776301 linter/no-inner-declarations: Flag `var` statement as body of `for` loop (#11632) (overlookmotel)

### 🚜 Refactor

- 5ca3d04 ast: Add `TSArrayType` as `AstKind` (#11745) (camchenry)
- 219adcc ast: Don't generate AstKind for ArrayExpressionElement (#11684) (Ulrich Stark)
- c1be6b8 linter: Shorten Span construction (#11686) (Ulrich Stark)
- 4ca659c linter: Cleanup typescript/prefer-function-type  (#11672) (Brad Dunbar)
- 8e30c5f ast: Don't generate AstKind for ForStatementInit (#11617) (Ulrich Stark)

### 📚 Documentation

- ea6ce9d linter: Fix typo in import/no-namespace (#11741) (camc314)
- 8b6076e linter: Document options for the `typescript/array-type` rule (#11665) (yefan)

### ⚡ Performance

- f539f64 allocator: Remove `Arc` from `AllocatorPool` (#11760) (overlookmotel)
- cfdc518 linter/no-inner-declarations: Move work to cold path (#11746) (overlookmotel)
- 7c0fff7 linter: Skip running `consistent-function-scoping` on `.d.ts` files (#11739) (camc314)
- b34c6f6 parser,semantic: Improve handling of diagnostics (#11641) (Boshen)
- 2cd786b linter/no-inner-declarations: Remove unnecessary code and reduce branches (#11633) (overlookmotel)

### 🧪 Testing

- 44a9df8 linter: Update testsuite for `no-undef` (#11706) (Sysix)


## [1.1.0] - 2025-06-12

### 🚀 Features

- 1181018 linter: Add eslint/no-extra-bind rule (#11588) (yefan)
- 3b03fd3 parser: Produce correct syntax error for `interface I extends (typeof T)` (#11610) (Boshen)
- 844a8a8 parser: Produce syntax error for `declare function foo() {}` (#11606) (Boshen)

### 🐛 Bug Fixes

- 0f24366 linter: Correct labels for redundant comparisons (#11620) (Wei Zhu)
- dd87f93 linter: Stack overflow in react/exhaustive-deps (#11613) (camc314)
- 4af58e0 linter: Add missing `additional_hooks` option to exhaustive-deps (#11602) (camc314)
- a6adc0c linter/exhaustive-deps: Handle destructuring inside hooks (#11598) (Don Isaac)
- 779727f linter: Improve span diagnostic loc within react/rules-of-hooks (#11589) (camc314)

### 🚜 Refactor

- b7b0dc3 parser: Improve `TSModuleDeclaration` parsing (#11605) (Boshen)
- d29bbb2 linter: Simplify implementation of `eslint/no-lonely-if` (#11550) (Ulrich Stark)
- d41fb13 ast: Get jsx types out of AstKind exceptions (#11535) (Ulrich Stark)


## [1.0.0] - 2025-06-10

## [0.18.1] - 2025-06-09

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🐛 Bug Fixes

- 6d68568 linter: False negative in typescript/array-type (#11574) (camc314)
- 6a856a0 linter/no-magic-numbers: Fix typo in error message (#11560) (overlookmotel)
- 3952e01 linter: False negative in jsx-a11y/aria-role (#11547) (camc314)
- b0e3e08 linter: Misplaced quote in jsx-curly-brace-presence test case (#11546) (camc314)
- a833ed1 linter: Misplaced quote in anchor-is-valid test case (#11545) (camc314)
- 4e53b80 linter: Misplaced backtick in exhaustive-deps test case (#11544) (camc314)
- e8a04b6 linter: Misplaced backtick in no-object-constructor test case (#11543) (camc314)
- 65311d0 linter: Remove duplicate rule/scope from diagnostic (#11531) (camc314)

### 🚜 Refactor

- 9b475ad linter: Use one instance of rope per file (#11552) (Sysix)

### 📚 Documentation

- fa924ab linter: Cleanup docs for multiple linter rules (#11551) (Ulrich Stark)

## [0.18.0] - 2025-06-06

### Features

- 825d40c linter: Fix casing in unicorn/no-useless-promise-resolve-reject (#11528) (camc314)
- 2faee3d linter: Fix grammer in react/exhaustive-deps (#11527) (camc314)

### Bug Fixes

- 1a71d7c linter: Misplaced backtick in unicorn/no-array-for-each diagnostic (#11529) (camc314)
- 7430b14 linter: Grammer in jest/valid-expect diagnostic (#11522) (camc314)
- b92ac41 linter: Grammer in eslint/no-redeclare diagnostic (#11521) (camc314)
- 17883e3 linter: Improve eslint/no-unsafe-negation diagnostic (#11520) (camc314)
- 8c89937 linter: Improve eslint/no-shadow-restricted-names diagnostic (#11519) (camc314)
- 3f0d889 linter: Add missing article to oxc/bad-array-method-on-arguments diagnostic (#11518) (camc314)
- cf0c24c linter: Improve message in react/prefer-es6-class diagnostic (#11516) (camc314)
- 91855df linter: Fix message in react/rules-of-hooks diagnostic (#11515) (camc314)
- b272194 linter: Misplaced backtick in jest/no-conditional-expect diagnostic (#11514) (camc314)
- 3403303 linter: Misplaced backtick in unicorn/prefer-dom-node-dataset diagnostic (#11513) (camc314)
- d5ca872 linter: Misplaced backtick in eslint/radix diagnostic (#11512) (camc314)
- 2dcf8be linter: Improve diagnostic message when function name is referenced (#11509) (camc314)
- 0de0c9c linter: Improve diagnostic message for no-unsafe-declaration-merging (#11508) (camc314)
- 0946dac linter: Correctly inherit categories when plugins are enabled (#11353) (Cameron)
- 510c1c6 linter: Add missing `allowArrowFunctions` option for eslint/func-style (#11455) (yefan)
- c77787c linter: Improve `eslint/no-loss-of-precision` (#11437) (magic-akari)
- 11d4523 linter: False positive in react/exhaustive-deps (#11438) (camc314)
- 616b613 linter/switch-case-braces: Align the logic with `unicorn` (#11405) (shulaoda)

### Refactor

- 0fdc51e linter: Simplify `OxlintRules::override_rules` (#11510) (camc314)

## [0.17.0] - 2025-05-30

- ead5309 linter: [**BREAKING**] Remove react from default plugin set (#11382) (camc314)

### Features

- 2d25bd8 linter: Remove `unicorn/no-for-loop` over `typescript/prefer-for-of` (#11354) (camc314)
- bbb7eb1 linter: Add auto-fix to react/forward-ref-uses-ref (#11342) (yefan)
- 590c27b linter: Add auto-fix to unicorn/no-await-expression-member (#11306) (yefan)
- 7824f01 linter: Implement suggestion for `jsx/no-useless-fragment` (#10800) (Cam McHenry)
- 2083d33 linter/language_server: Add second editor suggestion for `react/forward-ref-uses-ref` (#11375) (Sysix)

### Bug Fixes

- 25ecbfe linter: Remove use of `FrameworkFlags::React` to decide whether rules should run (#11383) (camc314)
- 0d240e4 linter: False positive in react/exhaustive-deps with default formal parameter (#11395) (camc314)
- c91697e linter: Fix panic in multi byte char in `TryFrom` aria (#11350) (camc314)
- 9798ef1 linter: Stack overflow in no-async-endpoint-handlers (#11317) (camc314)
- 348ad97 linter: Skip no-unused-vars on astro files (#11303) (camc314)
- 183d7f0 linter: Make `jest/no-large-snapshots` error easier to comprehend (#11294) (Ulrich Stark)
- 4e606a5 linter: Improve `jest/no-large-snapshots` (#11291) (Ulrich Stark)
- 14f790f linter: Improve `jest/no-restricted-matchers` (#11292) (Ulrich Stark)
- a2c82be linter/block-scoped-var: Better diagnostic messages (#11290) (DonIsaac)
- 19772e5 linter/no-unused-vars: Panic when variable is redeclared as function in same scope (#11280) (Dunqing)

### Performance

- a0ee946 linter: Reduce code size in `globals` (#11333) (shulaoda)
- c90c5e9 linter/no-unused-vars: Simplify checking self call usage (#11281) (Dunqing)

### Documentation

- eae51ca linter: Clarify jsdoc/check-tag-names configuration (#11394) (Rägnar O'ock)

### Refactor

- 42738f0 linter: Shorten code of match arms (#11389) (Ulrich Stark)
- 8a34447 linter: Improve `unicorn/text-encoding-identifier-case` (#11386) (shulaoda)
- eaa605e linter: Avoid some `Arc::clone` in linter runtime (#11388) (Boshen)
- 1cd8b9c linter: Fixes in `react/forward-ref-uses-ref` are suggestions (#11376) (Sysix)
- 50ef691 linter: Add `diagnostics_with_multiple_fixes` to `LintContext` (#11357) (Sysix)
- 606bb34 linter: Accept `PossibleFixes`  instead of `Fix` for `Messages` (#11295) (Sysix)
- 042a3f3 linter: Use `PossibleFixes` instead of `Option<Fix>` (#11284) (Sysix)
- ffcfb46 linter: Improve `unicorn/throw-new-error` (#11364) (shulaoda)
- 8fb55c3 linter: Cleanup docs and simplify code of `eslint/no-fallthrough` (#11331) (Ulrich Stark)
- e2f0f0a linter: Improve docs and simplify code of `eslint/no-duplicate-imports` (#11320) (Ulrich Stark)
- b53b053 linter: Simplify accessing span of NameSpan (#11305) (Ulrich Stark)
- 4bc2650 linter: Improve `eslint/no-void` (#11285) (shulaoda)

### Styling

- 49b664c linter: Remove needless newline in `declare_oxc_lint` (#11400) (camc314)

### Testing

- a404b2c linter: `eslint/no-duplicate-imports` shouldn't report the same span (#11324) (Ulrich Stark)

## [0.16.12] - 2025-05-25

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- 691416a linter: Add auto-fix to unicorn/no-static-only-class (#11249) (yefan)
- 6a7018e linter: Generate stricter json schema for lint plugins (#11219) (camc314)
- 66e0b1f linter: Implement unicorn/prefer-global-this (#11197) (camc314)
- b26554b linter: Implement unicorn/no-instanceof-builtins (#11196) (camc314)
- 699ec64 linter: Add autofix to eslint/no-unneeded-ternary (#11184) (yefan)
- b3bbdda linter: Implement unicorn/prefer-object-from-entries (#11174) (camc314)
- 20f9458 linter: Implement unicorn/prefer-array-find (#11170) (camc314)
- f294c42 linter: Implement unicorn/no-array-method-this-argument (#11169) (camc314)
- 07dac71 linter: Implement unicorn/prefer-array-index-of (#11168) (camc314)
- cd920d3 linter: Implement unicorn/no-unnecessary-array-flat-depth (#11167) (camc314)
- 385b84d linter: Unicorn/no-for-loop (#11154) (camc314)
- a762038 linter: Add auto-fix to eslint/no-array-constructor (#11148) (yefan)
- 12b0917 linter: Auto-generate docs for rule configs (#10629) (DonIsaac)

### Bug Fixes

- e8470d9 linter: Delay merging of oxlintrc configs (#10835) (camc314)
- 6e9de84 linter: False positives in no-instanceof-builtins (#11210) (camc314)
- 8e7fe03 linter: Fix panic in eslint/require-await (#11211) (camc314)
- 4104b01 linter: Fix false positive on React in `consistent-type-imports` (#11171) (DonIsaac)
- 25c6266 linter: Remove duplicate test case from no-useless-escape (#11146) (camc314)
- 6a5911a linter: Add `allow_regex_characters` option to `no-useless-escape` (#11139) (camc314)
- 7283f00 linter/prefer-todo: False fix for `test['skip']` (#11128) (shulaoda)

### Performance

- 0c7aae4 linter: Speed up counting lines in `eslint/max-lines` and `eslint/max-lines-per-function` (#11242) (Ulrich Stark)
- 1846b03 linter: Avoid iterating lines twice if blank lines are skipped (#11235) (Ulrich Stark)
- e8479bf linter: Remove unnecessary `codegen` in `jest/no-untyped-mock-factory` (#11097) (shulaoda)
- dd33e57 linter: Remove unnecessary `codegen` in `eslint/prefer-numeric-literals` (#11099) (shulaoda)
- 49d677c linter: Remove unnecessary `codegen` in `jest/prefer-to-have-length` (#11100) (shulaoda)
- c294447 linter: Remove unnecessary `codegen` in `vitest/prefer-to-be-object` (#11086) (shulaoda)
- 8faf518 linter: Remove unnecessary `codegen` in `unicorn/require-number-to-fixed-digits-argument` (#11085) (shulaoda)
- e1bbdab linter/prefer-expect-resolves: Remove unnecessary `codegen` (#11127) (shulaoda)
- 6e3e37b unicorn/no-instanceof-array: Reduce memory allocations in fixer (#11109) (shulaoda)

### Documentation

- f2e3e79 linter: Fix formatting inconsistencies in rule docs (#11227) (Ulrich Stark)
- df4cc8d linter: Update missing linter rule documentation (#11190) (Aman Desai)
- b70c51e linter: Google_font_preconnect: linter rules (#11189) (Aman Desai)
- 67c0b4d linter: Explanation for rule `nextjs/no_sync_script` (#11166) (Aman Desai)
- c974f32 linter: Improve rule docs for `eslint/no-bitwise` (#11126) (Ulrich Stark)
- cbb8e0b linter: Improve rule docs for `eslint/default-case` (#11125) (Ulrich Stark)
- bcc923c linter: Normalize rule docs format (#11104) (Ulrich Stark)
- 69a14ab linter: Improve docs for `eslint/no-debugger` (#11103) (Ulrich Stark)
- 2f7346b linter: Improve docs for `eslint/no-constructor-return` (#11102) (Ulrich Stark)

### Refactor

- c64f800 linter: Introduce `ConfigStoreBuilder.extended_paths` property (#11222) (Sysix)
- e4c7614 linter: Cleanup `eslint/no-useless-constructor` (#11221) (Ulrich Stark)
- a695472 linter: Simplify finding ancestor of specific kind (#11224) (Ulrich Stark)
- b822ac8 linter: Simplify deserialization for `AllowWarnDeny` (#11195) (camc314)
- 1210621 linter: Remove functions in favor of `Span::contains_inclusive` (#11133) (Ulrich Stark)
- 73b3f42 linter: Better names for parameters of diagnostic functions (#11129) (Ulrich Stark)
- 24fe879 linter: Remove unnecessary span construction (#11131) (Ulrich Stark)
- f079338 linter: Remove unnecessary macro (#11114) (overlookmotel)
- 9f3a14a linter: Cleanup diagnostic and docs for `eslint/no-console` (#11101) (Ulrich Stark)
- 18cdabf linter/prefer-todo: Remove unnecessary `codegen` (#11130) (shulaoda)
- bb8bde3 various: Update macros to use `expr` fragment specifier (#11113) (overlookmotel)

## [0.16.11] - 2025-05-16

- 4e5c73b span: [**BREAKING**] `SourceType::from_path(".js")` return js instead of jsx (#11038) (Boshen)

### Features

- eef93b4 linter: Add import/no-unassigned-import (#10970) (yefan)
- cc0112f linter: No-unused-vars add setting for `reportVarsOnlyUsedAsTypes` (#11009) (camc314)
- 17e49c3 linter: Implement configuration and checking loops for `eslint/no_constant_condition` (#10949) (Ulrich Stark)
- 21117ac linter: Implement react/forbid-elements (#10928) (Thomas BOCQUEZ)
- a064082 linter: Add import/consistent-type-specifier-style rule (#10858) (yefan)
- 4733b52 linter/no-extraneous-class: Add conditional fixer (#10798) (DonIsaac)

### Bug Fixes

- c52a9ba linter: Fix plugins inside overrides not being applied (#11057) (camc314)
- b12bd48 linter: Fix rule config not being correctly applied (#11055) (camc314)
- 9a368be linter: False negative in no-restriced-imports with `patterns` and side effects (#11027) (camc314)
- 8c2cfbc linter: False negative in no-restricted-imports (#11026) (camc314)
- 8956870 linter: False positive in no-unused-vars (#11002) (camc314)
- 33a60d2 linter: Skip eslint/no-redeclare when running on modules (#11004) (camc314)
- 39063ce linter: Reword diagnostic message for no-control-regex (#10993) (camc314)
- 9eedb58 linter: False positive with negative matches in no-restricted-imports (#10976) (camc314)
- 10e77d7 linter: Improve diagnostics for no-control-regex (#10959) (camc314)
- 82889ae linter/no-extraneous-class: Improve docs, reporting and code refactor (#10797) (DonIsaac)
- 11c34e7 linter/no-img-element: Improve diagnostic and docs (#10908) (DonIsaac)
- 126ae75 semantic: Distinguish class private elements (#11044) (magic-akari)
- 773d0de semantic: Correctly handle nested brackets in jsdoc parsing (#10922) (camc314)
- b215b6c semantic: Dont parse `@` as jsdoc tags inside `[`/`]` (#10919) (camc314)

### Documentation

- db6afb9 linter: Improve docs of no-debugger (#11033) (camc314)
- 16541de linter: Improve docs of default-param-last (#11032) (camc314)
- 2c2f3c4 linter: Improve docs of default-case-last (#11031) (camc314)
- 56bb9ce linter: Improve docs of array-callback-return (#11030) (camc314)
- 13dbcc6 linter: Correct docs for default config for no-redeclare (#10995) (camc314)
- a86cbb3 linter: Fix incorrect backticks of fenced code blocks (#10947) (Ulrich Stark)

### Refactor

- bb999a3 language_server: Avoid cloning linter by taking reference in LintService (#10907) (Ulrich Stark)
- d1b0c83 linter: Remove overrides index vec (#11058) (camc314)
- 7ad6cf8 linter: Store severity separately, remove `RuleWithSeverity` (#11051) (camchenry)
- e31c361 linter: Remove nested match statements in no-restricted-imports (#10975) (camc314)
- 6ad9d4f linter: Tidy `eslint/func-names` (#10923) (camc314)
- faf0a95 syntax: Rename `NameSpaceModule` to `NamespaceModule` (#10917) (Dunqing)

## [0.16.10] - 2025-05-09

- ad4fbf4 ast: [**BREAKING**] Simplify `RegExpPattern` (#10834) (overlookmotel)

### Features

- 4c62348 linter: Regex/no-useless-backreference (#10773) (camc314)
- d7ebdd7 linter: Add unicorn/no-unnecessary-slice-end rule (#10826) (yefan)

### Bug Fixes

- 7d09973 linter: False positive with `withResolvers` in prefer-await-to-then (#10896) (camc314)
- 9b94300 linter: Mark fixer as dangerous for erasing-op (#10868) (camc314)
- ae70cc1 linter: Add missing option to `no-shadow-restricted-names` (#10827) (camc314)
- b2c287f linter/no-unused-vars: Fixer cannot delete usused for in/of iterators (#10824) (DonIsaac)
- 5ce0a68 linter/no-unused-vars: Recognize parameters used in await/yield expressions within comma expressions (#10808) (magic-akari)

### Performance

- 96cca22 language_server: Use `simdutf8` when reading files from file system (#10814) (Sysix)

### Documentation

- efaadd3 linter: Fix a few incorrect backticks in `no_restricted_imports` (#10914) (Boshen)
- ccda8f0 linter: Improve no-plusplus docs (#10885) (Peter Cardenas)
- 5f15809 linter: Improve docs for jsdoc/require-property (#10705) (camc314)

### Refactor

- 3d47159 language_server: Use `IsolatedLintHandlerFileSystem` (#10830) (Sysix)
- 79819cc linter: Move around some config store logic (#10861) (camc314)
- 243c247 linter: Able to use custom file system in runtime (#10828) (Sysix)

### Testing

- 47b946d linter: Use `TesterFileSystem` for `Runtime`s filesystem (#10829) (Sysix)

## [0.16.9] - 2025-05-02

- a0a37e0 ast: [**BREAKING**] `AstBuilder` methods require an `Atom` with correct lifetime (#10735) (overlookmotel)

- 315143a codegen: [**BREAKING**] Remove useless `CodeGenerator` type alias (#10702) (Boshen)

### Features

- 63f02a8 linter: Add react/forward_ref_uses_ref (#10506) (x6eull)
- a3ada34 linter: Implement fixer for unicorn/prefer-number-properties (#10693) (camc314)
- e97a4e0 linter: Add fixer to unicorn/prefer-spread (#10691) (camc314)
- a69a0ee linter: Add eslint/block-scoped-var (#10237) (yefan)
- 387af3a linter: Report vars only used as types (#10664) (camc314)
- eac205f linter: Add unicorn/consistent-assert rule (#10653) (Shota Kitahara)
- 0e6a727 linter: Add autofixer for eslint/radix (#10652) (yefan)
- fb070c4 linter/no-extra-boolean-cast: Implement auto-fixer (#10682) (DonIsaac)
- 432cd77 linter/no-new-wrapper: Implement auto-fixer (#10680) (DonIsaac)

### Bug Fixes

- b38338a linter: Make require post message target origin a fixer a suggestion (#10754) (camc314)
- 48c542d linter: Skip linting vue <script> where `lang` is not js / ts (#10740) (Boshen)
- c9575f6 linter: Fix false positive in react/exhaustive deps (#10727) (camc314)
- d8d8f64 linter: Shorten span of promise/prefer-await-to-then (#10717) (camc314)
- a88e349 linter: Mark `isNan` and `isFinite` as dangerous fixes in `unicorn/prefer-number-properties` (#10706) (Sysix)
- f4ab05f linter: Panic in unicorn/no-useless-spread (#10715) (camc314)
- 06f1717 linter: False positive in no unused vars when importing value used as type (#10690) (camc314)
- 746b318 linter: False positive in typescript/explicit-function-return-type with `satisfies` (#10668) (camc314)
- cce1043 linter: False positive in typescript/explicit-function-return-type (#10667) (camc314)
- c89da93 linter: False positive in eslint/curly on windows (#10671) (camc314)
- 374e19e linter: False positive in react/jsx-curly-brace-presence (#10663) (camc314)
- e7c2b32 linter: Move `consistent-assert` to `pedantic` (#10665) (camc314)
- 344ef88 linter: False positive in `eslint/no-unused-vars` when calling inside sequence expression (#10646) (Ulrich Stark)
- 98bcd5f lsp: Incorrect quick fix offset in vue files (#10742) (camc314)

### Performance

- c753f75 transformer, linter: Use `format_compact_str!` (#10753) (overlookmotel)

### Refactor


## [0.16.8] - 2025-04-27

### Features

- 53394a7 linter: Add auto-fix for eslint/require-await (#10624) (yefan)
- 6908bc3 linter: Add autofix for react/self-closing-comp (#10512) (x6eull)
- e228840 parser: Fast forward lexer to EOF if errors are encountered (#10579) (Boshen)

### Bug Fixes

- 39adefe linter: Handle re-exporting of type correctly in `import/no-cycle` (#10606) (Ulrich Stark)
- e67901b linter: Incorrect fix for prefer start ends with (#10533) (camc314)
- 7c85ae7 linter/no-empty-function: Support 'allow' option (#10605) (Don Isaac)
- a9785e3 parser,linter: Consider typescript declarations for named exports (#10532) (Ulrich Stark)

### Testing

- 8a2b250 linter: Fix incorrect test fixture for prefer-each (#10587) (Boshen)

## [0.16.7] - 2025-04-21

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

### Features

- bb8a078 language_server: Use linter runtime (#10268) (Sysix)
- c94e6b8 linter: Allow `eqeqeq` to always be dangerously fixable (#10499) (camchenry)

### Bug Fixes

- 2fc083c linter: Incorrect fix for prefer start ends with (#10525) (camc314)
- 020d8f8 linter: Fix auto-fix issue for eslint/no-else-return (#10494) (yefan)
- f0c1eff linter: False positve in no-unused-vars (#10470) (camc314)
- d690060 linter: Fix the auto-fix issue of the eslint/no-plusplus rule (#10469) (yefan)
- 72d5074 linter: False positive in `eslint/no-redeclare` (#10402) (shulaoda)
- c1f5623 linter: Add check for plugin_name when applying LintFilterKind::Rule (#10339) (Ulrich Stark)
- 58ab8ff parser: Adjust class start position when decorators are involved (#10438) (Boshen)

### Performance

- 62178c2 linter: Replace `phf_set` with `array` for `DOM_PROPERTIES_NAMES` (#10501) (shulaoda)
- 9280707 linter: Replace `phf_set` with `array` for `DOM_ATTRIBUTES_TO_CAMEL` (#10500) (shulaoda)
- 0a4f9d9 linter: Replace `phf_set` with `array` for `ATTRIBUTE_TAGS_MAP` (#10498) (shulaoda)
- 09f7358 linter: Replace `phf_set` with `array` in `jsdoc/check-tag-names` (#10485) (shulaoda)
- da87390 linter: Replace `phf_set` with `array` in `jsx-a11y/autocomplete-valid` (#10484) (shulaoda)
- d4033bc linter: Replace `phf_set` with `array` in `globals.rs` (#10483) (shulaoda)
- 7e08618 linter: Replace `phf_set` with `array` in `unicorn/prefer-add-event-listener` (#10451) (dalaoshu)
- e2af873 linter: Replace `phf_set` with `array` in `unicorn/no-useless-undefined` (#10450) (dalaoshu)
- af635fb linter: Replace `phf_set` with `array` in `nextjs/no-unwanted-polyfillio` (#10452) (shulaoda)
- c0f0369 linter: Replace `phf_set` with `array` in `utils/vitest` (#10427) (shulaoda)
- 17c7bda linter: Replace `phf_set` with `array` in `unicorn/prefer-type-error` (#10426) (shulaoda)
- 5cde29b linter: Replace `phf_set` with `array` in `react/void-dom-elements-no-children` (#10425) (shulaoda)
- 7ef1e0d linter: Replace `phf_set` with `array` in `unicorn/new-for-builtins` (#10424) (shulaoda)
- 50fd839 linter: Replace `phf_set` with `array` in `utils/mod.rs` (#10405) (shulaoda)
- a7ac137 linter: Replace `phf_set` with `array` in `unicorn/prefer-set-has` (#10398) (shulaoda)

### Documentation

- 5d1dfb5 linter: Fix wording in the eqeqeq docs (#10401) (Connor Pearson)

### Refactor


## [0.16.6] - 2025-04-14

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Features

- d48e886 linter: Add `import/group-exports` rule (#10330) (yefan)

### Bug Fixes

- 04e2fd4 linter: Fix false positives for `no-control-regex` (#10345) (Cam McHenry)
- e000f60 linter: Make extended configs properly inherit plugins (#10174) (Sub)
- 81867c4 linter: Fix stack overflow in react/exhaustive deps (#10322) (camc314)

### Performance

- 1bb61c6 linter: Replace `phf_set` with `array` in `unicorn/prefer-native-coercion-functions` (#10384) (shulaoda)
- e1e7a19 linter: Replace `phf_set` with `array` in `unicorn/no-array-for-each` (#10377) (dalaoshu)
- 5f0e66c linter: Replace `phf_set` with `array` in `unicorn/prefer-spread` (#10376) (dalaoshu)
- 8d9559d linter: Replace `phf_set` with `array` in `react/jsx-key` (#10375) (dalaoshu)
- fbd4f92 linter: Replace `phf_set` with `array` in `utils::jest` (#10369) (shulaoda)
- 8d0eb33 linter: Replace `phf_set` with `array` in `utils::express` (#10370) (shulaoda)
- ba538ff linter: Use `binary_search` for arrays with more than `7` elements (#10357) (shulaoda)
- 283e4c7 linter: Replace `phf_set` with `array` in `react/exhaustive-deps` (#10337) (shulaoda)
- 8b8d708 linter: Replace `phf_set` with `array` in `nextjs/no-typos` (#10336) (shulaoda)
- 0fd93d6 linter: Replace `phf_set` with `array` in `utils::promise` (#10335) (shulaoda)
- 485ba19 linter: Replace `phf_set` with `array` in `jest/prefer-jest-mocked` (#10302) (shulaoda)
- 83931ec linter: Replace `phf_set` with `array` in `jsdoc/check-access` (#10303) (shulaoda)
- 651b56f linter: Replace `phf_set` with `array` in `jsdoc/empty-tags` (#10304) (shulaoda)
- 7ffb7aa linter: Replace `phf_set` with `array` in `jsdoc/require-returns` (#10305) (shulaoda)
- d7399c4 linter: Replace `phf_set` with `array` in `jsx-a11y/no-noninteractive-tabindex` (#10306) (shulaoda)
- afe663b linter: Replace `phf_set` with `array` in `jest/no-restricted-matchers` (#10297) (shulaoda)
- bd27959 linter: Replace `phf_set` with `array` in `eslint/array-callback-return` (#10296) (shulaoda)
- 1aa0d71 linter: Replace `phf_set` with `array` in `react/no-array-index-key` (#10294) (shulaoda)
- d9c4891 linter: Replace `phf_set` with `array` in `eslint/valid-typeof` (#10293) (shulaoda)

### Refactor

- 2e1ef4c linter: Extract common logic from `jsdoc/require-yields` and `jsdoc/require-returns` (#10383) (shulaoda)
- 9533d09 linter: Remove duplicate ARIA property lists (#10326) (camchenry)
- 67bd7aa linter: Add `AriaProperty` enum (#10325) (camchenry)
- 52ea978 linter: Update comments, improve tests, add variant All to LintFilterKind (#10259) (Ulrich Stark)

## [0.16.5] - 2025-04-07

### Features

- 2f6810a editor: Add named fixes for code actions (#10203) (camchenry)
- 794b180 linter: Add messages for complex fixes (#10279) (camchenry)
- bde73b5 linter: Add unicorn/no-accessor-recursion rule (#9971) (yefan)

### Bug Fixes

- 03ba760 linter: `jsdoc/require-param`: skip rule if any doc has `@type` tag (#10282) (Cam McHenry)
- 7c54ea1 linter: Rule `no-restricted-imports` allow combination of `paths` and `patterns` (#10224) (Sysix)
- 6174129 linter: Run `react/no-children-props` only when react framework is found (#10225) (Sysix)
- cc1267e linter: Fix `Display` impl for `ConfigBuilderError` (#10239) (overlookmotel)
- d691701 various: Unwrap `Result` of `write!` macro (#10228) (overlookmotel)

### Performance

- 5d40676 linter: Replace `phf_set` with `array` in `react/iframe-missing-sandbox` (#10281) (shulaoda)
- 0b2f22d linter: Replace `phf_set` with `array` in `globals` (#10274) (shulaoda)
- 3dfa876 linter: Replace `phf_set` with `array` in `eslint/no-import-assign` (#10271) (shulaoda)
- b34e876 linter: Avoid cloning filters by refactoring functions to take references (#10247) (Ulrich Stark)
- be048d2 linter: Remove `write!` macro where unnecessary (#10232) (overlookmotel)

### Documentation

- 3d4ed3e linter: Rule `eslint/eqeqeq` add "null" & "smart" options (#10258) (Jacob Smith)
- ec34ef3 rules/react: Adding missing code block ending (#10218) (Cannonbark)

### Styling

- fba11d2 linter: Remove unnecessary semi-colons (#10207) (camc314)

### Testing

- 72238fc linter: Ensure complex fixes have messages (#10280) (camchenry)

## [0.16.4] - 2025-04-01

- cd1f035 semantic: [**BREAKING**] Store symbol information as the first entry in `symbol_declarations` when it is redeclared (#10062) (Dunqing)

### Features

- 06e3db9 linter: Support `multipleFileExtensions` option for `unicorn/filename-case` (#10118) (shulaoda)
- dbe0e46 linter: Support `ignore` option for `unicorn/filename-case` (#10107) (shulaoda)
- 84a3490 semantic: Add `symbol_id` for declare function binding (#10078) (Dunqing)
- b804f7c semantic: Introduce `Redeclaraion` for `Scoping::symbol_declarations` (#10059) (Dunqing)

### Bug Fixes

- aba3654 linter: Span disable directive correctly on next line (#10141) (Ulrich Stark 🦀)

### Performance

- 566be59 linter: Replace `phf_set` with `array` in `eslint/func-names` (#10119) (shulaoda)
- 5e14fe9 linter: Inline `PRE_DEFINE_VAR` and use `array` format (#10079) (shulaoda)

### Refactor

- 09c0ac6 linter: Improve `unicorn/filename-case` (#10117) (shulaoda)
- d8e49a1 linter: Compute lintable extensions at compile time (#10090) (camchenry)
- b3ec235 linter: Use items of `oxc_ast::ast` module directly (#10100) (Ulrich Stark 🦀)
- 93e6c0b linter: Use `FormalParameter::has_modifier` to detect parameter properties (#10097) (Ulrich Stark 🦀)
- 5d829c2 semantic: Align handling of declaring symbol for function with TypeScript (#10086) (Dunqing)

## [0.16.3] - 2025-03-25

### Features

- 1b41cb3 linter: Add suggested fix to `unicorn/prefer-structured-clone` (#9994) (Ulrich Stark 🦀)
- 24cbe51 linter: Add suggested fixer to `typescript/no_unnecessary_parameter_property_assignment` and fix false positive (#9973) (Ulrich Stark 🦀)

### Bug Fixes

- 6c4b533 linter: False positive in `import/no-empty-named-blocks` (#9974) (shulaoda)
- ff13be6 linter: Correct fixer for spread in function arguments (#9972) (shulaoda)

### Refactor

- 0f1e0e8 linter: Gate rule docs behind feature (#10027) (camchenry)
- ad06194 linter: Add fixer for `typescript-eslint/no-non-null-asserted-optional-chain` (#9993) (camchenry)
- 402d8b7 linter: Improve `eslint/no-redeclare` (#9976) (shulaoda)
- be62d38 rust: Remove usages of `lazy_static` (#10007) (Boshen)
- 6432707 rust: Use `lazy-regex` (#10004) (Boshen)
- 0fa58d7 semantic: Always use `SymbolFlags::Function` for function id (#7479) (Dunqing)

## [0.16.2] - 2025-03-21

### Bug Fixes

- 2e8198e linter: Skip extending config files that look like named configs or not files (#9932) (camchenry)
- f649fb3 linter: Reclassify `unicorn/no-document-cookie` as restriction (#9933) (camchenry)

## [0.16.1] - 2025-03-20

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- 8e3d9be linter: Support `--report-unused-disable-directive` (#9223) (1zumii)
- 62c0132 linter: Add import/no-empty-named-blocks rule (#9710) (yefan)
- ea7e3f0 oxc_language_server: Support nested configs (#9739) (Nicholas Rayburn)

### Bug Fixes

- e9565c9 linter: Parse vue custom tag that starts with script (#9887) (Boshen)
- e6f7c74 linter: Import and fix tests for typescript::no_unnecessary_parameter_property_assignment (#9720) (Ulrich Stark)
- 4e39ba0 linter: Ignore modules with invalid source (#9801) (branchseer)
- 73fe248 linter/no_case_declarations: Fix span of error for `await using` (#9854) (overlookmotel)
- 2e023ab linter/react: `exhaustive-deps` report longest dependency (#9891) (overlookmotel)
- a113f7e parser: Error when `}` and `>` appear in `JSXText` (#9777) (Boshen)
- 3d4c5f3 semantic: Correctly visit `IfStmt` `test` when building cfg (#9864) (camc314)
- bc8bc08 semantic: Use correct scope flags for using declarations (#9751) (camc314)

### Performance

- d44ab9b linter: Return early in loop in `promise/no-nesting` (#9808) (therewillbecode)
- 2b65ed2 linter/no_unescaped_entities: Optimize string search and error generation (#9832) (overlookmotel)

### Documentation

- e408db8 linter: Improve docs for `unicorn/no-abusive-eslint-disable` (#9834) (shulaoda)
- 187fe39 linter: Add correctness examples to `typescript-prefer-as-const` (#9805) (therewillbecode)

### Refactor

- 723fdfb linter: Improve `jest-prefer-hooks-in-order` (#9892) (therewillbecode)
- 544a090 linter: Remove not implemented rule `constructor-super` (#9877) (Sysix)
- 8bdac56 linter: Improve `ast_util::is_method_call` (#9874) (shulaoda)
- a68e45c linter: Improve `unicorn/no-anonymous-default-export` (#9847) (dalaoshu)
- 6407200 linter: Improve `unicorn/new-for-builtins` (#9804) (dalaoshu)

## [0.16.0] - 2025-03-16

### Features

- 8dd6809 linter: Add `eslint/no-lonely-if` (#9660) (therewillbecode)
- c22276e oxc_linter: Sort rules by plugin and rule name when outputting resolved config as a JSON string (#9799) (Nicholas Rayburn)

### Bug Fixes

- 22f18ac linter: Improve `jsx-a11y/anchor-ambiguous-text` diagnostic message (#9789) (1zumii)
- 6c11740 linter: False positive in `unicorn/catch-error-name` (#9763) (shulaoda)

### Documentation

- ea6b6d9 linter: Improve docs for `eslint-valid-typeof` (#9797) (therewillbecode)
- 2c48fba linter: Fix typo in `oxc/bad-min-max-func` (#9791) (Flo)
- 210b876 linter: Improve `eslint-no-async-promise-executor` (#9778) (therewillbecode)
- f8628bc linter: Improve `eslint-no-class-assign` (#9779) (therewillbecode)
- faca7a8 linter: Improve `eslint-no-self-assign` (#9768) (therewillbecode)

### Refactor

- 227d203 linter: Improve `typescript-no-unnecessary-type-constraint` (#9798) (therewillbecode)
- 05fe2cd linter: Use `is_lexical` when checking for lexical decl (#9781) (camc314)
- fcdd810 linter: Remove if let nesting from `unicorn-no-date-clone` (#9767) (therewillbecode)
- 5a9e1b9 linter: Improve `typescript-no-misused-new` (#9766) (therewillbecode)
- 9df5565 linter: Improve `unicorn/filename-case` (#9762) (shulaoda)
- b0b1f18 linter: Remove if let nesting from `nextjs-no-async-client-component` (#9764) (therewillbecode)

## [0.15.15] - 2025-03-12

### Features

- 2ddad59 linter: Add unicorn/require-post-message-target-origin rule (#9684) (yefan)
- 474a57b linter: A new multi-file analysis runtime (#9383) (branchseer)

### Bug Fixes

- 6c0978b linter: No-single-promise-in-promise-methods: do not fix Promise.all when chained (#9697) (camchenry)
- ab594f1 linter: Turn oxc/no-redundant-constructor-init into typescript/no-unnecessary-parameter-property-assignment (#9618) (Uli)
- 91c009a linter: Add missing fail cases in `eslint-no-array-constructor` (#9659) (therewillbecode)
- 2810e5b linter: Add missing fail cases in eslint/no-self-compare (#9693) (therewillbecode)

### Performance

- bcbb468 linter: Use `OsStr` for faster path comparison and hashing (#9685) (Boshen)

### Refactor

- b9ab60b linter: Remove if let nesting from `bad-min-max-function` (#9722) (therewillbecode)
- 90b0227 linter: Remove if let nesting from `eslint-operator-assignment` (#9721) (therewillbecode)
- 5ef578e linter: Improve `jest/no-alias-methods` (#9694) (therewillbecode)

## [0.15.14] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

- 3c6f140 semantic: [**BREAKING**] Make `Scoping` methods consistent (#9628) (Boshen)

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 7331656 semantic: [**BREAKING**] Rename `SymbolTable` and `ScopeTree` methods (#9613) (Boshen)

### Features

- 0815fe8 linter: Add `promise/no-return-wrap` (#9537) (therewillbecode)
- ae7bb75 linter: Add react/jsx-filename-extension rule (#9474) (Cédric DIRAND)
- 50327f3 linter: Add import/exports-last (#9578) (yefan)
- 75e4b8d linter: Add import/no-anonymous-default-export rule (#9481) (yefan)
- 2f08b16 linter: Add `promise/prefer-catch` (#9488) (therewillbecode)
- 41f32ea linter: Allow adding more overrides via `extends` configs (#9475) (camchenry)
- fb7cf10 linter: Allowing `plugins` to be extended with `extends` (#9473) (camchenry)
- fc74849 linter: Inherit `rules` via the extended config files (#9308) (camchenry)
- 3fce826 linter: Add support for `extends` property in oxlintrc (#9217) (camchenry)
- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

### Bug Fixes

- 2d42569 linter: Rule `eslint/no-unsafe-optional-chaining` (#9632) (therewillbecode)
- a9d7df9 linter: False positive in `unicorn/escape-case` (#9638) (shulaoda)
- 3831819 linter: Fix example lint declaration and macro syntax (#9626) (Uli)
- 4ca62ab linter: Output right file line and column for `.vue`, `.astro` and `.svelte` files (#9484) (Sysix)
- 3105159 linter: Do not output number of rules with nested configs (#9476) (camchenry)
- 5ecda01 linter: Support nested extending (#9472) (camchenry)

### Documentation

- b7c61e9 linter: Improve docs for `eslint-guard-for-in` (#9658) (therewillbecode)
- 1cc43f7 linter: Improve the documentation of `eslint-no-console` (#9612) (therewillbecode)
- 608bb77 linter: Improve the docs and add test case for `typescript-no-extra-non-null-assertion` (#9609) (therewillbecode)
- 43add5d linter: Better docs for `typescript-no-non-null-asserted-nullish-coalescing` rule (#9610) (therewillbecode)
- bd90ce6 linter: Improve the docs and add test cases for `eslint-no-shadow-restricted-names` (#9597) (therewillbecode)
- a0c9f7c linter: Improve the documentation of `eslint-no-func-assign` (#9596) (therewillbecode)
- ec922e9 linter: Improve the documentation of `typescript-consistent-type-definitions` (#9575) (therewillbecode)
- 165c89d linter: Improve the documentation of `typescript-no-namespace` (#9545) (therewillbecode)

### Refactor

- c174600 linter: Improve `eslint/no-duplicate-imports` (#9627) (therewillbecode)
- 31ba425 linter: Improve `eslint/no-self-assign` (#9635) (therewillbecode)
- 03a40df linter: Access scoping from `ctx` directly (#9624) (Boshen)
- be5e5dc linter: Improve `unicorn/escape-case` (#9568) (shulaoda)
- b7f82fc linter: Improve `unicorn/error-message` (#9560) (shulaoda)
- 069ef2d linter: Improve `promise/no-nesting` (#9544) (therewillbecode)
- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

### Testing

- 934a387 linter: Remove test dependency on oxlint (#9513) (camchenry)

## [0.15.13] - 2025-03-04

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 7bb0121 linter: Add `react/no-namespace` (#9404) (Mikhail Baev)
- 0a7ca20 linter: Support allowable method diagnostic for eslint/no-console (#9454) (Boshen)
- d99bc51 linter: Add import/no-absolute-path rule (#9415) (yefan)
- 8c71590 linter: Add import/no-mutable-exports rule (#9434) (yefan)
- b65f8a5 linter: Add `promise/no-nesting` (#9345) (Tom)
- d38e6de linter: Add `eslint/no-spaced-func` (#9360) (Tom)
- 25392de linter: Add eslint/operator-assignment rule (#9208) (yefan)
- bf77167 linter: Add `curly` rule (#8123) (Yuichiro Yamashita)
- e3b6eeb linter: Add `unicorn/consistent-date-clone` (#9346) (Amol Bhave)
- 5ee2cab linter: Improve no_invalid_fetch_options (#9347) (Brooooooklyn)
- 4ad328b linter: Add oxc/no-redundant-constructor-init (#9299) (Ben Jones)
- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)

### Bug Fixes

- c4624a6 linter: Fix panic in `import/no-absolute-path` (#9500) (camc314)
- 4b0327b linter: False positive in `eslint/curly` (#9471) (Kevin Deng 三咲智子)
- 8804555 linter: Skip `no-absolute-path` tests on windows (#9435) (Cameron)
- 06fe76d linter: Rule `no-restricted-imports` use right span for exports statements (#9442) (Sysix)
- 3da3565 linter: Rule `unicorn/no-invalid-fetch-options` (#9416) (Tom)
- 85fbe8c linter: Rule `eslint/radix` look into globals config (#9407) (Sysix)
- 1113e3b linter: Rule `eslint/no-object-constructor` look into globals config (#9406) (Sysix)
- 0217ebb linter: Support more cases for no_redundant_constructor_init (#9364) (Ben Jones)

### Documentation

- 24850e7 linter: Add example of how configure rule (#9469) (Cédric DIRAND)
- acb1e2c linter: Add end code tag on rule doc (#9470) (Cédric DIRAND)
- d43b456 linter: Add full documentation to rule `no-restricted-imports` (#9440) (Sysix)

### Refactor

- ffec3f6 linter: Improve `eslint/no-new` (#9423) (Tom)
- 7c27f10 linter: Move rule `no-restricted-imports` to category `restriction` (#9443) (Sysix)
- 7e118a3 linter: Improve `typescript/explicit-function-return-type` (#9439) (Tom)
- 5318cf2 linter: Improve `eslint/no-spaced-func` (#9419) (shulaoda)
- 802f00e linter: Use the `javascript-globals` crate (#9412) (Boshen)
- bff83c9 linter: Improve `eslint/no-unsafe-negation` (#9362) (dalaoshu)
- 228bf99 linter: Improve `unicorn/empty-brace-spaces` (#9341) (dalaoshu)
- 55d071b linter: Improve `unicorn/consistent-existence-index-check` (#9339) (dalaoshu)
- 17acece linter: Improve `eslint/no-template-curly-in-string` (#9090) (dalaoshu)

## [0.15.12] - 2025-02-23

### Features

- 9bc3017 linter: Add support for nested config files (#9153) (camchenry)
- 914dd46 linter: Add eslint/max-depth (#9173) (ikkz)
- 0b08159 linter: Add eslint/max-lines-per-function (#9161) (ikkz)
- cc8dd48 linter: Add unicorn/no-invalid-fetch-options rule (#9212) (Mikhail Baev)
- af13b1b linter: Promote `eslint/no-eval` to `correctness` (#9231) (dalaoshu)
- 542bbd7 linter: Support `import-x` plugin name (#9074) (Sysix)
- d266c29 linter: Add eslint/max-nested-callbacks (#9172) (ikkz)
- 86795d0 linter: Implement grouped-accessor-pairs (#9065) (yefan)
- d70bad3 linter: Add eslint/no-unneeded-ternary rule (#9160) (Cédric DIRAND)
- 4bd86e6 linter: Add `fixer` for `unicorn/catch-error-name` (#9165) (dalaoshu)

### Bug Fixes

- 3031845 linter: Add option "allowTypeImports" for rule "no-restricted-imports" (#7894) (Alexander S.)

### Performance

- e2eb849 linter: Use concurrent hashmap `papaya` (#9218) (Boshen)

### Documentation

- 6c0f006 linter: Improve the documentation of eslint/no-useless-concat (#9179) (Tom)
- 3414824 oxc: Enable `clippy::too_long_first_doc_paragraph` (#9237) (Boshen)

### Refactor

- e32d6e2 allocator, linter: Shorten `serde` impls (#9254) (overlookmotel)
- b6fc0f6 linter: Improve `unicorn/consistent-function-scoping` (#9163) (dalaoshu)
- 63bb214 oxc: Apply `clippy::redundant_clone` (#9252) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.15.11] - 2025-02-16

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

- 9091387 ast: [**BREAKING**] Remove `TSType::TSQualifiedName` (#9051) (Dunqing)

### Features

- d93bf0e linter: Implement func-style rule (#8977) (yefan)
- a870526 linter: Add vitest/no-standalone-expect rule (#8986) (Tyler Earls)
- addaa8e linter: Support es2025 env (#8985) (Sysix)
- 5d508a4 linter: Support `env` and `globals` in `overrides` configuration (#8915) (Sysix)
- 41ad42a linter: Add init-declarations rule (#8909) (yefan)
- 125d610 minifier: Fold String::charAt / String::charCodeAt more precisely (#9082) (sapphi-red)

### Bug Fixes

- 8cbdf00 ecmascript: To_boolean for shadowed undefined (#9105) (sapphi-red)
- cfc71f9 ecmascript: To_string for shadowed undefined (#9103) (sapphi-red)
- b68e240 linter: Rule `unicorn/new-for-builtins` do not look into `globals` (#9146) (Sysix)
- 490c77d linter: Rule `no-constant-binary-expression` do not look into `globals` (#9145) (Sysix)
- b36734c linter: Rule `promise/avoid-new` do not look into `globals` (#9144) (Sysix)
- 091a5c1 linter: Rule `no-new-native-nonconstructor` do not look into globals (#9143) (Sysix)
- 1c1d2e6 linter: Rule `symbol-description` do not look into `globals` (#9142) (Sysix)
- 6d15153 linter: Rule `prefer-object-spreads` do not look into `globals` (#9141) (Sysix)
- 9214661 linter: Rule `valid-typeof` do not check for `globals` (#9140) (Sysix)
- 29141d6 linter: Rule `no-restricted-globals`: do not check for `globals` entries (#9139) (Sysix)
- 23d0d95 linter: Report `no-console` when the `globals.console` is `off` (#9138) (Sysix)
- 157e1a1 linter: False positive in `jest/no-conditional-expect` (#9053) (dalaoshu)
- 28b5990 linter: Rule `no-restricted-imports`: improve diagnostics (#8113) (Alexander S.)
- b191390 linter: `no-global-assign` look into `globals` config (#8963) (Sysix)
- 44d985b linter: Correct the `is_reference_to_global_variable` (#8920) (dalaoshu)

### Documentation

- 02cb45b linter: Add prettier-ignore where formatting ruins code (#8978) (camchenry)

### Refactor

- 97cc1c8 ast: Remove `TSLiteral::NullLiteral` (replaced by `TSNullKeyword`) (#9147) (Boshen)
- 9ca22f4 linter: Improve `jsx-a11y/heading-has-content` (#9089) (dalaoshu)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.15.10] - 2025-02-06

- b7ff7e1 span: [**BREAKING**] Export `ContentEq` trait from root of `oxc_span` crate (#8869) (overlookmotel)

### Features

- d6d80f7 linter: Add suggestion fixer for `eslint/no-iterator` (#8894) (dalaoshu)

### Bug Fixes

- baf3e4e linter: Correctly replace rule severity with duplicate rule name configurations (#8840) (dalaoshu)

### Performance

- 8a4988d linter: Use parallel iterator directly instead of iter and parallel bridge (#8831) (Cam McHenry)

### Refactor

- bb9d763 linter: Remove usage of `url` crate (#8833) (camchenry)
- 4fcf719 linter: Replace MIME guessing with extension check (#8832) (camchenry)

## [0.15.9] - 2025-02-01

### Features

- 1a41181 linter: Implement `eslint/prefer-object-spread` (#8216) (tbashiyy)
- adb8ebd linter: Implement no-useless-call rule (#8789) (keita hino)
- 3790933 linter: Add vitest/prefer-lowercase-title rule (#8152) (Tyler Earls)
- e8e6917 linter: Unicorn/switch-cases-braces support options (#8704) (1zumii)

### Bug Fixes

- 8ce21d1 linter: Can't disable `no-nested-ternary` rule anymore (#8600) (dalaoshu)
- 4f30a17 linter: Unicorn/switch-case-braces mangles code when applying fix (#8758) (Tyler Earls)
- 1de6f85 linter: No-lone-blocks erroring on block statements containing comments (#8720) (Tyler Earls)
- 77ef61a linter: Fix diagnostic spans for `oxc/no-async-await` (#8721) (camchenry)
- f15bdce linter: Catch `Promise` in `typescript/array-type` rule (#8702) (Rintaro Itokawa)

### Performance

- d318238 linter: Remove sorting of rules in cache (#8718) (camchenry)

### Documentation

- 57b7ca8 ast: Add documentation for all remaining JS AST methods (#8820) (Cam McHenry)

### Refactor

- c2fdfc4 linter: Correctly handle loose options for `eslint/eqeqeq` (#8798) (dalaoshu)
- 0aeaedd linter: Support loose options for `eslint/eqeqeq` (#8790) (dalaoshu)

## [0.15.8] - 2025-01-24

### Features

- dcaebe6 linter: Add "strict" option to `promise/prefer-await-to-then` rule (#8674) (Neil Fisher)

### Refactor

- a3dc4c3 crates: Clean up snapshot files (#8680) (Boshen)
- e66da9f isolated_declarations, linter, minifier, prettier, semantic, transformer: Remove unnecessary `ref` / `ref mut` syntax (#8643) (overlookmotel)
- 23b49a6 linter: Use `cow_to_ascii_lowercase` instead `cow_to_lowercase` (#8678) (Boshen)
- b8d9a51 span: Deal only in owned `Atom`s (#8641) (overlookmotel)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.15.7] - 2025-01-19

- 4ce6329 semantic: [**BREAKING**] Ensure program outlives semantic (#8455) (Valentinas Janeiko)

### Features

- 01ac773 linter: Support `ignoreTypeOfTestName` for `jest/valid-title` (#8589) (dalaoshu)
- 538b24a linter: Format the configuration documentation correctly (#8583) (Tapan Prakash)
- 7ab14cc linter: Add more Vitest compatible Jest rules (#8445) (Anson Heung)
- d178360 linter: Implement `eslint/prefer-promise-reject-errors` (#8254) (tbashiyy)

### Bug Fixes

- 855c839 codegen: Shorthand assignment target identifier consider mangled names (#8536) (Boshen)
- c15af02 linter: False positive in `eslint/no-lone-blocks` (#8587) (dalaoshu)
- 41f2070 linter: Rule `no-restricted-imports` support missing options (#8076) (Alexander S.)
- 869bc73 linter: Enhance `default_param_last` rule to handle optional parameters (#8563) (Tapan Prakash)
- c6260c2 linter: Support rest params for `prefer_promise_reject_errors` (#8468) (Yuichiro Yamashita)
- 2be1e82 linter/no-unused-vars: False positives when variable and type have same name (#8465) (Dunqing)

### Performance

- 250bbd1 linter/react-exhaustive-deps: Use stack of `AstType`s instead of `AstKind`s (#8522) (overlookmotel)

### Refactor

- 40f5165 linter: Improve `eslint/no-lone-blocks` (#8588) (dalaoshu)
- b4c87e2 linter: Move DiagnosticsReporters to oxlint (#8454) (Alexander S.)
- bf00f82 linter: Move rule `prefer-each` from vitest to jest + remapping (#8448) (Alexander S.)
- 8dd0013 linter/consistent-function-scoping: Remove `Visit::enter_node` usage (#8538) (overlookmotel)
- 30c0689 linter/no-map-spread: Remove `Visit::enter_node` usage (#8537) (overlookmotel)
- b5ed58e span: All methods take owned `Span` (#8297) (overlookmotel)

### Styling

- 3789d2f linter/react-exhaustive-deps: Fix indentation (#8520) (overlookmotel)

## [0.15.6] - 2025-01-13

### Features

- 457aa31 linter: Implement `no-lone-blocks` rule (#8145) (Yuichiro Yamashita)

### Refactor

- aea9551 ast: Simplify `get_identifier_reference` of `TSType` and `TSTypeName` (#8273) (Dunqing)
- 43ed3e1 linter: Add output formatter (#8436) (Alexander S.)
- b19d809 linter: Split `unicorn/prefer-spread` and `eslint/prefer-spread` into own rules (#8329) (Alexander S.)
- 3c534ae linter: Refactor `LintBuilder` to prep for nested configs (#8034) (camc314)
- 2f9fab9 linter: Remove remapping for plugin name in diagnostics (#8223) (Alexander S.)

### Testing

- b6c1546 linter: Use plugin name instead of category for finding rule (#8353) (Alexander S.)

## [0.15.5] - 2025-01-02

### Features

- 0e168b8 linter: Catch more cases in const-comparisons (#8215) (Cameron)
- bde44a3 linter: Add `statement_span` to `ModuleRecord/ImportEntry` (#8195) (Alexander S.)
- ccaa9f7 linter: Implement `eslint/new-cap`  (#8146) (Alexander S.)

### Bug Fixes

- 2b14a6f linter: Fix `ignorePattern` config for windows  (#8214) (Alexander S.)

## [0.15.4] - 2024-12-30

- ed75e42 semantic: [**BREAKING**] Make SymbolTable fields `pub(crate)` instead of `pub` (#7999) (Boshen)

### Features

- 47cea9a linter: Implement `eslint/no-extra-label` (#8181) (Anson Heung)
- ef76e28 linter: Implement `eslint/no-multi-assign` (#8158) (Anson Heung)
- 384858b linter: Implement `jsx-a11y/no-noninteractive-tabindex`  (#8167) (Tyler Earls)
- afc21a6 linter: Implement `eslint/vars-on-top` (#8157) (Yuichiro Yamashita)
- 65796c4 linter: Implement `eslint/prefer-rest-params` (#8155) (Yuichiro Yamashita)
- 5234d96 linter: Implement `eslint/no-nested-ternary` (#8150) (Yuichiro Yamashita)
- 1c5db72 linter: Implement eslint/no-labels (#8131) (Anson Heung)
- 0b04288 linter: Move `import/named` to nursery (#8068) (Boshen)

### Bug Fixes

- f3050d4 linter: Exclude svelte files from `no_unused_vars` rule (#8170) (Yuichiro Yamashita)
- faf7464 linter: Disable rule `react/rules-of-hook` by file extension (#8168) (Alexander S.)
- 1171e00 linter: Disable `react/rules-of-hooks` for vue and svelte files (#8165) (Alexander S.)
- 1b9a5ba linter: False positiver in private member expr in oxc/const-comparison (#8164) (camc314)
- 6bd9ddb linter: False positive in `typescript/ban-tslint-comment` (#8094) (dalaoshu)
- 10a1fd5 linter: Rule: `no-restricted-imports` support option `patterns` with `group` key (#8050) (Alexander S.)
- b3f38ae linter: Rule `no-restricted-imports`: support option `allowImportNames` (#8002) (Alexander S.)
- 340cc90 linter: Rule `no-restricted-imports`: fix option "importNames" (#7943) (Alexander S.)
- ec2128e linter: Fix line calculation for `eslint/max-lines` in diagnostics (#7962) (Dmitry Zakharov)
- 79af100 semantic: Reference flags not correctly resolved when after an export stmt (#8134) (camc314)

### Performance

- d8d2ec6 linter: Run rules which require typescript syntax only when source type is actually typescript (#8166) (Alexander S.)
- 2736657 semantic: Allocate `UnresolvedReferences` in allocator (#8046) (Boshen)

### Refactor

- 774babb linter: Read `exported_bindings_from_star_export` lazily (#8062) (Boshen)
- 547c102 linter: Use `RwLock<FxHashMap>` instead of `FxDashMap` for module record data (#8061) (Boshen)
- 952d7e4 linter: Rename `flat.rs` to `config.rs` (#8033) (camc314)
- 50848ed linter: Simplify `ConfigStore` to prep for nested configs (#8032) (camc314)
- b2a4a78 linter: Remove unused `with_rules` and `set_rule` methods (#8029) (camc314)
- 02f968d semantic: Change `Bindings` to a plain `FxHashMap` (#8019) (Boshen)

## [0.15.3] - 2024-12-17

### Features

- 25ddb35 linter: Add the import/no_named_default rule (#7902) (Guillaume Piedigrossi)
- ee26b44 linter: Enhance `get_element_type` to resolve more element types (#7885) (dalaoshu)

### Bug Fixes

- 6f41d92 linter: False positive in `unicorn/no-useless-spread` (#7940) (dalaoshu)
- 0867b40 linter: Fix configuration parser for `no-restricted-imports` (#7921) (Alexander S.)
- 9c9b73d linter: Fix incorrect fixer for `prefer-regexp-test` (#7898) (Cameron)
- 32935e6 linter: False positive in `jsx-a11y/label-has-associated-control` (#7881) (dalaoshu)
- 14c51ff semantic: Remove inherting `ScopeFlags::Modifier` from parent scope (#7932) (Dunqing)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)
- b99ee37 linter: Move rule "no-restricted-imports" to nursery (#7897) (Alexander S.)
- ff2a68f linter/yoda: Simplify code (#7941) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.15.2] - 2024-12-14

### Refactor

- e55ab24 linter: Use `Expression::is_super` (#7850) (overlookmotel)

## [0.15.1] - 2024-12-13

### Bug Fixes

- 2b187e5 linter: Fix configuration casing for `typescript/no_this_alias` (#7836) (Boshen)
- 06e6d38 linter: Fix unicorn/prefer-query-selector to use the correct replacement for getElementsByClassName (#7796) (Nicholas Rayburn)
- 7a83230 semantic: Missing reference when `export default` references a type alias binding (#7813) (Dunqing)

## [0.15.0] - 2024-12-10

- 39b9c5d linter: [**BREAKING**] Remove unmaintained security plugin (#7773) (Boshen)

### Features

- 065f7dc linter: Support `expectTypeOf`, `assert` and `assertType` in `vitest/expect-expect` (#7742) (Yuichiro Yamashita)
- 3d5f0a1 linter/no_restricted_imports: Add the no_restricted_imports rules (#7629) (Guillaume Piedigrossi)

### Bug Fixes

- ad27b20 linter: Only resolve esm files for import plugin (#7720) (Boshen)
- 5e6053f linter: False positive in `eslint/yoda` (#7719) (dalaoshu)

### Refactor

- c6a19aa linter: Remove unused `serde` features (#7738) (Boshen)
- b9a2b35 linter: Remove `aho-corasick` (#7718) (Boshen)

### Testing

- 62f0a22 linter: Port `react-jsx-uses-vars` rules to no_unused_vars (#7731) (Tyler Earls)
- 02f9903 linter: Add regression tests for `import/namespace` (#7723) (dalaoshu)

## [0.14.1] - 2024-12-06

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- fd0935c linter: Change `react/rules-of-hooks` category to `pedantic` (#7691) (Boshen)
- e64fd95 linter: Map `.js` to `.ts` when resolving with tsconfig.json (#7675) (Boshen)
- bd9d38a linter: Implement eslint:yoda (#7559) (tbashiyy)
- a14e76a linter: Report identical logical expressions in const-comparisons (#7630) (camc314)
- afe1e9b linter: Enhance `const-comparisons` for more cases (#7628) (camc314)
- 4eb87ea linter: RulesOfHooks from nursery to correctness (#7607) (Boshen)
- 275d625 linter: Output rules to json array (#7574) (camc314)
- b8dc333 syntax: Add `ExportEntry::is_type` (#7676) (Boshen)

### Bug Fixes

- 7cee065 linter: Panic in `yoda` (#7679) (camc314)
- 6ae178e linter: Ignore type references in `no-undef` (#7670) (Boshen)
- fcc2546 linter: Move `no-unused-expressions` from TS to eslint (#7624) (camc314)
- 29db060 linter: Detect typescript eslint alias rules (#7622) (Alexander S.)
- e824501 linter: False positive in exhaustive-deps (#7626) (camc314)
- 8a68ef4 linter: Update reporting spans for exhaustive-deps (#7625) (camc314)
- 543df6e linter: Fix false positives in exhaustive-deps (#7615) (camc314)
- e80214c linter: Fix false positives in rules-of-hooks (#7606) (camc314)
- 3dc46a8 linter: No-unused-expressions false positive with arrow fn expressions (#7585) (Cameron)
- 810671a linter: Detect vitest jest alias rules (#7567) (Alexander S.)
- 4e3044e linter: Rules-of-hooks fix false positive with default export (#7570) (camc314)

### Documentation

- f029090 linter: Update rule documentation (#7684) (camc314)
- 4e489bd linter: Update rule documentation (#7681) (camc314)
- 56fe5f8 linter: Update rule documentation (#7680) (Cameron)

### Refactor

- a0973dc linter: Use `BigIntLiteral::raw` field (#7660) (overlookmotel)
- 3711a8e linter: Rename `is_same_reference` to `is_same_expression` (#7654) (camc314)
- b445654 linter: Use `get_inner_expression` in `const-comparisons` (#7627) (camc314)
- f0e7acc syntax: Change `ModuleRecord::not_esm` to `has_module_syntax` (#7579) (Boshen)
- 18519de syntax: Remove `ModuleRecord::export_default` (#7578) (Boshen)
- d476660 syntax: Remove `ModuleRecord::exported_bindings_duplicated` because it is a syntax error (#7577) (Boshen)
- 17663f5 syntax: Remove `ModuleRecord::export_default_duplicated` because it is a syntax error (#7576) (Boshen)
- 79014ff syntax: Clean up `ModuleRecord` (#7568) (Boshen)

### Testing

- be9863a linter: Add more tests fo rules-of-hooks (#7683) (camc314)
- 6dd71c6 linter: Port eslint tests to no-unused-expressions (#7611) (camc314)

## [0.14.0] - 2024-12-01

- c2ced15 parser,linter: [**BREAKING**] Use a different `ModuleRecord` for linter (#7554) (Boshen)

- 0be5233 semantic: [**BREAKING**] Remove `ModuleRecord` from `Semantic` (#7548) (Boshen)

- 8a788b8 parser: [**BREAKING**] Build `ModuleRecord` directly in parser (#7546) (Boshen)

### Features

- 32f860d linter: Add support for ignorePatterns property within config file (#7092) (Nicholas Rayburn)
- 053bc08 linter: Implement typescript/no-unused-expressions (#7498) (camc314)
- 60b28fc linter: Implement typescript/consistent-generic-constructors (#7497) (camc314)
- bd0693b linter: Allow lint rules with the same name (#7496) (camc314)
- 2ac9f96 linter: Typescript/no-inferrable-types (#7438) (camc314)
- 8d89fdc linter: Add eslint/prefer-spread (#7112) (tbashiyy)

### Bug Fixes

- 123b5b7 linter: False positive in `typescript/consistent-type-definitions` (#7560) (dalaoshu)
- cc078d6 linter: Add missing error message prefix to `eslint/no-const-assign` (Boshen)
- 17c0dd8 linter: Fix `jsx_no_script_url` doc failed to build (Boshen)

### Performance

- 6cc7a48 linter: Use `OsString` for module cache hash (#7558) (Boshen)
- 6655345 linter: Use `FxDashMap` for module cache (#7522) (overlookmotel)

### Documentation

- a6b0100 linter: Fix config example headings (#7562) (Boshen)

### Refactor

- 0f3f67a linter: Add capability of adding semantic data to module record (#7561) (Boshen)
- 8392177 linter: Clean up the runtime after the module record change (#7557) (Boshen)
- 823353a linter: Clean up APIs for `ModuleRecord` (#7556) (Boshen)
- f847d0f linter: Call `str::ends_with` with array not slice (#7526) (overlookmotel)
- 2077ff9 linter: Remove `once_cell` (#7510) (Boshen)
- 169b8bf linter, syntax: Introduce type alias `FxDashMap` (#7520) (overlookmotel)

## [0.13.2] - 2024-11-26

### Features

- 7236d14 eslint/jsx_a11y: Implement anchor_ambiguous_text (#5729) (Jelle van der Waa)
- 79ab8cc lint-unicorn: Add rule prefer set has (#7075) (jordan boyer)
- 87c893f linter: Add the eslint/no_duplicate_imports rule (#7309) (Guillaume Piedigrossi)
- 0b9da38 linter: Implement `unicorn/prefer-negative-index` (#6920) (Brian Liu)
- f0643c4 linter: Implement `jsx-no-script-url` (#6995) (Radu Baston)
- 00060ca linter: Implement eslint/no-object-constructor (#7345) (Naoya Yoshizawa)

### Bug Fixes

- db6558f linter: False positive in `eslint/prefer-object-has-own` (#7463) (dalaoshu)

### Refactor

- d7d0735 semantic: Remove `SymbolFlags::TypeLiteral` (#7415) (Dunqing)

## [0.13.1] - 2024-11-23

- 6f0fe38 semantic: [**BREAKING**] Correct all `ReferenceFlags::Write` according to the spec (#7388) (Dunqing)

### Features

- 4ad26b9 linter: Add `no-promise-in-callback` (#7307) (no-yan)

### Bug Fixes

- 8507464 linter: Hanging when source has syntax/is flow (#7432) (Cameron)
- e88cf1b linter: Make `overrides` globs relative to config path (#7407) (camchenry)
- 9002e97 linter: Add proper support for findIndex and findLastIndex for `unicorn/prefer-array-some` (#7405) (Dmitry Zakharov)

### Documentation

- 6730e3e linter: Add more examples for `unicorn/prefer-array-some` (#7411) (Dmitry Zakharov)

### Refactor

- 6c0d31b linter: Remove useless `const` declaration (#7430) (Song Gao)
- c8adc46 linter/no-unused-vars: Improve implementation to remove using SymbolFlags::Export (#7412) (Dunqing)
- c90537f linter/only-used-in-recursion: Improve implementation to remove using SymbolFlags::Export (#7413) (Dunqing)

## [0.13.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 878189c parser,linter: [**BREAKING**] Add `ParserReturn::is_flow_language`; linter ignore flow error (#7373) (Boshen)

- 7bf970a linter: [**BREAKING**] Remove tree_shaking plugin (#7372) (Boshen)

### Features

- 7f8747d linter: Implement `react/no-array-index-key` (#6960) (BitterGourd)
- be152c0 linter: Add `typescript/no-require-imports` rule (#7315) (Dmitry Zakharov)
- 849489e linter: Add suggestion for no-console (#4312) (DonIsaac)
- 8cebdc8 linter: Allow appending plugins in override (#7379) (camchenry)
- 8cfea3c oxc_cfg: Add implicit return instruction (#5568) (IWANABETHATGUY)
- e6922df parser: Fix incorrect AST for `x?.f<T>()` (#7387) (Boshen)

### Bug Fixes

- e91c287 linter: Fix panic in react/no-array-index-key (#7395) (Boshen)
- a32f5a7 linter/no-array-index-key: Compile error due to it uses a renamed API (#7391) (Dunqing)
- 666b6c1 parser: Add missing `ChainExpression` in optional `TSInstantiationExpression` (#7371) (Boshen)

### Documentation

- df143ca linter: Add docs for config settings (#4827) (DonIsaac)
- ad44cfa linter: Import/first options (#7381) (Zak)

### Refactor

- c34d649 linter: Use `scope_id` etc methods (#7394) (overlookmotel)

## [0.12.0] - 2024-11-20

- 20d9080 linter: [**BREAKING**] Override plugins array when passed in config file (#7303) (camchenry)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- 1d9f528 linter: Implement `unicorn/prefer-string-raw` lint rule (#7335) (Ryan Walker)
- d445e0f linter: Implement `unicorn/consistent-existence-index-check`  (#7262) (Ryan Walker)
- 01ddf37 linter: Add `allowReject` option to `no-useless-promise-resolve-reject` (#7274) (no-yan)
- 755a31b linter: Support bind function case for compatibility with `promise/no-return-wrap` (#7232) (no-yan)
- 428770e linter: Add `import/no-namespace` rule (#7229) (Dmitry Zakharov)
- 9c91151 linter: Implement typescript/no-empty-object-type (#6977) (Orenbek)
- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)
- 3dcac1a linter: React/exhaustive-deps (#7151) (camc314)

### Bug Fixes

- bc0e72c linter: Handle user variables correctly for import/no_commonjs (#7316) (Dmitry Zakharov)
- bf839c1 linter: False positive in `jest/expect-expect` (#7341) (dalaoshu)
- ff2a1d4 linter: Move `exhaustive-deps` to `react` (#7251) (camc314)
- df5c535 linter: Revert unmatched rule error (#7257) (Cameron A McHenry)
- c4ed230 linter: Fix false positive in eslint/no-cond-assign (#7241) (camc314)
- ef847da linter: False positive in `jsx-a11y/iframe-has-title` (#7253) (dalaoshu)
- 62b6327 linter: React/exhaustive-deps update span for unknown deps diagnostic (#7249) (camc314)

### Refactor

- c6a4868 linter: Temporarily remove unknown rules checking (#7260) (camchenry)

## [0.11.1] - 2024-11-09

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- d1d1874 ast: [**BREAKING**] Change `comment.span` to real position that contain `//` and `/*` (#7154) (Boshen)

- 843bce4 ast: [**BREAKING**] `IdentifierReference::reference_id` return `ReferenceId` (#7126) (overlookmotel)

### Features

- 1fcd709 linter: Add jsx support for only-used-in-recursion (#7120) (no-yan)
- 4d577cf linter: Add `import/first` rule (#7180) (Dmitry Zakharov)
- 9b8973f linter: Add `import/unambiguous` rule (#7187) (Dmitry Zakharov)
- 5ab1ff6 linter: Implement @typescript-eslint/no-unsafe-function-type (#6989) (Orenbek)

### Bug Fixes

- b73cfd9 linter: Fix `is_method_call` with parentheses and chain expression (#7095) (tbashiyy)

### Refactor

- 8c0a362 linter: Use `ctx.source_range(comment.content_span())` API (#7155) (Boshen)
- c5485ae semantic: Add `ancestor_kinds` iterator function (#7217) (camchenry)
- abf1602 semantic: Rename `iter_parents` to `ancestors` (#7216) (camchenry)
- 42171eb semantic: Rename `ancestors` to `ancestor_ids` (#7215) (camchenry)

## [0.11.0] - 2024-11-03

- 1f2a6c6 linter: [**BREAKING**] Report unmatched rules with error exit code (#7027) (camchenry)

- 9fd9f4f linter: [**BREAKING**] Sync sindresorhus/globals; removed Object.prototype properties from builtin and es* globals (#6991) (Boshen)

- 9a6a2f9 semantic: [**BREAKING**] Remove `SymbolTable::get_symbol_id_from_span` API (#6955) (Boshen)

### Features

- 2184588 linter: Do not bail for unmatched rules yet (#7093) (Boshen)
- a6fcd81 linter: Add `import/no-commonjs` rule (#6978) (Dmitry Zakharov)
- 1691cab linter: Support user-configurable secrets for `oxc-security/api-keys` (#5938) (DonIsaac)
- 610621c linter: Implement `react/style-prop-object` (#6342) (Albert Kaaman)
- 1e2f012 linter: Add `oxc/no-map-spread` (#6751) (DonIsaac)
- 1c66473 linter: Implement `eslint/prefer-object-has-own` (#6905) (tomoya yanagibashi)

### Bug Fixes

- 79bf74a linter: Check is_reference_to_global_variable in `no-array-constructor` (#7067) (Naoya Yoshizawa)
- 147e2e4 linter: Allow replacing rule when none are enabled yet (#7014) (camchenry)
- 7aa496a linter: Remove unsafe fixer of `no-useless-spread` (#6655) (dalaoshu)
- f5a7134 linter/no-unused-vars: False positive for discarded reads within sequences (#6907) (DonIsaac)

### Documentation

- 4551baa linter: Document `rules` (#6983) (Boshen)

### Refactor

- 8f1460e linter: Move `LintPlugins` from `LintOptions` to `LintConfig` (#6932) (DonIsaac)

### Testing

- c35d3f2 linter: Improve test failure output (#6975) (camchenry)

## [0.10.3] - 2024-10-26

- 90c786c regular_expression: [**BREAKING**] Support ES2025 Duplicated named capture groups (#6847) (leaysgur)

- 8032813 regular_expression: [**BREAKING**] Migrate to new regexp parser API (#6741) (leaysgur)

### Features

- a73c5af linter: Add fixer for `jsx-a11y/no-access-key` rule (#6781) (Tapan Prakash)
- 2aa763c linter: Warn unmatched rule names (#6782) (Tapan Prakash)
- 0acca58 linter: Support `--print-config all` to print config file for project (#6579) (mysteryven)

### Bug Fixes

- f49b3e2 linter: `react/iframe-missing-sandbox` ignores vanilla JS APIs (#6872) (DonIsaac)
- 54a5032 linter: Correct false positive in `no-duplicates` (#6748) (dalaoshu)
- a47c70e minifier: Fix remaining runtime bugs (#6855) (Boshen)

### Documentation

- 3923e63 linter: Add schema to config examples (#6838) (Dmitry Zakharov)

### Refactor

- a148023 linter: Dereference IDs as soon as possible (#6821) (overlookmotel)
- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.10.2] - 2024-10-22

- 1248557 ast: [**BREAKING**] Remove `AstKind::FinallyClause` (#6744) (Boshen)

- 202c7f6 ast: [**BREAKING**] Remove `AstKind::ExpressionArrayElement` and `AstKind::ClassHeritage` (#6740) (Boshen)

### Features

- dbe1972 linter: Import/no-cycle should turn on ignore_types by default (#6761) (Boshen)
- 619d06f linter: Fix suggestion for `eslint:no_empty_static_block` rule (#6732) (Tapan Prakash)

### Bug Fixes


### Performance

- 8387bac linter: Apply small file optimization, up to 30% faster (#6600) (camchenry)

### Refactor

- b884577 linter: All ast_util functions take Semantic (#6753) (DonIsaac)
- 744aa74 linter: Impl `Deref<Target = Semantic>` for `LintContext` (#6752) (DonIsaac)

## [0.10.1] - 2024-10-21

### Features

- af25752 linter: Add `unicorn/prefer-math-min-max` (#6621) (Brian Liu)
- 5095f02 linter: Added fixer for duplicate prefix in valid title jest rule (#6699) (Tapan Prakash)
- e9976d4 linter: Add title whitespace fixer for jest valid title rule (#6669) (Tapan Prakash)
- 45f02d5 linter: Add `unicorn/consistent-empty-array-spread` (#6695) (Brian Liu)
- 01a35bb linter/eslint: Show ignore patterns in `eslint/no-unused-vars` diagnostic messages (#6696) (DonIsaac)

### Bug Fixes

- ce25c45 linter: Panic in `disable-directives` (#6677) (dalaoshu)
- a5de230 linter/import: `import/no-duplicates` handles namespace imports correctly (#6694) (DonIsaac)
- b0b6ac7 linter/no-cond-assign: False positive when assignment is in body statement (#6665) (camchenry)

### Performance

- 6a76ea8 linter/no-unused-vars: Use default IgnorePattern when /^_/ is provided as a pattern (#6697) (DonIsaac)

### Refactor

- d6609e9 linter: Use `run_on_jest_node` for existing lint rules (#6722) (camchenry)
- 97195ec linter: Add `run_on_jest_node` to run rules on only jest nodes (#6721) (camchenry)
- 155fe7e linter: Allow `Semantic` to be passed for collecting Jest nodes (#6720) (camchenry)
- ad8f281 linter: Use iter for collecting jest nodes (#6719) (camchenry)
- dc19a8f linter: Use iterator for collecting jest imports (#6718) (camchenry)
- 29c1447 linter: `jest/valid-title` fixer to use `Span::shrink` method (#6703) (Tapan Prakash)
- 2eb984a linter: Add missing `should_run` implementations (#6666) (camchenry)
- 23f88b3 linter/import: Better diagnostic messages for `import/no-duplicates` (#6693) (DonIsaac)

## [0.10.0] - 2024-10-18

- 782f0a7 codegen: [**BREAKING**] Rename `print_char` method to `print_ascii_byte` (#6512) (overlookmotel)

- 7645e5c codegen: [**BREAKING**] Remove CommentOptions API (#6451) (Boshen)

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

- 80266d8 linter: [**BREAKING**] Support plugins in oxlint config files (#6088) (DonIsaac)

### Features

- 6f22538 ecmascript: Add `ToBoolean`, `ToNumber`, `ToString` (#6502) (Boshen)
- 1e7fab3 linter: Implement `no-callback-in-promise` (#6157) (dalaoshu)
- c56343d linter: Promote `no_unsafe_optional_chaining` to correctness (#6491) (Boshen)
- 454874a linter: Implement `react/iframe-missing-sandbox` (#6383) (Radu Baston)
- c8174e2 linter: Add suggestions for `no-plusplus` (#6376) (camchenry)
- 6e3224d linter: Configure by category in config files (#6120) (DonIsaac)
- c5e66e1 linter/no-unused-vars: Report own type references within class, interface, and type alias declarations (#6557) (DonIsaac)
- 8c78f97 linter/node: Implement no-new-require (#6165) (Jelle van der Waa)

### Bug Fixes

- e340424 linter: Support import type with namespaced import in `import/no-duplicates` (#6650) (Dmitry Zakharov)
- a668397 linter: Panic in `no-else-return` (#6648) (dalaoshu)
- 41dc8e3 linter: Stack overflow in `oxc/no-async-endpoint-handlers` (#6614) (DonIsaac)
- d07a9b0 linter: Panic in `no-zero-fractions` (#6607) (dalaoshu)
- d6a0d2e linter: Fix file name checking behavior of `unicorn/filename-case` (#6463) (camchenry)
- 0784e74 linter: Error fixer of `switch-case-braces` (#6474) (dalaoshu)
- e811812 linter: Error diagnostic message based on parameter length of valid-expect (#6455) (dalaoshu)
- f71c91e linter: Move `eslint/sort-keys` to `style` category (#6377) (DonIsaac)
- 2b86de9 linter/no-control-regex: False negative for flags in template literals (#6531) (DonIsaac)
- 685a590 linter/no-control-regex: Better diagnostic messages (#6530) (DonIsaac)
- 6d5a9f2 linter/no-control-regex: Allow capture group references (#6529) (DonIsaac)
- ba53bc9 linter/no-unused-vars: False positives in TS type assertions (#6397) (DonIsaac)
- d3e59c6 linter/no-unused-vars: False positive in some default export cases (#6395) (DonIsaac)
- e08f956 linter/no-unused-vars: False positive for functions and classes in arrays (#6394) (DonIsaac)
- b9d7c5f no-unused-vars: Consider functions within conditional expressions usable (#6553) (Brian Donovan)

### Performance

- 0cbd4d0 linter: Avoid megamorphism in `RuleFixer` methods (#6606) (DonIsaac)
- 725f9f6 linter: Get fewer parent nodes in `unicorn/prefer-dom-node-text-content` (#6467) (camchenry)
- c00f669 linter: Use NonZeroUsize for pending module cache entries (#6439) (DonIsaac)
- a1a2721 linter: Replace `ToString::to_string` with `CompactStr` in remaining rules (#6407) (camchenry)
- c5c69d6 linter: Use `CompactStr` in `valid-title` (#6406) (camchenry)
- d66e826 linter: Use `CompactStr` in `prefer-lowercase-title` (#6405) (camchenry)
- 889400c linter: Use `CompactStr` for `get_node_name` in Jest rules (#6403) (camchenry)
- 9906849 linter: Use `CompactStr` in `no-large-snapshots` (#6402) (camchenry)
- c382ec4 linter: Use `CompactStr` in `no-hooks` (#6401) (camchenry)
- 24a5d9b linter: Use `CompactStr` in `expect-expect` (#6400) (camchenry)
- 71dbdad linter: Use `CompactStr` in `no-console` (#6399) (camchenry)
- f5f00a1 linter: Use `CompactStr` in `no-bitwise` (#6398) (camchenry)
- 62afaa9 linter/jsx-no-comment-textnodes: Remove regex for checking comment patterns (#6534) (camchenry)
- b3d0cce linter/no-unescaped-entities: Add fast path to check if char should be replaced (#6594) (camchenry)
- ee73f56 linter/no-unused-vars: Do not construct `Regex` for default ignore pattern (#6590) (camchenry)
- 77ddab8 linter/numeric-separators-style: Replace regex with number parser (#6546) (camchenry)
- 8f47cd0 linter/react: Remove regex patterns in `no-unknown-property` (#6536) (camchenry)

### Documentation

- 557f941 linter: Add docs to no-unused-vars and Tester (#6558) (DonIsaac)

### Refactor

- ecce5c5 linter: Improve recursive argument handling and diagnostics creation (#6513) (no-yan)
- f960e9e linter: Add suggested file names for `unicorn/filename-case` (#6465) (camchenry)
- 7240ee2 linter: Make advertised fix kinds consistent (#6461) (Alexander S.)
- b48c368 linter: `no_global_assign` rule: reduce name lookups (#6460) (overlookmotel)
- 2566ce7 linter: Remove OxlintOptions (#6098) (DonIsaac)
- 002078a linter: Make Runtime's members private (#6440) (DonIsaac)
- 6a0a533 linter: Move module cache logic out of Runtime (#6438) (DonIsaac)
- c18c6e9 linter: Split service code into separate modules (#6437) (DonIsaac)
- 5ea9ef7 linter: Improve labels and help message for `eslint/no-useless-constructor` (#6389) (DonIsaac)
- 2c32dac linter/no-control-regex: Remove duplicate code (#6527) (DonIsaac)
- 435a89c oxc: Remove useless `allocator.alloc(program)` calls (#6571) (Boshen)
- f70e93b oxc: Ban index methods on std::str::Chars (#6075) (dalaoshu)

### Testing

- a6cae98 linter: Make sure all auto-fixing rules have fixer test (#6378) (DonIsaac)
- 06b09b2 linter/no-unused-vars: Enable now-passing tests (#6556) (DonIsaac)
- badd11c linter/no-unused-vars: Ignored catch parameters (#6555) (DonIsaac)
- 84aa2a2 linter/no-useless-constructor: Add cases for initializers in subclass constructors (#6390) (DonIsaac)

## [0.9.10] - 2024-10-07

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

### Features

- 376cc09 linter: Implement `no-throw-literal` (#6144) (dalaoshu)
- 5957214 linter: Allow fixing in files with source offsets (#6197) (camchenry)
- a089e19 linter: Eslint/no-else-return (#4305) (yoho)
- 183739f linter: Implement prefer-await-to-callbacks (#6153) (dalaoshu)
- ae539af linter: Implement no-return-assign (#6108) (Radu Baston)

### Bug Fixes

- 9e9808b linter: Fix regression when parsing ts in vue files (#6336) (Boshen)
- 93c6db6 linter: Improve docs and diagnostics message for no-else-return (#6327) (DonIsaac)
- e0a3378 linter: Correct false positive in `unicorn/prefer-string-replace-all` (#6263) (H11)
- ea28ee9 linter: Improve the fixer of `prefer-namespace-keyword` (#6230) (dalaoshu)
- f6a3450 linter: Get correct source offsets for astro files (#6196) (camchenry)
- be0030c linter: Allow whitespace control characters in `no-control-regex` (#6140) (camchenry)
- e7e8ead linter: False positive in `no-return-assign` (#6128) (DonIsaac)

### Performance

- ac0a82a linter: Reuse allocator when there are multiple source texts (#6337) (Boshen)
- 50a0029 linter: Do not concat vec in `no-useless-length-check` (#6276) (camchenry)

### Documentation

- 7ca70dd linter: Add docs for `ContextHost` and `LintContext` (#6272) (camchenry)
- a949ecb linter: Improve docs for `eslint/getter-return` (#6229) (DonIsaac)
- 14ba263 linter: Improve docs for `eslint-plugin-import` rules (#6131) (dalaoshu)

### Refactor

- 642725c linter: Rename vars from `ast_node_id` to `node_id` (#6305) (overlookmotel)
- 8413175 linter: Move shared function from utils to rule (#6127) (dalaoshu)
- ba9c372 linter: Make jest/vitest rule mapping more clear (#6273) (camchenry)
- 82b8f21 linter: Add schemars and serde traits to AllowWarnDeny and RuleCategories (#6119) (DonIsaac)
- ea908f7 linter: Consolidate file loading logic (#6130) (DonIsaac)
- db751f0 linter: Use regexp AST visitor in `no-control-regex` (#6129) (camchenry)
- 3aa7e42 linter: Use RegExp AST visitor for `no-hex-escape` (#6117) (camchenry)
- 9d5b44a linter: Use regex visitor in `no-regex-spaces` (#6063) (camchenry)
- 0d44cf7 linter: Use regex visitor in `no-useless-escape` (#6062) (camchenry)
- eeb8873 linter: Use regex visitor in `no-empty-character-class` (#6058) (camchenry)

### Testing

- d883562 linter: Invalid `eslint/no-unused-vars` options (#6228) (DonIsaac)

## [0.9.9] - 2024-09-27

### Bug Fixes

- bd8f786 linter: Rule and generic filters do not re-configure existing rules (#6087) (DonIsaac)
- c5cdb4c linter: Disable all rules in a plugin when that plugin gets turned off (#6086) (DonIsaac)
- 6c855af linter: Only write fix results if source code has changed (#6096) (DonIsaac)
- 8759528 linter: Category filters not re-configuring already-enabled rules (#6085) (DonIsaac)
- c2616f7 linter: Fix panic in fixer for `oxc/only-used-in-recursion` (#6070) (camc314)
- 3da3845 linter: Malformed snippets in `eslint/for-direction` docs (#6060) (DonIsaac)
- c047d42 linter: `no-useless-escape`: do not crash on backslash character (#6048) (camchenry)
- 6f76ebe linter: Ignore invalid or partial disable directives (#6045) (camchenry)
- 09a24cd linter: Fix false positives for generics in `no-unexpected-multiline` (#6039) (camchenry)
- d05fd20 linter: Newline in type parameters causing false positive in `no-unexpected-multiline` (#6031) (DonIsaac)

### Performance

- f8464a3 linter: `no-magic-numbers` remove redudant checks in `is_array_index` (#6033) (Alexander S.)
- c16ae60 linter: `jest/prefer-hooks-in-order`: rewrite rule to allocate less and iterate fewer times (#6030) (camchenry)

### Documentation

- a4fdf1b linter: Improve docs for promise rules (#6051) (dalaoshu)
- 21cdb78 linter: Fix incorrect "bad" example in `only-used-in-recursion` (#6029) (Boshen)

### Refactor

- 1f92d61 linter: `jest/prefer-hooks-in-order`: improve diagnostic messages (#6036) (camchenry)

### Testing

- 55949eb linter: Add `OxlintRules::override_rules` tests (#6081) (DonIsaac)
- 1a6923a linter: Add filter parsing test cases (#6080) (DonIsaac)
- 58d333a linter: Add more test cases for disable directives (#6047) (camchenry)

## [0.9.8] - 2024-09-24

### Bug Fixes

- e3c8a12 linter: Fix panic in sort-keys (#6017) (Boshen)
- 4771492 linter: Fix `import/no_cycle` with `ignoreTypes` (#5995) (Boshen)

### Performance

- 5ae3f36 linter: `no-fallthrough`: Use string matching instead of Regex for default comment pattern (#6008) (camchenry)
- 2b17003 linter, prettier, diagnostics: Use `FxHashMap` instead of `std::collections::HashMap` (#5993) (camchenry)

## [0.9.7] - 2024-09-23

### Features

- d24985e linter: Add `oxc-security/api-keys` (#5906) (DonIsaac)
- f9b44c5 linter: Add unicode sets support to `no-useless-escape` rule (#5974) (camchenry)
- 0f19848 linter: Implement `no-unexpected-multiline` rule (#5911) (camchenry)
- 16fe383 linter: Implement `no-extend-native` rule (#5867) (Cam McHenry)

### Bug Fixes

- eed9ac7 linter: Include actual span size in `no-regex-spaces` diagnostic (#5957) (camchenry)
- 40c89c2 linter: Move `promise/avoid-new` to style category (#5961) (DonIsaac)

### Performance

- 608d637 linter: Use `aho-corasick` instead of `regex` for string matching in `jsx-a11y/img-redundant-alt` (#5892) (camchenry)
- 3148d4b linter: Check file path after checking node kind for `nextjs/no-head-element` (#5868) (Cam McHenry)

### Refactor

- 0a5a4a9 linter: Use parsed patterns for `unicorn/no-hex-escape` (#5985) (camchenry)
- 2cf2edd linter: Use parsed patterns in `no-empty-character-class` rule (#5980) (camchenry)
- a9a8e2a linter: Use regex parser in `eslint/no-regex-spaces` (#5952) (camchenry)
- 05f592b linter: Use parsed patterns in `unicorn/prefer-string-starts-ends-with` (#5949) (camchenry)
- 3273b64 linter: Use parsed patterns for `unicorn/prefer-string-replace-all` rule (#5943) (camchenry)
- ba7b01f linter: Add `LinterBuilder` (#5714) (DonIsaac)
- db4f16a semantic: Call `with_trivias` before `build_with_jsdoc` (#5875) (Boshen)
- 3d13c6d semantic: Impl `IntoIterator` for `&AstNodes` (#5873) (DonIsaac)

### Testing

- b681c9a linter: Import test cases for `no-empty-character-class` (#5981) (camchenry)
- 767602b linter: Add regression test for #5227 (#5975) (camchenry)

## [0.9.6] - 2024-09-18

### Features

- 3bf7b24 linter: Make `typescript/no-duplicate-enum-values` a `correctness` rule (#5810) (DonIsaac)
- 7799c06 linter/react: Implement `no-danger-with-children` rule (#5420) (Cam McHenry)

### Bug Fixes

- f942485 linter: Remove all* remaining "Disallow <foo>" messages (#5812) (DonIsaac)
- b5ad518 linter: Improve diagnostic messages for various lint rules (#5808) (DonIsaac)
- 858f7af linter: Plugin prefix name for eslint-plugin-node (#5807) (DonIsaac)
- 737ba1d linter: Fix some cases on ```AssignmentExpression``` for ```unicorn/consistent-function-scoping``` (#5675) (Arian94)
- 148c7a8 linter: Replace bitwise AND (&) with logical AND (&&) in explici… (#5780) (kaykdm)
- b4ed564 linter/no-unused-vars: Writes to members triggering false positive (#5744) (Dunqing)
- e9c084a linter/no-unused-vars: False positive when a variable used as a computed member property (#5722) (Dunqing)

### Performance

- 3725d5d linter: Make all rules share a diagnostics vec (#5806) (DonIsaac)
- e978567 linter: Shrink size of `DisableDirectives` (#5798) (DonIsaac)
- 1bfa515 linter: Remove redundant clone of diagnostics in context (#5797) (DonIsaac)
- e413cad linter: Move shared context info to `ContextHost` (#5795) (DonIsaac)

### Refactor

- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)
- 7caae5b codegen: Add `GetSpan` requirement to `Gen` trait (#5772) (Boshen)
- 026ee6a linter: Decouple module resolution from import plugin (#5829) (dalaoshu)
- 50834bc linter: Move `override_rule` to `OxlintRules` (#5708) (DonIsaac)
- a438743 linter: Move `OxlintConfig` to `Oxlintrc` (#5707) (DonIsaac)
- f61e8b5 linter: Impl serde and schemars traits for `LintPlugins` (#5706) (DonIsaac)
- 20a7861 linter: Shorten `Option` syntax (#5735) (overlookmotel)
- d8b612c oxc_linter: Prefer pass Enum instead of str `no_plus_plus` (#5730) (IWANABETHATGUY)
- cc0408b semantic: S/AstNodeId/NodeId (#5740) (Boshen)

## [0.9.5] - 2024-09-12

### Features

- 4b04f65 linter: Implement `no-plusplus` rule (#5570) (Cam McHenry)

## [0.9.4] - 2024-09-12

- 1fa3e56 semantic: [**BREAKING**] Rename `SymbolTable::iter` to `symbol_ids` (#5621) (overlookmotel)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

### Features

- 9ca2593 linter: Eslint/sort-keys  (#4845) (Na'aman Hirschfeld)
- 023c160 linter: Impl `Serialize` for `OxlintConfig` (#5594) (DonIsaac)
- 24d6a47 linter: Implement `eslint/no-invalid-regexp` (#5443) (Boshen)
- c6bbf94 minifier: Constant fold unary expression (#5669) (Boshen)

### Bug Fixes

- af6d240 linter: Panic in consistent-function-scoping (#5613) (DonIsaac)
- 54e2e76 linter: `react/no_set_state` + `react/no_string_refs` rules find correct parent (#5615) (overlookmotel)
- 3b87ac4 linter: Fix no_unused_vars panic when encountering unicode (#5582) (Boshen)

### Performance

- bfe9186 linter: Use `cow_replace` instead of `replace` (#5643) (dalaoshu)
- e3ae5db linter: Use cow_to_ascii_lowercase/uppercase (#5637) (heygsc)
- a0370bf linter: Use cow_utils in no_script_url (#5633) (heygsc)
- 37e922c linter: `eslint/no_shadow_restricted_names` use `run_on_symbol` (#5618) (overlookmotel)
- 0b7fccf linter: `react/no_set_state` + `react/no_string_refs` rules reduce iteration over ancestors (#5616) (overlookmotel)
- 2c3f3fe linter: Make `jsx_key` slightly faster (#5585) (Boshen)
- cd81d12 linter: Add `should_run` to check path only once to nextjs/no_typos (#5584) (Boshen)
- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

### Documentation

- 64f9575 linter: Add plugin usage to example with configuration (Boshen)
- 8c9179d linter: Fix typos (#5591) (Brian Donovan)

### Refactor

- 9e9435f linter: Add `LintFilter` (#5685) (DonIsaac)
- 4f70fe5 linter: Start internal/external split of LintPluginOptions (#5660) (DonIsaac)
- 5ae9b48 linter: Start internal/external split of `OxlintOptions` (#5659) (DonIsaac)
- c8bc6f0 linter: Use `std::ptr::eq` (#5649) (overlookmotel)
- a37c064 linter: Use `ContentHash` for `no_duplicate_case`; remove `calculate_hash` (#5648) (Boshen)
- 0b3c1d7 linter: Start internal/external split of `OxlintConfig` (#5595) (DonIsaac)
- 89bdf55 linter: Inline `Rule` trait default methods (#5619) (overlookmotel)
- afea8d5 linter: Rename `Rule` trait method params (#5617) (overlookmotel)
- 4e748b5 linter: Replace ast "compare by hash" to "compare by content" (#5602) (dalaoshu)
- bac03e3 linter: Make fields of `LintServiceOptions` private (#5593) (DonIsaac)
- 2661d8b linter: Jest prefer_strict_equal (#5588) (IWANABETHATGUY)
- 067f9b5 semantic: Introduce `IsGlobalReference` trait (#5672) (Boshen)- 26d9235 Enable clippy::ref_as_ptr  (#5577) (夕舞八弦)

### Testing

- 8e79f8d linter: Add class method test cases for `oxc/no-async-await` (#5550) (DonIsaac)
- 3835189 linter: Add test case for no_unused_vars in 3b87ac4 (Boshen)
- 5f27551 linter: Add a passing case to no_undef (#5580) (Boshen)

## [0.9.3] - 2024-09-07

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

- 87c5df2 ast: [**BREAKING**] Rename `Expression::without_parentheses` (#5448) (overlookmotel)

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)
- be3a432 linter: Implement typescript/no-magic-numbers (#4745) (Alexander S.)
- 09aa86d linter/eslint: Implement `sort-vars` rule (#5430) (Jelle van der Waa)
- 2ec2f7d linter/eslint: Implement no-alert (#5535) (Edwin Lim)
- a786acf linter/import: Add no-dynamic-require rule (#5389) (Jelle van der Waa)
- 4473779 linter/node: Implement no-exports-assign (#5370) (dalaoshu)
- b846432 linter/oxc: Add fixer for `erasing-op` (#5377) (camc314)
- aff2c71 linter/react: Implement `self-closing-comp` (#5415) (Jelle van der Waa)

### Bug Fixes

- 0df1d9d ast, codegen, linter: Panics in fixers. (#5431) (rzvxa)
- cdd1a91 linter: Typescript/no-magic-numbers: remove double minus for reporting negative bigint numbers (#5565) (Alexander S.)
- ff88c1f linter: Don't mark binding rest elements as unused in TS function overloads (#5470) (Cam McHenry)
- 088733b linter: Handle loops in `getter-return` rule (#5517) (Cam McHenry)
- 82c0a16 linter: `tree_shaking/no_side_effects_in_initialization` handle JSX correctly (#5450) (overlookmotel)
- 6285a02 linter: `eslint/radix` rule correctly check for unbound symbols (#5446) (overlookmotel)
- c8ab353 linter/tree-shaking: Align JSXMemberExpression's report (#5548) (mysteryven)
- 5187f38 linter/tree-shaking: Detect the correct export symbol resolution (#5467) (mysteryven)

### Performance

- 8170954 linter/react: Add should_run conditions for react rules (#5402) (Jelle van der Waa)

### Documentation

- a540215 linter: Update docs `Examples` for linter rules (#5513) (dalaoshu)
- 7414190 linter: Update docs `Example` for linter rules (#5479) (heygsc)

### Refactor

- 0ac420d linter: Use meaningful names for diagnostic parameters (#5564) (Don Isaac)
- 81a394d linter: Deduplicate code in `oxc/no-async-await` (#5549) (DonIsaac)
- 979c16c linter: Reduce nested if statements in eslint/no_this_before_super (#5485) (IWANABETHATGUY)
- 1d3e973 linter: Simplify `eslint/radix` rule (#5445) (overlookmotel)
- fdb8857 linter: Use "parsed pattern" in `no_div_regex` rule. (#5417) (rzvxa)
- 2ccbd93 linter: `react/jsx_no_undef` rule `get_member_ident` do not return Option (#5411) (overlookmotel)

### Styling

- 2a43fa4 linter: Introduce the writing style from PR #5491 and reduce the if nesting (#5512) (dalaoshu)- d8b29e7 Add trailing line breaks to JSON files (#5544) (overlookmotel)- 694f032 Add trailing line breaks to `package.json` files (#5542) (overlookmotel)

### Testing

- 340b535 linter/no-unused-vars: Arrow functions in tagged templates (#5510) (Don Isaac)
- af69393 linter/no-useless-spread: Ensure spreads on identifiers pass (#5561) (DonIsaac)- dc92489 Add trailing line breaks to conformance fixtures (#5541) (overlookmotel)

## [0.9.2] - 2024-09-02

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

### Features

- 180b1a1 ast: Add `Function::name()` (#5361) (DonIsaac)
- f81e8a1 linter: Add `oxc/no-async-endpoint-handlers` (#5364) (DonIsaac)
- b103737 linter: Improve no-accumulating-spread (#5302) (camc314)
- 9c22ce9 linter: Add hyperlinks to diagnostic messages (#5318) (DonIsaac)
- 1967c67 linter/eslint: Implement no-new-func (#5360) (dalaoshu)
- b867e5f linter/eslint-plugin-promise: Implement catch-or-return (#5121) (Jelle van der Waa)
- 8d781e7 linter/oxc: Differentiate between array/object in `no-accumulating-spread` loop diagnostic (#5375) (camc314)
- db55444 linter/oxc: Add fixer for `double-comparisons` (#5378) (camc314)
- e5c755a linter/promise: Add `spec-only` rule (#5124) (Jelle van der Waa)
- 4c0861f linter/unicorn: Add fixer for `prefer-type-error` (#5311) (camc314)
- 084c2d1 linter/vitest: Implement prefer-to-be-object (#5321) (dalaoshu)

### Bug Fixes

- 11b93af linter/unicorn: Consistent-function-scoping false positive on assignment expression (#5312) (Arian94)

### Performance

- f052a6d linter: `react/jsx_no_undef` faster check for unbound references (#5349) (overlookmotel)
- 05636b7 linter: Avoid unnecessary work in `jsx_a11y/anchor_is_valid` rule (#5341) (overlookmotel)

### Refactor

- afb038e linter: `react/jsx_no_undef` use loop instead of recursion (#5347) (overlookmotel)
- fe62687 linter: Simplify skipping JSX elements in `unicorn/consistent_function_scoping` (#5351) (overlookmotel)
- 381d9fe linter: Shorten code in `react/jsx_no_useless_fragment` (#5350) (overlookmotel)
- 83b9a82 linter: Fix indentation in `nextjs/no_script_component_in_head` rule (#5338) (overlookmotel)
- 89f0188 linter: Improve docs for `react/jsx_no_target_blank` rule (#5342) (overlookmotel)
- 57050ab linter: Shorten code in `jsx_a11y/aria_activedescendant_has_tabindex` rule (#5340) (overlookmotel)
- ed31d67 linter/jest: Fix indentation in code comment (#5372) (camc314)
- 2499cb9 linter/oxc: Update rule docs for `erasing-op` (#5376) (camc314)
- 69493d2 linter/oxc: Improve diagnostic for `no-accumulating-spread` in loops (#5374) (camc314)
- 024b585 linter/oxc: Improve code comment for `no-accumulating-spread` (#5373) (camc314)
- 3ae94b8 semantic: Change `build_module_record` to accept &Path instead of PathBuf (Boshen)

## [0.9.1] - 2024-08-29

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

### Features

- 6633972 linter: Add fixer for `no-empty` (#5276) (camc314)
- a58e448 linter/eslint: Add fixer to `no-var` (#5144) (camc314)
- a6e9769 linter/jsx-a11y: Add `label-has-associated-control` (#5163) (Billy Levin)
- c8e8532 linter/unicorn: Add fixer to `throw-new-error` (#5275) (camc314)
- 7ccde4b linter/unicorn: Add fixer to `prefer-date-now` (#5147) (camc314)

### Bug Fixes

- 76e86f8 linter: Eslint-plugin-unicorn prefer-spread wrong linter suggestion on variables of type string (#5265) (Arian94)
- b39544e linter/jest: Fixer for `prefer-jest-mocked` creates invalid LHS expressions (#5243) (camc314)
- 9953fa5 linter/no-null: Incorrect fixer for `NullLiteral` within `ReturnStatement` (#5247) (Dunqing)
- 318479e linter/no-unused-vars: Mark the class/function in the new expression as used (#5306) (magic-akari)

### Refactor

- fa1d460 linter: Clean up Fixer and Message (#5308) (DonIsaac)

## [0.9.0] - 2024-08-26

- 5946748 linter: [**BREAKING**] Parse and display syntax errors for regular expressions (#5214) (Boshen)

- b894d3b linter: [**BREAKING**] Make `no-unused-vars` correctness (#5081) (DonIsaac)

### Features

- 1ce9630 linter/config: Implement FromIterator for LintPluginOptions (#5102) (DonIsaac)
- 34bfaf6 linter/react: Add fixer to `jsx-props-no-spread-multi` (#5145) (camc314)
- 982bd6e linter/unicorn: Add fixer to `require-array-join-separator` (#5152) (camc314)
- a6704bd linter/unicorn: Add fixer to `prefer-set-size` (#5149) (camc314)
- ac7edcc linter/unicorn: Add fixer to `prefer-array-some` (#5153) (camc314)
- 1d01aa3 linter/unicorn: Add partial fixer for `prefer-array-flat` (#5143) (camc314)
- 22d57f9 linter/unicorn: Add fixer to `prefer-string-slice` (#5150) (Cameron)
- 2fe4415 linter/unicorn: Add fixer to `no-redundant-roles` (#5146) (Cameron)
- d35c6f5 linter/unicorn: Add fixer to `prefer-regexp-test` (#5151) (Cameron)
- 27db769 linter/unicorn: Add fixer to `text-encoding-identifier-case` (#5154) (Cameron)
- f7958c4 linter/unicorn: Add prefer-structured-clone (#5095) (Jelle van der Waa)
- 004ffa0 linter/vitest: Implement `prefer-each` (#5203) (dalaoshu)

### Bug Fixes

- aaaf26c linter: Error in fixer for prefer-to-have-length (#5197) (dalaoshu)
- 1f5b6b6 linter: Bug in fixer for prefer-to-have-length (#5164) (dalaoshu)
- 7eb052e linter: `no-hex-escape` fixer removing regex flags (#5137) (Cameron)
- 76c66b4 linter/max-lines: Point span to end of file for disable directive to work (#5117) (Boshen)
- 8ff6f2c linter/no-unused-vars: Panic on UsingDeclarations (#5206) (DonIsaac)
- d29042e linter/no-unused-vars: Function expression in implicit arrow function return (#5155) (DonIsaac)
- 36e4a28 linter/no-unused-vars: Panic in variable declarator usage checks (#5160) (DonIsaac)
- ba62a71 linter/react: Fixed false positive with missing key inside React.Children.toArray() for fragments  (#5133) (Earl Chase)
- fd1031a linter/unicorn: Breaking fixer in case statements for `no-null` (#5176) (DonIsaac)
- 7b86ed6 linter/unicorn: Handle type casts and parens in `no-null` (#5175) (Don Isaac)
- b629e16 linter/unicorn: Improve diagnostic message for `no-null` (#5172) (DonIsaac)

### Performance
- ce454cf Use simdutf8 to validate UTF-8 when reading files  (#5196) (dalaoshu)

### Refactor

- 543cad6 codegen: Remove some pub APIs (Boshen)
- 0d3661a linter: Remove meaningless `span0` (#5209) (dalaoshu)
- 2a91ef1 linter: `eslint/no_redeclare` rule use `run_on_symbol` not `run_once` (#5201) (overlookmotel)
- 33599b0 linter: Split options into multiple files (#5101) (DonIsaac)
- 7ab6152 linter/unicorn: Clean up `no-null` (#5174) (DonIsaac)

### Testing

- a877e5a linter/no-unused-vars: Ensure type annotations on property accessors are considered used (#5183) (DonIsaac)
- 7886618 linter/unicorn: Add fixer tests for `no-null` (#5173) (DonIsaac)

## [0.8.0] - 2024-08-23

- 5f4c9ab semantic: [**BREAKING**] Rename `SymbolTable::get_flag` to `get_flags` (#5030) (overlookmotel)

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

- b2ff2df parser: [**BREAKING**] Remove builder pattern from `Parser` struct (#5000) (Boshen)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 2292606 linter: Typescript-eslint/no-wrapper-object-types (#5022) (camc314)
- a0effab linter: Support more flexible config.globals values (#4990) (Don Isaac)
- cdbfcfb linter: Start import fixer for eslint/no-unused-vars (#4849) (DonIsaac)
- 915cb4d linter: Add dangerous fixer for oxc only used in recursion (#4805) (camc314)
- 3f28c77 linter/eslint: Improve no-dupe-keys (#4943) (DonIsaac)
- e1582a5 linter/eslint: Improve no-duplicate-case rule (#4942) (DonIsaac)
- f1e4611 linter/eslint-plugin-vitest: Implement no-conditional-in-test (#4971) (dalaoshu)
- 14bf5d5 linter/eslint-plugin-vitest: Implement no-restricted-vi-methods (#4956) (dalaoshu)
- ed9a1c4 linter/eslint-plugin-vitest: Implement require-local-test-context-for-concurrent-snapshots (#4951) (dalaoshu)
- 7859f58 linter/eslint-plugin-vitest: Implement no-conditional-tests (#4955) (dalaoshu)
- 841174f linter/no-unused-vars: Delete non-root arrows, skip `await` (#5083) (Don Isaac)

### Bug Fixes

- 86d0c0c linter: Change consistent-function-scoping to suspicious (#5010) (DonIsaac)
- 7b99386 linter: Missing closing ticks in some example blocks (#4994) (DonIsaac)
- 9c64b12 linter: Improve no-zero-fractions rule for member expressions and scientific notation (#4793) (Burlin)
- c43945c linter/consistent-function-scoping: Allow functions passed as arguments (#5011) (Don Isaac)
- 9354779 linter/no-unused-vars: Give `argsIgnorePattern` the same default behavior as `varsIgnorePattern` (#5018) (DonIsaac)
- 5a55dcf linter/no-unused-vars: `type` specifier not deleted for type imports (#5029) (DonIsaac)
- 4081293 linter/no-unused-vars: Panic in fixer when removing destructures (#4923) (Don Isaac)
- ddf83ff linter/react: Fixed false positive with missing key inside React.Children.toArray() (#4945) (Earl Chase)
- 508644a linter/tree-shaking: Correct the calculation of `>>`, `<<` and `>>>` (#4932) (mysteryven)
- e99836d linter/unicorn: Allow set spreading in no-useless-spread (#4944) (Don Isaac)
- 5f8a7c2 oxlint: Rules in the configuration file are not being correctly … (#4949) (dalaoshu)

### Documentation

- e331ca0 linter: Improve documentation for several rules (#4997) (DonIsaac)
- cd9f1cd linter/consistent-function-scoping: Improve rule documentation (#5015) (DonIsaac)

### Refactor

- eca6fdb linter: Move plugin options into separate struct (#5100) (DonIsaac)
- 06f2d81 linter: Avoid unnecessary temp `Vec`s (#4963) (overlookmotel)
- 4cb8c37 linter: Move default_true to utils (#4947) (Don Isaac)
- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)
- 59d15c7 semantic: `root_unresolved_references` contain only `ReferenceId` (#4959) (overlookmotel)

### Testing

- c21d735 linter/no-unused-vars: Add ignored destructuring test cases (#4922) (Don Isaac)

## [0.7.2] - 2024-08-15

### Features

- 97e38cd linter: Add fixer for unicorn/prefer-optional-catch-binding (#4867) (heygsc)
- 93ae1c7 linter: Eslint-plugin-react jsx-props-no-spread-multi (#4866) (keita hino)
- 0a23610 linter: Add fixer for unicorn/prefer-array-flat-map (#4844) (heygsc)
- 13c7b1b linter/jsx-a11y: Add fixer for aria-unsupported-elements (#4854) (DonIsaac)
- a6195a6 linter/jsx-a11y: Add fixer for anchor-has-content (#4852) (DonIsaac)
- 4d28d03 task/website: Support render `subschemas.all_of` (#4800) (mysteryven)

### Bug Fixes

- 21f5762 codegen: Minify large numbers (#4889) (Boshen)
- a08d7a7 linter/jsx-a11y: Reduce false negatives for html-has-lang (#4855) (DonIsaac)
- a81ce3a linter/no-unused-vars: Do not delete function expressions when fixing (#4848) (DonIsaac)

### Refactor

- 56f033c linter: Improve diagnostics for several jsx-a11y rules (#4853) (DonIsaac)
- c53c210 linter/no-unused-vars: Split fixer logic into multiple files (#4847) (DonIsaac)

## [0.7.1] - 2024-08-12

### Features

- 3d40528 linter: Add fix emoji to rules table and doc pages (#4715) (DonIsaac)
- d2734f3 linter: Start fixer for no-unused-vars (#4718) (DonIsaac)
- 070ae53 linter: Add fixer for unicorn prefer-string-replace-all (#4801) (camc314)
- b3c3125 linter: Overhaul unicorn/no-useless-spread (#4791) (DonIsaac)
- 5992b75 linter: Implement `eslint-plugin-promise/no-return-in-finally, prefer-await-to-then` rule (#4318) (Jelle van der Waa)
- b259f47 linter: Add fixer for unicorn/no-length-as-slice-end (#4780) (heygsc)
- abd83fa linter: Add fixer for jsx_ally/no_aria_hidden_on_focusable (#4772) (heygsc)
- b20e335 linter: Add fixer for eslint/no-eq-null (#4758) (heygsc)
- 2f6c3b9 linter: Add fixer for eslint/no-compare-neg-zero (#4748) (heygsc)
- eaddc8f linter: Add fixer for eslint/func_names (#4714) (DonIsaac)
- 80557a9 linter: Add fixer for eslint/for-direction (#4679) (heygsc)
- c3c5766 linter/eslint-plugin-promise: Implement valid-params (#4598) (Jelle van der Waa)
- c509a21 linter/eslint-plugin-vitest: Implement prefer-to-be-falsy (#4770) (dalaoshu)
- 41f861f linter/eslint-plugin-vitest: Implement prefer-to-be-truthy (#4755) (dalaoshu)
- cc922f4 vscode: Provide config's schema to oxlint config files (#4826) (Don Isaac)
- f629514 website: Auto-generate rule docs pages (#4640) (DonIsaac)

### Bug Fixes

- b22ed45 linter: Improve prefer_namespace_keyword rule (#4751) (Burlin)
- db68a6c linter: Fixer for eslint/for-direction (#4727) (heygsc)
- 6273994 linter: Block in eslint/no_cond_assign (#4721) (heygsc)
- b9d6aa5 linter: Fix false positives in no-confusing-non-null-assertion (#4665) (Renée)
- cbf08d2 linter: Skip no-multi-str on jsx attributes (#4666) (heygsc)
- a6f9f96 linter: No unused errors should be warnings (Boshen)
- 7345bc9 linter/func-names: Handle ts accessibility when reporting missing names (#4713) (DonIsaac)

### Performance

- d191823 linter: Optmize allocations in jest fn parsing (#4787) (lucab)
- e3abdfa linter: Reduce String allocations and clones (#4673) (DonIsaac)

### Documentation

- 4b7dfd6 linter: Correct docs for no-unused-vars (#4716) (Don Isaac)

### Refactor

- 096ac7b linter: Clean up jsx-a11y/anchor-is-valid (#4831) (DonIsaac)
- 15a0fd4 linter: Use Option to reduce nested level in `eslint/getter-return` (#4814) (IWANABETHATGUY)
- 63f274c linter: Simplify NoObjCalls resolution logic (#4765) (lucab)
- 6708680 linter: Replace Windows-style line breaks with Unix-style in test fixture (#4768) (overlookmotel)
- e285903 linter: Clean up eslint/func_names (#4710) (DonIsaac)

### Testing

- 8f2a566 linter: Ensure rule docs have valid syntax (#4644) (DonIsaac)
- 4dd29db linter: Add fixer test for unicorn/no-zero-fractions (#4783) (heygsc)

## [0.7.0] - 2024-08-05

- 85a7cea semantic: [**BREAKING**] Remove name from `reference` (#4329) (Dunqing)

### Features

- aaee07e ast: Add `AstKind::AssignmentTargetPattern`, `AstKind::ArrayAssignmentTarget` and `AstKind::ObjectAssignmentTarget` (#4456) (Dunqing)
- 9df7b56 jsx-a11y/no-autofocus: Implement fixer support (#4171) (Jelle van der Waa)
- b87bf70 linter: Add fix capabilties to existing lint rules (#4560) (DonIsaac)
- ddd8b27 linter: Support conditional fix capabilities (#4559) (DonIsaac)
- b952942 linter: Add eslint/no-unused-vars (⭐ attempt 3.2) (#4445) (DonIsaac)
- 6543958 linter: Add auto-fix metadata to RuleMeta (#4557) (Don Isaac)
- 85e8418 linter: Add react/jsx-curly-brace-presence (#3949) (Don Isaac)
- 4c4da56 linter: Add typescript-eslint/prefer-keyword-namespce (#4438) (Aza Walker)
- d8c2a83 linter: Eslint-plugin-vitest/no-import-node-test (#4440) (cinchen)
- e3b0c40 linter: Eslint-plugin-vitest/no-identical-title (#4422) (cinchen)
- c936782 linter: Eslint-plugin-vitest/no-conditional-expect (#4425) (cinchen)
- 27fdd69 linter: Eslint-plugin-vitest/no-commented-out-tests (#4424) (cinchen)
- 51f5025 linter: Add fixer for unicorn/prefer-string-starts-ends-with (#4378) (DonIsaac)
- 3c0c709 linter: Add typescript-eslint/no-extraneous-class (#4357) (Jaden Rodriguez)
- 7afa1f0 linter: Support suggestions and dangerous fixes (#4223) (DonIsaac)
- acc5729 linter: Eslint-plugin-vitest/expect-expect (#4299) (cinchen)
- 2213f93 linter: Eslint-plugin-vitest/no-alias-methods (#4301) (cinchen)
- c296bc3 linter/eslint: Implement func-names (#4618) (Alexander S.)
- e116ae0 linter/eslint: Implement fixer for prefer-numeric-literals (#4591) (Jelle van der Waa)
- eaf834f linter/eslint: Implement prefer-numeric-literals (#4109) (Jelle van der Waa)
- db2fd70 linter/eslint-plugin-promise: Implement no-webpack-loader-syntax (#4331) (Jelle van der Waa)
- 5f1e070 linter/eslint-plugin-unicorn: Add fixer for prefer-code-point (#4353) (Jelle van der Waa)
- ed49e16 linter/eslint-plugin-unicorn: Implement fixer for prefer-dom-node-append (#4306) (Jelle van der Waa)
- e2b15ac linter/react: Implement react-jsx-boolean-value (#4613) (Jelle van der Waa)
- 68efcd4 linter/react-perf: Handle new objects and arrays in prop assignment patterns (#4396) (DonIsaac)

### Bug Fixes

- 368112c ast: Remove `#[visit(ignore)]` from `ExportDefaultDeclarationKind`'s `TSInterfaceDeclaration` (#4497) (Dunqing)
- d384f60 ci: Remove unused(?) .html file (#4545) (Yuji Sugiura)
- 06aec77 linter: Invalid binary expression with overflow (#4647) (DonIsaac)
- b2da22b linter: Invalid tags in rule docs (#4646) (DonIsaac)
- 94440ad linter: Panic on invalid lang in `a11y/lang`. (#4630) (rzvxa)
- e0b03f8 linter: Improve the boundary for eslint/for-direction (#4590) (heygsc)
- 70b8cfa linter: Missing return in no-obj-calls recursion (#4594) (DonIsaac)
- fe1356d linter: Change no-unused-vars to nursery (#4588) (DonIsaac)
- 72337b1 linter: Change typescript-eslint/no-namespace to restriction (#4539) (Don Isaac)
- 289dc39 linter: Overflow in no-obj-calls (#4397) (DonIsaac)
- a664715 linter/eslint: Fix invalid regexp in no_regex_spaces test (#4605) (Yuji Sugiura)
- 74fa75a linter/eslint: Drop quotes around max-params lint warning (#4608) (Jelle van der Waa)
- 9fcd9ae linter/eslint: Fix invalid regexp in no_control_regex test (#4544) (leaysgur)
- ac08de8 linter/react_perf: Allow new objects, array, fns, etc in top scope (#4395) (DonIsaac)
- 73d2558 oxlint: Fix oxlint failed to build due to missing feature (Boshen)

### Performance

- 6ff200d linter: Change react rules and utils to use `Cow` and `CompactStr` instead of `String`  (#4603) (DonIsaac)
- f259df0 linter: Make img-redundant-alt only build a regex once (#4604) (DonIsaac)
- 7585e16 linter: Remove allocations for string comparisons (#4570) (DonIsaac)
- b60bdf1 linter: `no_shadow_restricted_names` only look up name in hashmap once (#4472) (overlookmotel)
- 81384f5 linter: Avoid unnecessary work in `nextjs:no_duplicate_head` rule (#4465) (overlookmotel)
- f7da22d linter: Disable lint rules by file type (#4380) (DonIsaac)
- 348c1ad semantic: Remove `span` field from `Reference` (#4464) (overlookmotel)
- 6a9f4db semantic: Reduce storage size for symbol redeclarations (#4463) (overlookmotel)- a207923 Replace some CompactStr usages with Cows (#4377) (DonIsaac)

### Refactor

- 7a75e0f linter: Use diagnostic codes in lint rules (#4349) (DonIsaac)
- ccb1835 semantic: Methods take `Span` as param, not `&Span` (#4470) (overlookmotel)
- 7cd53f3 semantic: Var hoisting (#4379) (Dunqing)
- c99b3eb syntax: Give `ScopeId` a niche (#4468) (overlookmotel)

## [0.6.1] - 2024-07-17

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- 1f8968a linter: Add eslint-plugin-promise rules: avoid-new, no-new-statics, params-names (#4293) (Jelle van der Waa)
- a4dc56c linter: Add fixer for unicorn/no_useless_promise_resolve_reject (#4244) (Burlin)
- 6fb808f linter: Add typescript-eslint/no-confusing-non-null-assertion (#4224) (Jaden Rodriguez)
- 126b66c linter: Support eslint-plugin-vitest/valid-describe-callback (#4185) (cinchen)
- 05b9a73 linter: Support eslint-plugin-vitest/valid-expect (#4183) (cinchen)
- 3e56b2b linter: Support eslint-plugin-vitest/no-test-prefixes (#4182) (cinchen)
- 3016f03 linter: Let fixer functions return a `None` fix (#4210) (DonIsaac)
- bbe6137 linter: Implement unicorn/no-useless-undefined (#4079) (Burlin)
- 20cdb1f semantic: Align class scope with typescript (#4195) (Dunqing)

### Bug Fixes

- 9df60da linter: Correct find first non whitespace logic in @typescript-eslint/consistent-type-imports (#4198) (mysteryven)
- 67240dc linter: Not ignore adjacent spans when fixing (#4217) (mysteryven)
- dd07a54 linter: Global variables should always check the builtin variables (#4209) (Jelle van der Waa)
- 351ecf2 semantic: Incorrect resolve references for `TSTypeQuery` (#4310) (Dunqing)
- 1108f2a semantic: Resolve references to the incorrect symbol (#4280) (Dunqing)

### Performance

- 0fdc88b linter: Optimize no-dupe-keys (#4292) (lucab)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)
- aa22073 codegen: Improve print API (#4196) (Boshen)
- b5a8f3c linter: Use get_first_parameter_name from unicorn utils (#4255) (Jelle van der Waa)
- 7089a3d linter: Split up fixer code into separate files (#4222) (DonIsaac)
- ace4f1f semantic: Update the order of `visit_function` and `Visit` fields in the builder to be consistent (#4248) (Dunqing)
- 7f1addd semantic: Correct scope in CatchClause (#4192) (Dunqing)

## [0.6.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Features

- fb549e1 linter: Add vitest/no-focused-tests rule (#4178) (mysteryven)
- 6c49007 linter: Add fixer for @typescript-eslint/consistent-type-imports (#3984) (mysteryven)
- 278c3e9 linter: Add fixer for jsx-a11y/aria-props (#4176) (DonIsaac)
- 2188144 linter: Eslint-plugin-jest/prefer-hooks-in-order (#4052) (cinchen)
- cc58614 linter: Better schemas for allow/warn/deny (#4150) (DonIsaac)
- c5b4be0 linter: Add fixer for prefer-node-protocol (#4129) (DonIsaac)
- 7ec0c0b linter/eslint: Implement no-label-var (#4087) (Jelle van der Waa)

### Bug Fixes

- ed4c54c eslint/radix: Detect yield Number.parseInt variant (#4110) (Jelle van der Waa)
- e9ad03b linter: Fixer for no-debugger creates incorrect code (#4184) (DonIsaac)
- bd69571 linter: Fix top level return panic in eslint/array_callback_return (#4167) (Boshen)
- c8f5664 linter: Fix panic with unicode in unicorn/prefer_dom_node_dataset (#4166) (Boshen)
- f2b3273 linter: Fix fixer panic in typescript/consistent_indexed_object_style (#4165) (Boshen)
- 2334515 linter: Panic in `get_enclosing_function` (#4121) (DonIsaac)
- 1b91d40 linter: Incorrect fixer for `no-unused-labels` (#4123) (Don Isaac)
- 1729249 linter: Incorrect fix in `no-single-promise-in-promise-methods` rule; (#4094) (DonIsaac)
- cc7e893 linter/tree-shaking: Avoid recursive function stackoverflow (#4191) (mysteryven)
- 28eeee0 parser: Fix asi error diagnostic pointing at invalid text causing crash (#4163) (Boshen)
- 0f02608 semantic: Bind `TSImportEqualsDeclaration`s (#4100) (Don Isaac)

### Performance

- ddfa343 diagnostic: Use `Cow<'static, str>` over `String` (#4175) (DonIsaac)

### Refactor

- 2687ebc react: Use find_binding helper for finding React binding (#4108) (Jelle van der Waa)

## [0.5.3] - 2024-07-07

### Features

- 1681b11 linter: Eslint-plugin-jest/consistent-test-it (#4053) (cinchen)
- 6876490 linter: Add rule no-undefined (#4041) (jordan boyer)
- bf04dee linter: Implement unicorn/no-negation-in-equality-check (#4034) (Nissim Chekroun)
- aa45604 linter/eslint: Implement no-multi-str (#4038) (Jelle van der Waa)

### Bug Fixes

- 7b2dc3b linter: Fix panic in import/namespace (#4080) (Boshen)

## [0.5.2] - 2024-07-02

### Features

- b257d53 linter: Support report `@typescript-eslint/consistent-type-imports` (#3895) (mysteryven)
- 2114475 linter: Implement @typescript-eslint/no-dynamic-delete (#3971) (kaykdm)
- 10a3c9a linter/eslint-plugin-react: Implement no-set-state (#3975) (Jelle van der Waa)

### Bug Fixes

- 432d6d9 linter: Find disabled directives using the message's `Span`. (#4010) (rzvxa)
- dbbb6fc linter: Global variable check should always check builtin variables (#3973) (Boshen)


## [0.5.1] - 2024-06-29

### Features

- f64ad4b semantic: Make jsdoc building optional (turned off by default) (#3955) (Boshen)

### Bug Fixes

- c26975a linter: Only show the filename for max-lines (#3966) (Boshen)
- 94329e4 linter: Handle useful but empty constructors in no-useless-constructor (#3951) (DonIsaac)
- 6498a08 linter: No-useless-spread fixer with multiple spread elements (#3950) (DonIsaac)

### Refactor

- 1cca2a8 eslint: Convert with_labels to with_label where applicable (#3946) (Jelle van der Waa)
- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.5.0] - 2024-06-27

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

- 0537d29 cfg: [**BREAKING**] Move control flow to its own crate. (#3728) (rzvxa)

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 4bce59d semantic/cfg: [**BREAKING**] Re-export `petgraph` as `control_flow::graph`. (#3722) (rzvxa)

### Features

- 3ae2628 linter: Change `no-import-assign` to correctness (#3928) (Boshen)
- a89d501 linter: Implement @typescript-eslint/no-non-null-asserted-nulli… (#3850) (kaykdm)
- fc48cb4 linter: eslint-plugin-jest/prefer-jest-mocked (#3865) (cinchen)
- 63b98bd linter: Accept multiple fixes when fix code (#3842) (mysteryven)
- 328445b linter: Support `vitest/no-disabled-tests` (#3717) (mysteryven)
- 8c61f9c linter: Implement @typescript-eslint/no-non-null-assertion (#3825) (kaykdm)
- 080ecbd linter: Add `no-fallthrough`. (#3673) (rzvxa)
- 9493fbe linter: Add `oxc/no-optional-chaining` rule (#3700) (mysteryven)
- 139adfe linter: Add `@typescript-eslint/no-import-type-side_effects` (#3699) (mysteryven)
- 5f84500 linter/eslint-plugin-react: Implement prefer-es6-class (#3812) (Jelle van der Waa)
- fafe67c linter/import: Implement max-dependencies (#3814) (Jelle van der Waa)
- d5f6aeb semantic: Check for illegal symbol modifiers (#3838) (Don Isaac)

### Bug Fixes

- 4bd2c88 linter: Fix and promote `getter-return` to correctness. (#3777) (rzvxa)
- 1190dee linter: False positives with setters in the `getter-return` rule. (#3714) (rzvxa)
- de0690f linter: Do not run getter-return in typescript (#3693) (Boshen)
- cf71c23 linter: Edge case with infinite loops. (#3672) (rzvxa)
- 99a40ce semantic: `export default foo` should have `ExportLocalName::Default(NameSpan)` entry (#3823) (Boshen)
- abd6ac8 semantic/cfg: Discrete finalization path after `NewFunction`s. (#3671) (rzvxa)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 4d2b7f1 linter: `LintContext` can now only be constructed with a cfg enabled semantic. (#3761) (rzvxa)
- 7302429 linter/prefer_number_properties: Remove the unused `IdentifierName` check (#3822) (Boshen)
- d8ad321 semantic: Make control flow generation optional. (#3737) (rzvxa)

### Testing

- 887da40 linter: Enable `no-fallthrough` test with `disable-next-line`. (#3766) (rzvxa)

## [0.4.4] - 2024-06-14

### Features

- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)
- 29c78db linter: Implement @typescript-eslint/explicit-function-return-type (#3455) (kaykdm)
- 21d3425 linter: Typescript-eslint no-useless-empty-export (#3605) (keita hino)
- 85c3b83 linter: Eslint-plugin-jest/max-nested-describes (#3585) (cinchen)
- f6d9ca6 linter: Add `eslint/sort-imports` rule (#3568) (Wang Wenzhe)
- 046ff3f linter/eslint: Add `no_unreachable` rule. (#3238) (rzvxa)
- e32ce00 linter/jsdoc: Implement require-param-name rule (#3636) (Yuji Sugiura)
- 110661c linter/jsdoc: Implement require-param-description (#3621) (Yuji Sugiura)
- d6370f1 linter/jsdoc: Implement require-param-type rule (#3601) (Yuji Sugiura)
- d9c5b33 semantic/cfg: Add `Condition` instruction. (#3567) (Ali Rezvani)
- f2dfd66 semantic/cfg: Add iteration instructions. (#3566) (rzvxa)

### Bug Fixes

- f0b689d linter: Panic in jsdoc/require-param (#3590) (Don Isaac)
- e148a32 semantic/cfg: Correct unreachability propagation in try-finally. (#3667) (Ali Rezvani)

### Refactor

- 84304b4 linter: Add a `ctx.module_record()` method (#3637) (Boshen)
- f98f777 linter: Add rule fixer (#3589) (Don Isaac)
- fa11644 linter: Pass `Rc` by value (#3587) (overlookmotel)
- f702fb9 semantic/cfg: Cleanup control flow and it's builder. (#3650) (rzvxa)
- 5793ff1 transformer: Replace `&’a Trivias` with `Rc<Trivias>` (#3580) (Dunqing)

## [0.4.3] - 2024-06-07

### Features

- 1fb9d23 linter: Add fixer for no-useless-fallback-in-spread rule (#3544) (Don Isaac)
- 6506d08 linter: Add fixer for no-single-promise-in-promise-methods (#3531) (Don Isaac)
- daf559f linter: Eslint-plugin-jest/no-large-snapshot (#3436) (cinchen)
- 4c17bc6 linter: Eslint/no-constructor-return (#3321) (谭光志)
- 4a075cc linter/jsdoc: Implement require-param rule (#3554) (Yuji Sugiura)
- 747500a linter/jsdoc: Implement require-returns-type rule (#3458) (Yuji Sugiura)
- 6b39654 linter/tree-shaking: Support options (#3504) (Wang Wenzhe)
- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

### Bug Fixes

- b188778 linter/eslint: Fix `require-await` false positives in `ForOfStatement`. (#3457) (rzvxa)
- 350cd91 parser: Should parser error when function declaration has no name (#3461) (Dunqing)

## [0.4.2] - 2024-05-28

### Features

- 14ef4df lint/eslint: Implement require-await (#3406) (Todor Andonov)
- e275659 linter: Add `oxc/no-rest-spread-properties` rule (#3432) (Wang Wenzhe)
- 0d2c977 linter: Add `oxc/no-const-enum` rule (#3435) (Wang Wenzhe)
- 085f917 linter: Add `oxc/no-async-await` rule (#3438) (Wang Wenzhe)
- ded59bc linter: Eslint-plugin-jest/require-top-level-describe (#3439) (cinchen)
- edaa555 linter: Eslint-plugin-jest/prefer-hooks-on-top (#3437) (cinchen)
- aa26ce9 linter: @typescript-eslint/consistent-indexed-object-style (#3126) (Todor Andonov)
- b589fd6 linter/eslint: Implement no-div-regex (#3442) (Jelle van der Waa)
- 147864c linter/eslint: Implement no-useless-concat (#3363) (Jelle van der Waa)

### Bug Fixes

- 5e06298 linter: Memorize visited block id in `neighbors_filtered_by_edge_weight` (#3407) (mysteryven)
- 74b06a7 linter: Accept more valid regex (#3408) (magic-akari)
- 19bb1c0 website: Hack `schemars` to render code snippet in markdown (#3417) (Boshen)

### Documentation

- 5c7041b linter: Add docs for consistent-indexed-object-style (#3409) (Wang Wenzhe)

## [0.4.0] - 2024-05-24

### Features

- e241136 cli,linter: Add `--disable-oxc-plugin` (#3328) (Boshen)
- 8ab9856 cli,linter: Add `--disable`-react/unicorn/typescript-`plugin` (#3305) (Boshen)
- ecdffcf linter: Temporary move react/require-render-return to nursery (Boshen)
- b8997f5 linter: Eslint/no-restricted-globals (#3390) (mysteryven)
- 79811ca linter: Change jsdoc/require-returns from correctness to pedantic (Boshen)
- 8a1db67 linter: Change jsdoc/require-render-return from correctness to pedantic (Boshen)
- fe208dd linter: Start adding json schema for configuration file (#3375) (Boshen)
- aec613b linter: Eslint-plugin-jest/no-duplicate-hooks (#3358) (cinchen)
- e4b3a3c linter: Backward compability for `react-hooks` and `deepscan` plugins (#3334) (Boshen)
- 9744707 linter/eslint: Implement default_case rule (#3379) (Jelle van der Waa)
- 74be8b1 linter/eslint: Implement no-new (#3368) (Jelle van der Waa)
- c588e52 linter/eslint: Implement prefer-exponentiation-operator (#3365) (Jelle van der Waa)
- 283d6c7 linter/eslint: Implement symbol-description (#3364) (Jelle van der Waa)
- b6e2d62 linter/jsdoc: Implement require-returns-description (#3397) (Yuji Sugiura)
- 3a5f088 linter/jsdoc: Implement require-returns rule (#3218) (Yuji Sugiura)
- 3671b5c tasks/website: Code generate the linter rules (Boshen)
- 57d2bca tasks/website: Start generating linter config markdown from json schema (#3386) (Boshen)
- ead637b website: Generate linter configuration page (Boshen)

### Bug Fixes

- c664c6c linter: `no-new` false positive when return from arrow expression (#3393) (Boshen)
- fbccd1f linter: Only report issues on top-level fragment (#3389) (Jovi De Croock)
- a23bbf9 linter: Avoid infinite loop in `jest/expect-expect` (#3332) (mysteryven)
- 385965f linter: Avoid infinite loop when traverse ancestors in `jest/no_conditional_expect` (#3330) (mysteryven)
- 95e9b69 linter: Fix panic in jest/expect-expect (#3324) (Boshen)
- 6c3d99a linter/jsx-no-undef: Check for globals when an identifier is undefined (#3331) (Boshen)
- bb2221e linter/next: False positives for non-custom font link (#3383) (Dunqing)
- 712ee0d linter/react: Fix false positives for async components in `rules_of_hooks` (#3307) (rzvxa)
- 0864cd0 linter/react: Better detection for hooks in the `rules_of_hooks`. (#3306) (rzvxa)
- 9594441 linter/react: `rules_of_hooks` add support for property hooks/components. (#3300) (rzvxa)
- c8f1f79 linter/react: `rules_of_hooks` resolve false positives with conditional hooks. (#3299) (rzvxa)
- d46538e linter/react: Fix loop hooks false positives. (#3297) (Ali Rezvani)

### Performance

- 8388c7b linter: Use `usize` for `RuleEnum` hash (#3336) (Boshen)

### Refactor

- c9d84af diagnostics: S/warning/warn (Boshen)
- 5bf595d linter: Rename variable names prefix `ESLint` to `Oxlint` (Boshen)
- d8c3187 linter: Remove unnecessary check in `eslint/no-global-assign` (#3391) (mysteryven)
- d7849f8 linter: Find return statement by using CFG in `react/require-render-return` (#3353) (mysteryven)
- 8383b6e linter: Remove `with_rule_name` from the tight loop (#3335) (Boshen)
- 4f76cb6 linter: Merge deepscan rules into oxc rules (#3327) (Boshen)
- 78e6326 semantic/cfg: Alias petgraph's `NodeIndex` as `BasicBlockId`. (#3380) (rzvxa)

## [0.3.5] - 2024-05-15

### Features

- 5b2fc39 linter: Add use-isnan fixer for (in)equality operations (#3284) (Don Isaac)
- 3644400 linter/eslint: Implement fixer for unicode-bom rule (#3259) (Jelle van der Waa)

### Bug Fixes

- e12323f linter/no-direct-mutation-state: False positive when class is declared inside a `CallExpression` (#3294) (Boshen)

### Refactor

- 6128171 linter: Rewrite react/require-render-return (#3276) (Wang Wenzhe)

## [0.3.4] - 2024-05-13

### Features

- 6edcae8 linter: Move react/rules_of_hooks to nursery (Boshen)
- 44b16ef linter/eslint: Implement max-classes-per-file (#3241) (Jelle van der Waa)

## [0.3.3] - 2024-05-13

### Features

- c6874ad linter: Demote `no-inner-declarations` from correctness to pedantic (eslint v9) (Boshen)
- 4ccc3ee linter: Demote `react/jsx-no-useless-fragment` from correctness to pedantic (Boshen)
- d45b28a linter: Unicorn/no-anonymous-default-export (#3220) (1zumii)
- 7113e85 linter: Add `radix` rule (#3167) (Kuba Jastrzębski)
- fa0093b linter: Eslint-plugin-next/no-page-custom-font (#3185) (Dunqing)
- 4defe37 linter: Remove deprecated eslint v9 rules `no-return-await` and `no-mixed-operators` (#3188) (Boshen)
- ca9f13f linter: Eslint/no-new-native-nonconstructor (#3187) (Boshen)
- 5514936 linter: Eslint-plugin-next/no-styled-jsx-in-document (#3184) (Dunqing)
- cb2e651 linter: Eslint-plugin-next/no-duplicate-head (#3174) (Boshen)
- 8244d2b linter/eslint: Implement unicode-bom rule (#3239) (Jelle van der Waa)
- 5081652 linter/eslint: Implement no-empty-function rule (#3181) (Jelle van der Waa)
- f88f330 linter/import: Improve multiple exports error message (#3160) (Dunqing)
- 1f135ce linter/react: Add the `rules_of_hooks` rule. (#3071) (rzvxa)
- c0abbbd linter/tree-shaking: Add `isPureFunction` (#3175) (Wang Wenzhe)

### Bug Fixes

- edb30e1 linter: Handle `import { default as foo }` in import/named (#3255) (Boshen)
- 313fb83 linter/default: Ignore unsupported files (e.g. .vue) (Boshen)
- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)

### Refactor

- dbde5b3 diagnostics: Remove export of `miette` (Boshen)
- 551632a diagnostics: Remove thiserror (Boshen)
- 312f74b diagnostics: S/OxcDiagnostic::new/OxcDiagnostic::error (Boshen)
- f7a3773 linter: Clean up diagnostics (Boshen)
- 5671714 linter: Clean up diagnostics in fixer (Boshen)
- 6e90f67 linter: Remove unnecessary usages of `CompactStr` (Boshen)
- 15f275f linter: Reduce llvm lines generated by `RuleEnum::read_json` (#3207) (Boshen)
- a84454c linter: Clean up prefer_node_protocol and move to restriction (#3171) (Boshen)
- f6f7adc linter,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3235) (Boshen)
- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)- 893af23 Clean up more diagnostics usages (Boshen)

## [0.3.2] - 2024-05-04

### Features

- 80cf0b2 linter: @typescript-eslint/prefer-literal-enum-member (#3134) (kaykdm)
- cd600fa linter: Add more "ban-ts-comment" test cases. (#3107) (谭光志)
- bef8a71 linter: Eslint-plugin-jest/require-hook (#3110) (cinchen)
- 388ee51 linter: Typescript-eslint/prefer-enum-initializers (#3097) (Todor Andonov)
- be9cdfc linter: Eslint/no-await-in-loop (#3070) (谭光志)
- 6f5df11 linter/import: Move some rules out of nursery (#2841) (Dunqing)
- 5a1d63a linter/jsdoc: Implement require-yields rule (#3150) (Yuji Sugiura)
- d7a8345 linter/jsdoc: Support settings.ignore(Private|Internal) (#3147) (Yuji Sugiura)
- 5866086 linter/jsdoc: Implement no-defaults rule (#3098) (Yuji Sugiura)
- fa3d9d2 linter/jsdoc: Implement `implements-on-classes` rule (#3081) (Yuji Sugiura)
- d109767 linter/jsdoc: Implement check-tag-names rule (#3029) (Yuji Sugiura)
- 32df6d7 linter/tree-shaking: Support While/Switch/Yield Statement (#3155) (Wang Wenzhe)
- 8290421 linter/tree-shaking: Support SequenceExpression (#3154) (Wang Wenzhe)
- 5c21b7f linter/tree-shaking: Support UnaryExpression (#3153) (Wang Wenzhe)
- 7333618 linter/tree-shaking: Support JSX (#3139) (Wang Wenzhe)
- 16a31e9 linter/tree-shaking: Support import statement (#3138) (Wang Wenzhe)
- 88ded0c linter/tree-shaking: Support ForStatement (#3078) (Wang Wenzhe)
- c3ec710 linter/tree-shaking: Support ExportNamedDeclaration (#3072) (Wang Wenzhe)
- 8cdd5b0 linter/tree_shaking: Support LogicExpression and MemberExpression (#3148) (Wang Wenzhe)

### Bug Fixes

- fde7d65 linter: Handle named export default in import-plugin/named (#3158) (Boshen)
- b1bddac linter: Fix hang if a file fails to parse while using `--import-plugin` (Boshen)
- dcb2528 semantic: Revert test code pushed to the main by accident. (#3085) (Ali Rezvani)
- 8d17ab3 semantic: Allow `root_node` to be empty for empty trees. (#3084) (Ali Rezvani)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 942b2ba ast: Add array element `Elision` type (#3074) (overlookmotel)
- 222030c linter: Render `--rules` in a table (Boshen)
- 1f12aee linter/jsdoc: Misc improvements (#3109) (Yuji Sugiura)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- ae65613 syntax: Use `FxHashMap` for `ModuleRecord::request_modules` (#3124) (Boshen)

## [0.3.1] - 2024-04-22

### Bug Fixes

- a5a7351 linter: Fix unwanted plugin rules being enabled (Boshen)

## [0.3.0] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)
- 8d17bb4 linter: --deny all should not enable nursery rules (Boshen)
- c2ad8f8 linter: Implement fixer for `typescript-eslint/consistent-type-definitions` (#3045) (Todor Andonov)
- ae72be1 linter: Remove all ESLint Stylistic rules (Boshen)
- 58d6438 linter: Change no-empty-static-block to `correctness` (Boshen)
- 5cf55c2 linter: No barrel file. (#3030) (Ali Rezvani)
- ae1f15a linter: Support eslint globals (#3038) (Boshen)
- 1a1ba11 linter/tree-shaking: Support `ExportDefaultDeclaration` (#3052) (Wang Wenzhe)

### Bug Fixes

- b88dfd7 linter: Support `-D all -D nursery` (Boshen)
- 598bbba linter: Fix crashing with `unwrap` in import/no-cycle (#3035) (Boshen)

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)

### Refactor

- 53c0ff5 linter: Improve the ergonomics around `ESlintConfig` (#3037) (Boshen)
- 7e4beb0 linter/import/no_cycle: Use ModuleGraphVisitor. (#3064) (Ali Rezvani)

## [0.2.18] - 2024-04-19

### Features

- 2bac1d5 linter: Support `oxlint-disable` alongside `eslint-disable` (#3024) (Boshen)
- fa08abe linter: Remove import/no-unresolved (#3023) (Boshen)
- 9b4e87a linter: Eslint/max-len (#2874) (谭光志)
- df2036e linter: Implement plugin-jsdoc/check-property-names (#2989) (Yuji Sugiura)
- aa62dbb linter: Add missing test cases to no-empty-interface and add config (#2973) (Jose)
- ba2121f linter: Add --jsdoc-plugin flag (#2935) (Yuji Sugiura)
- 395ad76 linter/jsdoc: Update settings.jsdoc method (#3016) (Yuji Sugiura)
- 5d89e75 linter/jsdoc: Implement require-property-(type|name|description) rules (#3013) (Yuji Sugiura)
- 7de9c91 linter/jsdoc: Implement require-property rule (#3011) (Yuji Sugiura)
- ac37d55 linter/tree-shaking: Support DoWhileStatement and IfStatement (#2994) (Wang Wenzhe)
- 5b02ae1 linter/tree-shaking: Support ConditionalExpression (#2965) (Wang Wenzhe)
- da5ea41 linter/tree-shaking: Support Class (#2964) (Wang Wenzhe)

### Bug Fixes

- 627dd42 linter/no-empty-interface: Add missing test (#2979) (Jose)

## [0.2.17] - 2024-04-11

### Features

- 6757dba linter: Eslint-plugin-jest/prefer-lowercase-title (#2911) (cinchen)
- b4b471f linter: Typescript-eslint/consistent-type-definitions (#2885) (Todor Andonov)
- 990eda6 linter/tree-shaking: Support part BinaryExpression (#2922) (Wang Wenzhe)

### Bug Fixes

- 5abbb0c linter: Import/no-cycle ignore type-only imports (#2924) (John Daly)

### Refactor

- 0a77d62 semantic/jsdoc: Rework JSDoc struct for better Span handling (#2917) (Yuji Sugiura)

## [0.2.16] - 2024-04-08

### Features

- acb6eb2 linter: @typescript-eslint/prefer-for-of (#2789) (Denis Gonchar)
- aa63b64 linter: Implement jsdoc/check-access (#2642) (Yuji Sugiura)
- 6823482 linter: Implement jsdoc/empty-tags (#2893) (Yuji Sugiura)
- 7bc638e linter: Eslint-plugin-jest/prefer-mock-promise-sorthand (#2864) (cinchen)
- 6de1b77 linter/import: Add `ignoreTypes` option for the `import/no-cycle` rule (#2905) (John Daly)
- b053d54 linter/tree-shaking: Support try-catch and AwaitExpression (#2902) (Wang Wenzhe)
- 59869d0 linter/tree-shaking: Check `this` in different environment (#2901) (Wang Wenzhe)
- ce34829 linter/tree-shaking: Support ThisExpression and NewExpression (#2890) (Wang Wenzhe)
- 15d08f6 linter/tree-shaking: Support ArrowFunctionExpression (#2883) (Wang Wenzhe)
- 4a86dcb linter/tree-shaking: Support `ArrayExpression` and `ArrayPattern`  (#2882) (Wang Wenzhe)

### Bug Fixes

- 5f8f7f8 ast: `FinallyClause` won't get visited as `BlockStatement` anymore. (#2881) (Ali Rezvani)
- 79e2c95 linter: Handle self closing script tags in astro partial loader (#2017) (#2907) (Kalven Schraut)
- 1cd5e75 linter: Svelte partial loader handle generics (#2875) (#2906) (Kalven Schraut)

## [0.2.15] - 2024-03-30

### Features

- 2365198 cli: Add tsconfig file validation in LintRunner (#2850) (Dunqing)
- d63807e linter: Fallback to the default tsconfig path (#2842) (Dunqing)
- f6391f9 linter: Eslint-plugin-jest/prefer-comparison-matcher (#2806) (cinchen)
- f131442 linter: Eslint-plugin-jest/no-untyped-mock-factory (#2807) (cinchen)
- 451162e linter: Eslint/no-iterator (#2758) (Jose)
- 53ffbc6 linter: Eslint-plugin-react checked-requires-onchange-or-readonly (#2754) (keita hino)
- 1c07a99 linter: Default_param_last (#2756) (Jose)
- 291dc05 linter: No_script_url (#2761) (Jose)
- 76cc906 linter/import: Ignore type-only imports and exports in no_unresolved (#2849) (Dunqing)
- 0cae373 linter/tree-shaking: Pass CallExpression cases (#2839) (Wang Wenzhe)
- fa39fa8 linter/tree-shaking: Check CallExpression when called (#2809) (Wang Wenzhe)
- 3c9e77d linter/tree-shaking: Detect CallExpression in MemberExpression (#2772) (Wang Wenzhe)

### Bug Fixes

- df62828 linter/import: Ignore export declaration in no-duplicates (#2863) (Dunqing)
- c452897 linter/import: False positive for indirect export in namespace (#2862) (Dunqing)
- 64e4de7 linter/max-lines: Only report codes that exceed the line limit (#2778) (Wang Wenzhe)

### Refactor

- 1b5e544 semantic: Distinguish whether requested_modules is type imports/exports (#2848) (Dunqing)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.2.14] - 2024-03-19

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

### Features

- ac813a6 linter: No_template_curly_in_string (#2763) (Jose)
- 134e15e linter: Eslint/no-proto (#2760) (Jose)
- 39b98ba linter: No_eq_null (#2757) (Jose)
- 86d006e linter: Eslint/max-params (#2749) (Jose)
- 22c84c5 linter: Eslint/guard-for-in (#2746) (Jose)
- f5b4599 linter: Eslint/no-ternary (#2744) (Jose)
- 6189985 linter: Eslint/no-continue (#2742) (Jose)
- 91e8a71 linter: Eslint/no-with (#2741) (Andi Pabst)
- 81752b2 linter: Eslint/max-lines (#2739) (Andi Pabst)
- 0623a53 linter: Eslint-plugin-jest: `prefer-to-contain` (#2735) (cinchen)
- 9edda49 linter: Eslint-plugin-jest: `prefer-expect-resolves` (#2703) (cinchen)
- 53a8e7f linter: Add settings.jsdoc (#2706) (Yuji Sugiura)
- f8fe3af linter: Eslint-plugin-jest: prefer-to-be (#2702) (cinchen)
- 265030d linter: Eslint-plugin-jest: prefer-spy-on (#2666) (cinchen)
- 3ae9479 linter: Report side effect for array element in node_side_effects rule (#2683) (Wang Wenzhe)
- 366a879 linter: Resolve ESM star exports (#2682) (Boshen)
- 9b56134 linter: Support check ImportNamespaceSpecifier in no_import_assign (#2617) (Dunqing)
- 7605cd3 linter: Change ban-ts-comment to pedantic (Boshen)
- 95ac265 linter/import: Check ObjectPattern syntax in namespace (#2691) (Dunqing)
- e86cd62 linter/import: Support check reexport binding in namespace (#2678) (Dunqing)
- 4947809 linter/jest: Add new property for `parse_jest_fn` (#2715) (cinchen)
- 2ef4762 linter/tree-shaking: Add cache for checking mutating identifiers (#2743) (Wang Wenzhe)
- 11219d4 linter/tree_shaking: Check assignment of identifier  (#2697) (Wang Wenzhe)
- 57ce737 semantic: Move redeclare varaibles to symbol table (#2614) (Dunqing)
- 8b3de77 span: `impl<'a> PartialEq<str> for Atom<'a>` (#2649) (Boshen)
- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)
- f8e8af2 task: Init eslint-plugin-tree-shaking rule (#2662) (Wang Wenzhe)- 265b2fb Miette v7 (#2465) (Boshen)

### Bug Fixes

- a671d75 linter: Fix guard_for_in span error (#2755) (Jose)
- 09d4c7d linter: Correct example for no-obj-calls rule (#2618) (overlookmotel)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)
- 220eba1 lint: Split files for no_side_effects rule (#2684) (Wang Wenzhe)
- 47e735a linter: Improve the implementation of no_shadow_restricted_names based on symbols (#2615) (Dunqing)
- 240ff19 parser: Improve parsing of `BindingPattern` in TypeScript (#2624) (Boshen)
- 798a6df span: Disallow struct expression constructor for `Span` (#2625) (Boshen)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.2.13] - 2024-03-05

### Features

- 35ce3cc linter: Eslint-plugin-jest: prefer-to-have-length (#2580) (cinchen)
- 212f128 linter: Eslint-plugin-jest: prefer-strict-equal (#2581) (cinchen)
- fe777f3 linter/import: Partial support namespace check (#2538) (Dunqing)

### Bug Fixes

- 951297e linter: Avoid crash if no members in TSTypeLiteral in typescript/prefer-function-type (#2604) (Wenzhe Wang)
- c09c602 linter: Exclude typescript syntax function in only_used_in_recursion (#2595) (Dunqing)
- f00834d linter: Fix getter return rule false positives in TypeScript (#2543) (BlackSoulHub)
- 24d46bc parser: Fix span start for TSModuleDeclaration (#2593) (Arnaud Barré)
- 37de80d semantic: Jsx reference with an incorrect node id (#2546) (Dunqing)- 7cc9013 Broken build from codegen API change (Boshen)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 1391e4a semantic/jsdoc: Misc fixes for JSDoc related things (#2531) (Yuji Sugiura)

## [0.2.12] - 2024-02-28

### Features

- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- 02c82c3 cli,linter: Provide tsconfig path from the cli (#2526) (Boshen)
- d41dcc3 linter: Remove all commonjs logic for import plugin (#2537) (Boshen)

## [0.2.11] - 2024-02-26

### Features

- f5aadc7 linter: Handle cjs `module.exports = {} as default export (#2493) (Boshen)
- f64c7e0 linter: Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492) (Boshen)
- d0a9c46 linter: Handle top-level `require` for import plugin (#2491) (Boshen)
- 696818a linter: Implement @typescript-eslint/prefer-ts-expect-error (#2435) (Alex Yip)
- 6aa8c2d linter: Initialize resolver lazily and automatically read tsconfig.json for now (#2482) (Boshen)
- 135e56a linter: Ignore unsupported extensions in import/no_unresolved (#2481) (Boshen)
- 7f86722 linter: Handle built-in modules in import/no_unresolved (#2479) (Boshen)
- 015b2ee linter: Eslint-plugin-react void-dom-elements-no-children (#2477) (keita hino)
- c5f67fe linter: Add boilerplate for eslint-plugin-import/no_duplicates (#2476) (Boshen)
- f1e364f linter: Eslint-plugin-import/no_unresolved (#2475) (Boshen)
- 2714a32 linter: Continue working on no_cycle (#2471) (Boshen)
- 6527bfd linter: Add boilerplace code for import/namespace,no_deprecated,no_unused_modules (#2470) (Boshen)
- ff6a337 linter: Typescript-eslint: prefer-function-type (#2337) (zhangrunzhao)

### Bug Fixes

- 93742f8 linter: Correct configuration file parsing for jsx-no-useless-fragment (#2512) (keita hino)
- fba66dc linter: Improve import/no-named-as-default (#2494) (Boshen)
- d741d72 linter: Fix import plugin hanging when ignored modules are imported (#2478) (Boshen)
- 35a0f89 linter: Handle cases where createElement is an Identifier in is_create_element_call (#2474) (keita hino)
- bc22ae5 semantic: Refactor jsdoc finding (#2437) (Yuji Sugiura)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)

## [0.2.10] - 2024-02-21

### Features

- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)
- f7e1576 linter: Eslint no-nonoctal-decimal-escape (#2428) (tudorbarbu)

### Bug Fixes

- 5bd2ce6 semantic: Incorrect reference flag for MemberExpression assign (#2433) (Dunqing)

### Refactor

- cd48e1e linter: Simplify getting ImportDefaultSpecifier (#2453) (Dunqing)
- 2a2bb2b linter: Improve implementation of no_dupe_class_members based on ClassTable (#2446) (Dunqing)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.2.9] - 2024-02-18

### Features

- 0b9e122 linter: Implement `unicorn/no-process-exit` rule (#2410) (Yuji Sugiura)
- d9f073e linter: Detect jest file by default glob pattern (#2408) (Wenzhe Wang)
- 8d6202f linter: Eslint-plugin-jest require-to-throw-message (#2384) (keita hino)
- 92afbb1 linter: Eslint-plugin-jest: prefer-equality-matcher (#2358) (cinchen)
- 40e9541 semantic: Add export binding for ExportDefaultDeclarations in module record (#2329) (Dunqing)- 8e0277e Add Typescript ban-tslint-comment (#2371) (tudorbarbu)

### Bug Fixes

- f49ffb2 linter: `getter-return` false positive with TypeScript syntax (#2363) (Boshen)
- ebc08d4 linter: Add missing typescript-eslint(_) prefix for some errors (#2342) (Maurice Nicholson)
- 6a25864 linter/jsx_a11y: Refactor jsx-a11y related utils and its usage (#2389) (Yuji Sugiura)
- 2521b52 linter/jsx_a11y: Ensure plugin settings are used (#2359) (Yuji Sugiura)

### Performance

- 7d7a3fc lint/no_var_requires: Quicker way to check if the `IdentifierReference` point to a global variable (#2376) (Yunfei He)

### Refactor

- 67d7a46 linter: Get arrow expression by scope_id in no_render_return_value (#2424) (Dunqing)
- 63b4741 linter/config: Use serde::Deserialize for config parsing (#2325) (Yuji Sugiura)

## [0.2.8] - 2024-02-06

### Features

- d571839 ast: Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317) (Dunqing)
- a762d17 linter: Promote `no-this-before-super` to correctness (#2313) (Boshen)
- 0060d6a linter: Implement no_this_before_super with cfg (#2254) (Tzvi Melamed)
- f3035f1 semantic: Apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307) (Dunqing)- 8771c64 Add typescript-eslint rule array-type (#2292) (luhc228)

### Bug Fixes

- b5e43fb linter: Fix no_dupe_keys false postive on similar key names (#2291) (Boshen)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.2.7] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- a95a16c linter: Complete custom components setting (#2234) (hjio)
- da3b305 linter: Implement @next/next/no-before-interactive-script-outsi… (#2203) (kaykdm)
- b694a6a linter: Implement @next/next/no-unwanted-polyfillio (#2197) (kaykdm)
- e561457 semantic: Track cfg index per ast node (#2210) (Tzvi Melamed)

### Bug Fixes

- 2beacd3 lexer: Correct the span for irregular whitespaces (#2245) (Boshen)
- 37a2676 linter: AllowFunction doesn't support generator (#2277) (Dunqing)
- f039ad6 linter: Ban `--fix` for variety files(vue, astro, svelte) (#2189) (Wenzhe Wang)
- f32228e linter: Jsx no undef match scope should check with ancestors (#2027) (西了意)
- 73ccf8a oxc_semantic: Proper traversal of try statements (#2250) (Tzvi Melamed)

### Refactor

- 1de3518 linter: Remove Regex and change error position (#2188) (Wenzhe Wang)- 650f6c9 Use our forked version of miette::Reporter for tests (#2266) (Boshen)- 87b9978 Move all miette usages to `oxc_diagnostics` (Boshen)

## [0.2.6] - 2024-01-26

### Features

- ee5b968 linter: Support read env from eslintrc (#2130) (fi3ework)
- 8898377 semantic: Cfg prototype (#2019) (Boshen)
- 2794064 transfrom: Transform-json-strings (#2168) (underfin)

### Bug Fixes

- 2e3153e linter: Rename react_perf/jsx_no_new_function_as_props to jsx_no_new_function_as_prop (#2175) (Yuji Sugiura)

### Refactor

- a17e43e linter: Move settings and env to the config module (#2181) (Boshen)

## [0.2.5] - 2024-01-25

### Features

- ac1d318 linter: Eslint-plugin-jest: prefer-called-with (#2163) (cin)
- 3891430 linter: Eslint: no-void (#2162) (cin)

### Bug Fixes

- 989ab88 codegen: Print `Directive` original string (#2157) (underfin)
- c18619e linter: Use correct rule name (#2169) (Yuji Sugiura)
- 2602232 linter: Explicit-length-check inside ternary (#2165) (Maurice Nicholson)

## [0.2.3] - 2024-01-23

### Features

- 6d808a6 linter: Linter-eslint-plugin-import/no-named-as-default (#2109) (Valerii Smirnov)
- d90db3a linter: Promote no-new-array to correctness with better help message (#2123) (Boshen)
- 69fecac linter: Eslint config jsonc support (#2121) (Boshen)
- 5ca07bc linter: Eslint-plugin-react-perf (#2086) (Hulk)
- b160842 linter: Support eslint config in nextjs eslint (#2107) (kaykdm)
- 16b3261 linter: Eslint-plugin-jest: no-restricted-jest-methods (#2091) (cin)- 20a34b5 Introduce --react-perf-plugin CLI flag, update rules to correctness (#2119) (Hulk)- 4adce6f (eslint-plugin-jest): no-restricted-matchers (#2090) (cin)

### Bug Fixes

- d00c44c linter: Allow `[...new Array(n)]` in no-useless-spread (#2124) (Boshen)
- 2228aa8 linter: Jsx_a11y/img-redundant linter enable test case(#2112) (msdlisper)
- 142f84f linter: Not use `new_inline` with flexible str (#2106) (Wenzhe Wang)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)
- 8bccdab semantic: Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114) (Dunqing)

## [0.2.2] - 2024-01-20

### Features

- 721a869 linter: Improve no_redeclare rule implementation (#2084) (Dunqing)- 2f1e1e2 Expose linter RULES and use it for listing (#2083) (Yuji Sugiura)

### Bug Fixes

- d7ecd21 linter: Eslint-plugin-import no-named-as-default-member rule (#2071) (Valerii Smirnov)
- 3faa2aa linter: S/consistent-type-export/consistent-type-exports (#2065) (Boshen)

### Refactor

- a368134 linter: Perfect the scope linter (#2092) (msdlisper)

## [0.2.1] - 2024-01-16

### Features

- 9e06bd7 linter: Remove the `--timings` feature (#2049) (Boshen)
- c60c315 linter: Eslint-plugin-import no-named-as-default-member rule (#1988) (Valerii Smirnov)
- 530d1be linter: Eslint-plugin-jsx-a11y no-redundant-roles rule (#1981) (Yuto Yoshino)
- 198f0e5 linter: Eslint-plugin-jsx-a11y aria-activedescendant-has-tabindex (#2012) (keita hino)
- a356918 linter: Eslint-plugin-next: no-document-import-in-page (#1997) (kaykdm)
- c70a065 linter: Eslint-plugin-next: no-head-element (#2006) (kaykdm)
- 8f0f824 linter: Eslint-plugin-next:  no-typos (#1978) (kaykdm)
- 04540f7 linter: Eslint-plugin-jsx-a11y click-events-have-key-events (#1976) (Yuji Sugiura)

### Bug Fixes

- 3b40fbd linter: False positive for filename_case where filename doesn't have a proper casing (#2032) (Boshen)
- 68606c4 linter: Keep rules disabled if the rule is not enabled in the config (#2031) (Boshen)
- 107a32e linter: Fix false positive for `erasing-op` in `0/0` case (#2009) (Cameron)

### Refactor

- f514410 linter: Move `LintSettings` to its own file (#2052) (Boshen)
- ae4e714 linter: Remove the `LintSettings` parameter from `LintContext::new`. (#2051) (Boshen)
- b386177 linter: Move away from tuples for test cases (#2011) (Boshen)

## [0.2.0] - 2024-01-12

### Features

- d51c9f1 linter: Eslint-plugin-jest: no-test-return-statement (#1979) (cin)
- fb5d0a7 linter: Add support for same rule name but different plugin names (#1992) (Boshen)
- b7ea4e5 linter: Support vue generic component (#1989) (Boshen)
- c5887bc linter: Implement @typescript-eslint/triple-slash-reference (#1903) (kaykdm)
- ac704cc linter: Eslint-plugin-jsx-a11y autocomplete-valid (#1901) (Yuto Yoshino)
- 40dbfae linter: Eslint-plugin-react: no-direct-mutation-state (#1892) (zhangrunzhao)
- 856b9a5 linter: Support overriding oxlint rules by eslint config (#1966) (Boshen)
- 7891670 linter: Eslint-plugin-react: require-render-return (#1946) (kaykdm)
- 2b7ca59 linter: Eslint-plugin-jsx-a11y role-has-required-aria-props (#1881) (Yuto Yoshino)
- f6047b6 linter: Eslint-plugin-jsx-a11y role-support-aria-props (#1961) (Rintaro Itokawa)
- fd5856e linter: Eslint-plugin-jsx-a11y role-support-aria-props (#1949) (Rintaro Itokawa)
- c6eb519 linter: Eslint-plugin-react: no-unknown-property (#1875) (Valerii Smirnov)
- fe48bfa lsp: Support vue, astro and svelte (#1923) (IWANABETHATGUY)- ac3b44b Nextjs plugin (#1948) (Cameron)

### Bug Fixes

- e0da12a linter: Allow eslintrc to add rule when overriding (#1984) (fi3ework)
- d4acd14 linter: Jsx-key: handle anonymous functional components in arrays that have a function body (#1983) (Maurice Nicholson)
- b5f4f1e linter: Fix plugin name parsing when reading config file (#1972) (Hao Cheng)
- 8d9894a linter: Support cases where aria-hidden includes expressions (#1964) (keita hino)
- 66e95a5 linter: Change severity of no-sparse-arrays to warnings (Boshen)

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)
- 64310fa linter: Remove duplicate `get_jsx_attribute_name` (#1971) (Cameron)

## [0.1.2] - 2024-01-06

### Features

- c2e8ef5 linter: Disable no-unused-labels for svelte (#1919) (Boshen)
- f2ed83c linter: <script> part of svelte file (#1918) (Boshen)

### Bug Fixes

- bb6128b linter: Change no-var to restriction (Boshen)

### Refactor

- 450791d linter: Rename *_partial_loader files (#1916) (Boshen)

## [0.1.1] - 2024-01-06

### Features

- 8f27a98 cli: Support walk vue and astro (#1745) (Wenzhe Wang)
- 0feeac5 lint: Add partial loader register (#1760) (Wenzhe Wang)
- 55a87b2 linter: Eslint: no-var (#1890) (zhangrunzhao)
- 497a207 linter: Parse two script tags from vue (#1899) (Boshen)
- 8a3eff1 linter: Parse multiple script tags in astro file (#1898) (Boshen)
- 4c5c61e linter: Add support for multiple script tags from vue and stro (#1897) (Boshen)
- 5f29e3f linter: No irregular whitespace (#1877) (Deivid Almeida)
- a63490c linter: Support astro front matter `---` block (#1893) (Boshen)
- 11ca5c2 linter: Do not lint when vue file has no js section (#1891) (Boshen)
- 8a1e894 linter: Eslint-plugin-jsx-a11y prefer-tag-over-role (#1831) (Yuto Yoshino)
- 2ac2630 linter: Eslint-plugin-jsx-a11y mouse-events-have-key-events (correctness) (#1867) (Ken-HH24)
- 3d41637 linter: Add Vue loader (#1814) (Wenzhe Wang)
- ba0a4a8 linter: Eslint-plugin-react: jsx-no-undef (#1862) (Valerii Smirnov)
- af61894 linter: Eslint plugin jsx a11y: aria-role (#1849) (msdlisper)
- 4c1673c linter: Use settings for eslint-plugin-jsx-a11y/html_has_lang (#1843) (msdlisper)
- f45a3cc linter: Support eslint/no-unused-private-class-members rule (#1820) (Dunqing)
- 98895ca linter: Eslint-plugin-jsx-a11y media-has-caption (#1822) (poteboy)
- 0280e06 linter: Refine test for no-distracting-elements (#1824) (Tapan Prakash)
- f0ad356 linter: Refine jsx-a11y settings (#1816) (msdlisper)
- d984d59 linter: Eslint-plugin-jsx-a11y lang (#1812) (msdlisper)

### Bug Fixes

- 24d209c linter: Fix vue parser not working for multiple scripts after <template> (#1904) (Boshen)
- 4ea6e5d linter: Do not check empty file for vue / astro files (#1900) (Boshen)
- b2a62dd linter: Error rule config in media_has_caption (#1864) (msdlisper)
- 1ddbe8f linter: Unexpected unwrap panic (#1856) (msdlisper)
- d803a3a linter: Ignore false positives in eslint-plugin-react(jsx-key) (#1858) (Maurice Nicholson)

### Performance

- dae5f62 semantic: Check duplicate parameters in Binder of FormalParameters (#1840) (Dunqing)

### Refactor

- 4515644 linter: Get js code slice from vue source code (#1876) (Wenzhe Wang)
- 040278a linter: Extract common code (#1848) (msdlisper)
- 0db2b84 linter: Simplify Parent Node Access in MediaHasCaption Rule (#1829) (poteboy)
- 6c5b22f semantic: Improve ClassTable implmention and merge properties and methods to elements (#1902) (Dunqing)

## [0.0.22] - 2023-12-25

### Features

- d41e3fd ast: Enter/leave ClassBody and PrivateInExpression (#1792) (Dunqing)
- b4138aa linter: Change double-comparisons to correctness (Boshen)
- 32413f5 linter: Eslint-plugin-jsx-a11y aria-props (#1797) (poteboy)
- bf527f4 linter: Eslint-plugin-jsx-a11y no-aria-hidden-on-focusable (#1795) (poteboy)
- b8a90c1 linter: Eslint-plugin-jsx-a11y no-distracting-elements rule (#1767) (Tapan Prakash)
- f8b386e linter: Correct example and docs url for number_arg_out_of_range (#1737) (legend80s)
- 117f44c linter/eslint/no-cond-assign: Span points to the operator (#1739) (Dunqing)
- 521aa2c linter/eslint/no-useless-escape: Support auto fix (#1743) (Dunqing)

### Bug Fixes

- b25f014 linter: Fix a typo in no_redeclare message (#1789) (Milo)
- 0bf7596 linter: Support read the third item in config file (#1771) (Wenzhe Wang)
- 2286181 linter: Update snapshots (Boshen)
- 5d7ea9d linter: Change non-error lints to warning (Boshen)
- 6d42022 linter: Improve the help message for const-comparisons (#1764) (Boshen)
- 0e563b3 linter: Fix missing ` in the help message for const-comparisons (Boshen)
- 8e6004f linter/eslint/no-obj-calls: Correctly resolves the binding name (#1738) (Dunqing)

### Performance

- 51a243b linter: Reduce the `RuleEnum` enum size from 168 to 16 bytes (#1783) (Boshen)
- 2e707bc linter: Use simd (memchr) for no-useless-escape search (#1766) (Boshen)
- d0cc3ec linter: Change regex to static in no_commented_out_tests (Boshen)
- e741b8f linter: Precompute `rule.name()` (#1759) (Boshen)

### Documentation

- 18f0e20 linter: Update comments (#1779) (Wenzhe Wang)

### Refactor

- 2d3ac95 linter: Shrink the error span for require_yield (Boshen)
- 38cb487 linter: Explain no-empty-pattern (Boshen)

## [0.0.21] - 2023-12-18

### Features

- 621a943 linter: Eslint-plugin-jsx-a11y no-access-key (correctness) for  (#1708) (yoshi2no)
- 774a257 linter: Eslint-plugin-unicorn no-null(style) (#1705) (Ken-HH24)
- 6a90cd4 linter: Add  jsx-a11y settings (#1668) (msdlisper)
- cf0793b linter: `tabindex-no-positive` for eslint-plugin-jsx-a11y (#1677) (yoshi2no)
- c9589b5 linter: Eslint-plugin-unicorn/prefer-prototype-methods (#1660) (Hao Cheng)
- 90524c8 linter: Add eslint-plugin-import(export) rule (#1654) (Wenzhe Wang)
- 282771a linter: Eslint-plugin-unicorn prefer-dom-node-text-content(style) (#1658) (Ken-HH24)

### Bug Fixes

- d101acf linter: Prefer-string-starts-ends-with: ignore `i` and `m` modifiers. (#1688) (Andy Armstrong)
- 0a8746c linter: Panic in prefer string starts, ends with (#1684) (Cameron)
- e5752a5 linter: Fix excape_case panicing on unicode strings (#1673) (RiESAEX)

### Performance

- 0080638 linter/react: Find class node by symbols in get_parent_es6_component (#1657) (Dunqing)

### Refactor

- 8f296df linter: Use fxHashMap in jsx-a11y settings (#1707) (msdlisper)
- d719af4 linter: Make some jest rules report more detailed (#1666) (Wenzhe Wang)- 0d7e166 Use `new_without_config` for `jsx_key` (#1685) (Cameron)

## [0.0.20] - 2023-12-13

### Features

- b425b73 linter: Eslint-plugin-unicorn prefer-modern-dom-apis(style) (#1646) (Ken-HH24)

### Bug Fixes

- 117d95f linter: Improve the span message for no-accumulating-spread (Boshen)- ef740c3 Improve span for no accumulating spread (#1644) (Cameron)- 65c0772 Remove escapes in no array reduce test cases (#1647) (Cameron)- a09060b Remove escapes in prefer regexp test test cases (#1645) (Cameron)

### Refactor

- 4ced3f9 linter: Separate out the category in the output of `--rules` (Boshen)

## [0.0.19] - 2023-12-08

### Features

- d88f4f4 linter: Eslint-plugin-jsx-a11y no-autofocus  (#1641) (msdlisper)
- ddb3c62 linter: Eslint-plugin-jsx-a11y scope rule (correctness) (#1609) (Shinobu Hayashi)
- 795db7c linter: Cxc: no accumulating spread (#1607) (Don Isaac)
- c8e2ef6 linter: Eslint-plugin-unicorn: explicit-length-check (#1617) (RiESAEX)
- 519b5f2 linter: Eslint-plugin-unicorn prefer-reflect-apply(style) (#1628) (Ken-HH24)
- 32504ca linter: Add a `perf` category (#1625) (Boshen)
- b573036 linter: Eslint-plugin-jsx-a11y iframe-has-title rule (correctness) (#1589) (Shinobu Hayashi)
- 967aa35 linter: Eslint-plugin-unicorn require-array-join-separator(style) (#1608) (Ken-HH24)
- ba5b13d linter: Eslint-plugin-unicorn no-unreadable-array-destructuring (style) (#1594) (Ken-HH24)
- a614255 linter: Eslint-plugin-jsx-a11y  img-redundant-alt (correctness) (#1571) (Shinobu Hayashi)
- 8bef1f1 linter: Eslint-plugin-unicorn numeric-separators-style (style) (#1490) (Jon Surrell)
- 72dd72b linter: Eslint-plugin-unicorn/no-unreadable-iife (#1572) (Hao Cheng)
- 3b2b6a0 linter: Eslint-plugin-unicorn no-await-expression-member (style) (#1569) (Ken-HH24)
- afeed17 linter: Eslint-lugin-unicorn no_useless_length_check (#1541) (Radu Baston)
- a2510be linter: No-is-mounted for eslint-plugin-react (#1550) (Ken-HH24)
- 59d0428 linter: Eslint 9.0 no empty static block (#1543) (Radu Baston)
- 8afda7d linter: Eslint-plugin-unicorn: escape-case (#1495) (RiESAEX)
- ebf5cf8 linter: Heading-has-content for eslint-plugin-jsx-a11y (#1501) (Ken-HH24)
- 7930f90 linter: Eslint-plugin-unicorn prefer-set-size (correctness) (#1508) (Cameron)
- 0dd5ec1 linter: Eslint-plugin-unicorn prefer-native-coercion-functions (pedantic) (#1507) (Cameron)
- c8cc814 linter: Eslint-plugin-jsx-a11y anchor_is_valid (correctness) (#1477) (msdlisper)- 872e8ad Eslint-plugin-unicorn (recommended) prefer-node-protocol (#1618) (IWANABETHATGUY)

### Bug Fixes

- b973261 linter: Improve the key span for jsx-key (Boshen)

### Refactor
- b5f8a65 Improve pattern match of prefer-reflect-apply (#1630) (IWANABETHATGUY)

## [0.0.18] - 2023-11-22

### Features

- 8b0032d linter: Eslint plugin unicorn: no useless switch case (#1463) (Cameron)
- 822ce76 linter: Html-has-lang for eslint-plugin-jsx-a11y (#1436) (Ken-HH24)
- 2ba69f1 linter: `anchor-has-content` for eslint-plugin-jsx-a11y (zhangpeng)
- 5543e8c linter: Eslint-plugin-unicorn/no-nested-ternary (#1417) (Hao Cheng)
- b72f83b linter: For-direction rule add check for condition in reverse o… (#1418) (Angelo Annunziata)
- 98279fc linter: Eslint-plugin-unicorn: no-hex-escape (#1410) (RiESAEX)
- 3df810b linter: Eslint-plugin-jest: no-deprecated-function (#1316) (cin)
- 5ade895 linter: Add rule `eslint(no_regex_spaces)` (#1129) (Iván Ovejero)
- 71b3f48 linter: Eslint-plugin-unicorn/number-literal-case (#1271) (Hao Cheng)
- 6ac5403 linter: Eslint-plugin-jest/max_expects (#1239) (cin)
- 18a3525 linter: Reimplement eslint-plugin-jest(no-identical-title) (#1229) (Wenzhe Wang)
- 82c1769 linter: Eslint-plugin-unicorn/no-abusive-eslint-disable (#1125) (Hao Cheng)
- 0218ae8 prettier: Print leading comments with newlines (#1434) (Boshen)

### Bug Fixes

- 5a4e611 linter: Detect assert function in Await Expression (#1202) (Wenzhe Wang)

### Refactor

- f775488 lint: Replace `parse_jest_fn_*` methods in eslint-plugin-jest(no-standalone-expect) rule (#1231) (Wenzhe Wang)
- 071311a lint: Migrate eslint-plugin-jest(expec-expect) (#1225) (Wenzhe Wang)
- dfc2c6a linter: Replace the old parse_expect_jest_fn.rs file (#1267) (Wenzhe Wang)
- f3788ee linter: Remove all old `parse_expect_jest_fn_call` (#1259) (Wenzhe Wang)
- 1dd321e linter: Remove all old `parse_general_jest_fn_call` in jest rules (#1232) (Wenzhe Wang)
- 72b3bdf linter: Replace all `is_type_of_jest_fn_call` (#1228) (Wenzhe Wang)
- efc346e linter: Migrate eslint-plugin-jest(no-alias-method) (#1226) (Wenzhe Wang)
- b4ce2b5 linter: Remove unused logic in `resolve_to_jest_fn` (#1208) (Wenzhe Wang)
- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.0.17] - 2023-11-09

### Bug Fixes

- 49b6be6 linter: Fix handling of repeated eslint-disable comments (#1200) (Hao Cheng)

## [0.0.16] - 2023-11-08

### Features

- 6e76669 lint: Remove unnecessary check (#1185) (cin)
- 9369424 linter: Eslint-plugin-jest: no-hooks (#1172) (cin)
- 033a112 linter: Support eslint(default-case-last) (#1156) (Yiming Pan)
- a5b87c4 linter: Eslint-plugin-unicorn no-object-as-default-parameter (#1162) (Cameron)
- 85651af linter: Jest/prefer-todo rule (#1065) (cin)
- 1aa4b4e linter: Add  rule `eslint-plugin-jsx-a11y(alt-text)` (#1126) (Trevor Manz)- 4e9260e Basic enable plugin (#1154) (Wenzhe Wang)

### Bug Fixes

- fa4e0ca linter: Fix covered span of eslint-disable-next-line comments (#1128) (Hao Cheng)
- 278a1d6 linter/jsx_key: Ignore ObjectProterty nodes (#1139) (Wei Zhu)

### Refactor

- 1cc449f linter: Reduce the lookup times of Call Expression in Jest rules (#1184) (Wenzhe Wang)- 3b88f74 Change jest rule's category (#1155) (Wenzhe Wang)- e9b88e0 Split parse_jest_fn_call (#1152) (Wenzhe Wang)

## [0.0.15] - 2023-10-30

### Features

- 407e406 linter: Change some rules pedantic and improve help message (#1112) (Boshen)
- 8b11592 linter: Demote prefer_array_flat_map to style (#1108) (Boshen)
- d4c05ff linter: Support unicorn/prefer-query-selector (#1068) (Dunqing)
- 0a0e93b linter: Eslint-plugin-unicorn require-number-to-fixed-digits-argument (#1073) (Mariusz Antas)
- 162c720 linter: Eslint-plugin-unicorn  switch-case-braces (#1054) (Mariusz Antas)
- 47837e5 linter: Support react/no-string-refs (#1055) (Dunqing)
- db83f66 linter: Eslint-plugin-unicorn - no-empty-file (#1052) (Cameron)
- ebab50e linter: Eslint-plugin-react no-string-refs (#1053) (Trevor Manz)
- d8f07ca linter: Support react/no-render-return-value (#1042) (Dunqing)
- 64988f4 linter: Eslint-plugin-react(no-unescaped-entities) (#1044) (Cameron)
- a2e40ef linter: Eslint-plugin-react/no-find-dom-node (#1031) (Hao Cheng)
- b703c0c linter/no_children_prop: Point the span to "children" (#1106) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- b4739e5 linter: Fix panic in no_unescaped_entities (#1103) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)
- bd34dc7 linter: Fix panic on no_mixed_operators rule (#1094) (Cameron)
- 59660a5 linter: NoTemplateLiterals configuration in no_string_refs rule not working (#1063) (Dunqing)
- 144c881 linter/no-render-return-value: Remove duplicate test case (#1111) (Dunqing)
- 22c31ce linter/no_empty_file: Point to start of file instead of the entire file (#1105) (Boshen)
- 4975440 linter/no_render_return_value: Fix false positive when nested inside an arrow expression (#1109) (Boshen)

## [0.0.14] - 2023-10-23

### Features

- dea9b7c linter: Eslint-plugin-react: jsx-no-duplicate-props (#1024) (Cameron)
- 25247e3 linter: Eslint/no-fallthrough (nursery) (Sg)
- f13fc22 linter: Eslint-plugin-react/no-useless-fragment (#1021) (Cameron)
- 88cf98a linter: Eslint-plugin-unicorn(throw-new-error) (#1005) (Cameron)
- 952139c linter: Eslint-plugin-unicorn(prefer-array-flat-map) (#997) (Cameron)
- 41c55bc linter: Eslint-plugin-unicorn no console spaces (#991) (Cameron)
- eaa0c58 linter: Eslint-plugin-unicorn(filename-case) (#978) (Boshen)
- 205f66b linter: Add `jest/no-confusing-set-timeout` (#938) (cin)
- 90828c4 linter: Add `eslint(jest/valid-title)` rule (#966) (Wenzhe Wang)
- 7a62d4b linter: Add `jest/no-identical-title` rule (#957) (Wenzhe Wang)
- 812baeb linter: Add `eslint(jest/valid-expect)` rule (#941) (Wenzhe Wang)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)

### Bug Fixes

- b6b6853 linter: Point to the opening fragment for jsx_no_useless_fragment (Boshen)
- ee134f0 linter: Incorrect reporting for jsx_key (#1020) (Cameron)
- c95f2e0 linter: Fix panic with `strip_prefix` (#1013) (Boshen)
- a710e73 linter: Fix clippy (Boshen)

### Refactor

- db5417f clippy: Allow clippy::too_many_lines (Boshen)
- eaeb630 clippy: Allow struct_excessive_bools (Boshen)

## [0.0.13] - 2023-09-29

### Features

- 982ae9b linter: Improve help message of no-thenable (Boshen)
- 2453954 linter: Add no-redeclare rule. (#683) (cin)
- d700cf8 linter: Add eslint(jest/no-standalone-expect) (#931) (Wenzhe Wang)
- e2a4927 linter: Add eslint(jest/no-export) (#925) (Wenzhe Wang)
- eec9fd4 linter: Add eslint(jest/no-mocks-import) (#924) (Wenzhe Wang)
- 4bf329e linter: Implement eslint-plugin-unicorn/no-thenable rule (#910) (Devin-Yeung)
- 9178451 linter: Add eslint-plugin-jest/no-jasmine-globals (#914) (Wenzhe Wang)
- dd74d93 linter: Add no-console rule (#887) (Todor Andonov)
- 8c44a9e linter: Add eslint-plugin-import/default (#895) (Wenzhe Wang)
- f488e1f linter: Eslint-plugin-import(no-cycle) (#890) (Boshen)
- 35e1898 linter: Add typescript/no-explicit-any (#881) (Don Isaac)
- 4e5f63a linter: Implement re-exports (#877) (Boshen)
- 1b8e2c0 linter: Add eslint-plugin-import/named (Boshen)
- a358856 linter: Add current_working_directory to tester (Boshen)
- 9ee7593 linter: Add rule_path to tester so the file extension can be changed (Boshen)
- 9679ca7 linter: Add `eslint-plugin-jest/no-done-callback` rule (#846) (Wenzhe Wang)
- ee54575 linter: Add runner for import-plugin (#858) (Boshen)
- 75d928a syntax: Add loaded_modules to ModuleRecord (Boshen)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)- f93c861 Add jest/no-interpolation-in-snapshots rule (#867) (Wenzhe Wang)

### Bug Fixes

- 85b113d linter: Improve error span for no-thenable (Boshen)

### Performance

- 9c46e7e linter: Early bail out if not jest fn (#885) (Wenzhe Wang)
- babbc47 parser: Lazily build trivia map instead of build in-place (#903) (Boshen)

## [0.0.12] - 2023-09-06

### Features

- 286049b linter: Implement unicorn/no-unnecessary-await (#856) (Devin-Yeung)
- fa1d7da linter: Add eslint-plugin-jest/no-conditional-expect rule (#832) (Wenzhe Wang)
- 7233aef linter: Add eslint-plugin-jest/no_alias_method rule (#818) (Wenzhe Wang)
- 3721837 linter: Eslint-plugin-jest/expect-expect (#802) (Wenzhe Wang)
- a44dde5 linter_plugin: Add linter plugin crate (#798) (u9g)

### Bug Fixes

- b89e931 cli: Spawn linting in another thread so diagnostics can be printed immediately (Boshen)
- 2b60b8a linter: Fix incorrect behaviour for "-D correctness -A rule-name" (Boshen)
- 8b24052 linter: No-var-requires not warning if has bindings in ancestors (#799) (Wenzhe Wang)

### Performance

- a969f69 linter: Parse ts-directive manually (#845) (Devin-Yeung)
- 6f270f1 linter: Swap the order of checks for no_caller (#844) (Boshen)

### Refactor

- 6706541 linter: Remove complicated linter service setup (Boshen)
- 91f4896 linter: Clean up Test a bit (Boshen)
- 6931451 linter: Less a global hashmap to reduce rule timer macro expansion (#822) (Boshen)

## [0.0.11] - 2023-08-27

### Features

- 741aa8d ast: Add to ChainExpression and ExpressionArrayElement to ASTKind (#785) (u9g)
- 5921375 cli: Use insta_cmd for cli snapshot testing (#791) (Boshen)
- fd2f8fb linter: Detect import (#778) (Wenzhe Wang)
- 9c50bc0 linter: Implement no-unsafe-declaration-merging (#748) (Makoto Tateno)

### Bug Fixes

- 33ea858 linter: Show the escaped span for no-useless-escape (#790) (Boshen)

### Refactor

- ed9e3e0 linter: Move the label message to help (Boshen)
- 31d5669 linter: Extract `is_valid_jest_call` (#781) (Wenzhe Wang)
- 29f8c02 linter: Clean up tester with fixes (#773) (Boshen)

## [0.0.10] - 2023-08-21

### Bug Fixes

- 58d2d1e cli: Fix a race condition where the program will hang (Boshen)

## [0.0.8] - 2023-08-21

### Features

- 2fde225 linter: Implement eslint-plugin-unicorn/no-instanceof-array (#752) (Kei Sakamoto)
- 3022655 linter: Add no-commented-out-tests (#723) (Wenzhe Wang)
- 607fa6a linter: Implement typescript-eslint/ban-ts-comment (#741) (Devin-Yeung)
- 6f00461 linter: Implement @eslint/no-shadow-restricted-names (#617) (#728) (Lqxc)
- 4f5e4c1 linter: Implement @typescript-eslint/no-duplicate-enum-values (#726) (Kei Sakamoto)
- 0c64517 linter: Valid-describe-callback(eslint-plugin-jest) (#706) (Wenzhe Wang)
- 3adca1c linter: Implement @typescript-eslint/prefer-as-const (#707) (Kei Sakamoto)
- f8358a1 linter: @typescript-eslint/no-namespace (#703) (Alexandr Metreniuc)
- c6245f8 linter: Implement `no-undef` (#672) (Makoto Tateno)
- d1531cd linter: Add no-extra-boolean-cast rule (#677) (Alexandr Metreniuc)
- e7d8d4b linter: Enable module record builder (Don Isaac)
- 3cf08a2 linter: No-focused-test(eslint-jest-plugin) (#609) (Wenzhe Wang)
- 9714e46 resolver: Add tracing (#710) (Boshen)- 8a915ce Vscode extension (#690) (阿良仔)

### Bug Fixes

- de7735d cli: Fix race condition when resolving paths (Boshen)
- ec85fd8 eslint/no-obj-calls: Should resolve non-global binding correctly (#745) (Yunfei He)
- 6eca2ad linter: Change severity of no-obj-calls to warning (Boshen)
- 4032e47 linter: Improve error and help message on no-duplicate-enum-values (Boshen)
- a1c2fa6 linter: Improve help message on no-namespace (Boshen)
- e5d7618 linter: Reduce the span of no-namespace to the keyword (Boshen)
- 77bc913 linter: No-extra-boolean-cast false positive (Boshen)
- a7a834a linter: Fix some race conditions (Boshen)
- ba8dbf5 linter: Fix false positives in loss-of-precision lint (#664) (Devin-Yeung)
- 2f48bdf parser,semantic: Make semantic own `Trivias` (#711) (Boshen)

### Refactor

- 1fdce7e cli: Split out group options (#760) (Boshen)
- 6f1daa6 cli: Clean up lint and cli options (#759) (Boshen)
- 772f71f cli: Add WalkOptions for walk logic (#757) (Boshen)
- a9a6bb8 cli,linter: Move path processing logic from cli to linter (#766) (Boshen)
- 324acfc cli,linter: Move the lint runner from cli to linter (#764) (Boshen)
- 3110490 cli,linter: Move LintOptions from cli to linter (#753) (Boshen)
- 98e4240 linter: Manually declare lint rules because `cargo fmt` breaks (#671) (Boshen)

## [0.0.7] - 2023-07-29

### Features

- 21f8abe cli: Add support for `TIMING` env var (#535) (Shannon Rothe)
- 1bc564e linter: Add style category and change no-empty-interface to style (Boshen)
- 72afdf6 linter: Eslint/no-loss-of-precision (#649) (Devin-Yeung)
- 786cf82 linter: Implement no-global-assign (#624) (SoonIter)
- 252d334 linter: Add a `run_once` callback (#647) (Boshen)
- 453edd2 linter: Eslint/no-empty-character-class (#635) (Sg)
- 52c3c37 linter: Implement no-var-requires (#575) (Makoto Tateno)
- 836d430 linter: Implement `adjacent-overload-signature` (#578) (阿良仔)
- 4b566c0 linter: Implement `no-test-prefixes` (#531) (阿良仔)
- 0346adb linter: Add eslint/no-control-regex (#516) (Don Isaac)
- 5d67094 linter: Implement eslint rule `no-return-await` (#529) (vagusX)
- 1b6fa7b linter: No disabled tests(eslint-jest-plugin) (#507) (Wenzhe Wang)
- 1aaeb79 linter: Implement `no-misused-new` (#525) (阿良仔)- 8fdb6b6 Add eslint/no-obj-calls (#508) (Don Isaac)

### Bug Fixes

- f5c9908 linter: Improve the span for no-inner-declarations (Boshen)
- e4020d6 linter: Change no-control-regex to severity warning (Boshen)
- ad51157 linter: Make disable directives work with plugin rule names (Boshen)
- 06aac50 linter: Change no-var-requires to severity warning (Boshen)
- 6089898 linter: Change severity of `no-this-alias` from error to warning (Boshen)- 357b28e No return await error (#539) (vagusX)

### Performance

- 6628fc8 linter: Reduce mallocs (#654) (Don Isaac)

### Refactor

- d8bfe14 linter: Remove `Box::leak` (#641) (阿良仔)
- 2f8b3f8 linter: Run eq_eq_eq fix in some condition (#545) (Wenzhe Wang)
- cdaff8b linter: Expose LintContext as the API for Linter::run (Boshen)
- 87e65ac semantic: Symbol declarations and references (#594) (Don Isaac)- 318d558 Format code (Matthew "strager" Glazar)- ad00720 Avoid unstable let_chains (Matthew "strager" Glazar)- fbb8aa3 Remove unstable feature const_trait_impl & const_slice_index & slice_group_by (#629) (Sg)

## [0.0.6] - 2023-07-01

### Features

- a2809bf linter: Implement @typescript-eslint/no-unnecessary-type-constraint (Boshen)
- 2e6cb6d linter: Implement @typescript-eslint/no-empty-interface (Boshen)
- 83f69b1 linter: Implement @typescript-eslint/no-non-null-asserted-optional-chain (Boshen)
- 7f300f8 linter: Implement @typescript-eslint/no-extra-non-null-assertion (Boshen)

## [0.0.5] - 2023-07-01

### Features

- 4085a95 linter: Implement no-prototype-builtins (Boshen)
- 9250811 linter: Implement no-useless-escape (Boshen)
- 1aa1129 linter: Implement no-inner-declarations (Boshen)
- c5402c1 linter: Implement no-import-assign (nursery) (Boshen)
- 2f8f974 linter: Implement no-dupe-else-if (Boshen)
- bb9838a linter: Implement no-cond-assign (Boshen)
- 9d16e68 linter: Implement no-self-assign (Boshen)
- 230f1d3 linter: Implement no-unsafe-finally (Boshen)
- 553af9b linter: Implement no-unsafe-optional-chaining (Boshen)
- af3ae9b linter: Implement no-useless-catch (Boshen)

### Bug Fixes

- f9aeebd linter: Fix no_useless_escape crashing on unicode boundaries (Boshen)
- ecdd7bc linter: Fix error message for no_dupe_keys (Boshen)

### Refactor

- 3ec2974 linter: Improve span for no-case-declarations (Boshen)
- 3b22992 linter: Remove redundant backticks from no-constant-binary-expression's error message (Boshen)

## [0.0.4] - 2023-06-28

### Features

- 2be637c linter: Implement no_sparse_arrays (Boshen)
- 87a9ce8 linter: Implement `no-ex-assign` (#495) (阿良仔)
- 2ad1339 linter: Implement require_yield (Boshen)
- a93a876 linter: Implement no_delete_var (Boshen)
- be2200d linter: Implement `no-case-declarations` (#491) (阿良仔)

### Bug Fixes

- 32bffa3 linter: Fix disable directives not working for no_func_assign (Boshen)
- 715698f linter: S/no_function_assign/no_func_assign per eslint (Boshen)
- 072abcc linter: Fix no_empty_pattern broken on rest elements (Boshen)

