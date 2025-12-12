# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.32.0] - 2025-12-08

### 🚀 Features

- 5c62c16 linter/plugins: Allow JS plugins to access `globals` (#16512) (Arsh)
- 7584938 linter/plugins: Add ESLint compat mode to `RuleTester` (#16538) (overlookmotel)
- b38c2d0 linter/plugins: Add `SourceCode#lineStartIndices` getter (#16510) (overlookmotel)
- ba93ffc linter/plugins: Add `SourceCode#tokensAndComments` getter (#16503) (overlookmotel)
- d2ca540 linter/plugins: Add `SourceCode#isESTree` property (#16499) (overlookmotel)
- 9001234 linter: Add fix support for tsgolint diagnostics (#16344) (camc314)
- 0ec454c linter/plugins: Merge default options into options (#16358) (overlookmotel)

### 🐛 Bug Fixes

- a806d74 linter: Use import type for ESTree in visitor.d.ts (#16472) (camc314)
- 1a69b06 linter: Junit support for multiple files/errors/diagnostics (#16568) (Shrey Sudhir)
- fd84dcc linter/plugins: `RuleTester` parser catch fatal errors (#16558) (overlookmotel)
- ab4deb0 linter/plugins: Improve safety of options merging (#16549) (overlookmotel)
- f7751cc linter/plugins: Fix TS types for `RuleTester` (#16546) (overlookmotel)
- 6d33320 linter/plugins: Prevent user modifying the default config (#16545) (overlookmotel)
- b4483c5 linter/plugins: Add config support skeleton to `RuleTester` (#16537) (overlookmotel)
- 3a49158 linter/plugins: Improve TS types for tokens (#16502) (overlookmotel)
- 2d3217e linter/plugins: Remove panics from `lint_file` and `setup_configs` (#16453) (overlookmotel)

### ⚡ Performance

- 793b989 linter/plugins: Move result-processing work off main JS thread (#16456) (overlookmotel)
- 44dff7b linter/plugins: Skip serialization overhead when no errors (#16443) (overlookmotel)
- 1aa2409 linter/plugins: Do not remove `messageId` field from `DiagnosticReport` before sending to Rust (#16442) (overlookmotel)

### 📚 Documentation

- e24aabd linter/plugins: Correct comment (#16559) (overlookmotel)
- 8c85e08 linter/plugins: Add TODO comment (#16511) (overlookmotel)
- a9b9298 linter/plugins: Add JSDoc comments to `SourceCode` properties (#16497) (overlookmotel)
- 467cc1a linter/plugins: Improve comment on error branch (#16464) (overlookmotel)

## [1.31.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 74cf572 ast: [**BREAKING**] Make `source` field of `TSImportType` a `StringLiteral` (#16114) (copilot-swe-agent)

### 🚀 Features

- 5da1a63 linter/plugins: Introduce `RuleTester` (#16206) (overlookmotel)
- 41129ab linter/plugins: Implement `languageOptions.parser` (#16292) (overlookmotel)
- 7150209 linter/plugins: Implement `SourceCode#getNodeByRangeIndex` (#16256) (overlookmotel)
- 3226864 linter/plugins: Implement options merging (#16217) (overlookmotel)
- cbb108a linter/plugins: Support default options (#16170) (overlookmotel)
- 04a3a66 linter/plugins: Implement `SourceCode#getTokenOrCommentAfter()` (#16045) (Arsh)
- 68b63d9 linter/plugins: Implement `SourceCode#getTokenOrCommentBefore()` (#16044) (Arsh)
- 04d9454 linter/plugins: Implement `SourceCode#getTokenByRangeStart()` (#16043) (Arsh)
- 7b8d578 linter/plugins: Implement `SourceCode#getTokensBetween()` (#16034) (Arsh)
- 79c242f linter/plugins: Implement `SourceCode#getLastTokensBetween()` (#16033) (Arsh)
- 1772078 linter/plugins: Implement `SourceCode#getFirstTokenBetween()` (#16032) (Arsh)
- 21bb86d linter/plugins: Implement `SourceCode#getFirstTokensBetween()` (#16019) (Arsh)
- 78f74b1 linter/plugins: Implement `SourceCode#getLastTokenBetween()` (#16008) (Arsh)
- df0b948 linter/plugins: Implement `SourceCode#getLastToken()` (#16003) (Arsh)

### 🐛 Bug Fixes

- cf249f5 linter/plugins: Fix message interpolation (#16300) (overlookmotel)
- 9149a26 linter/plugins, napi/parser: Deep freeze visitor keys (#16293) (overlookmotel)
- 653fa6c oxlint/oxfmt/lsp: Tell client the real tool name & version (#16212) (Sysix)
- 0df1901 linter/plugins: Reset state after error during AST visitation (#16246) (overlookmotel)
- 78aa294 linter/plugins: Deep freeze options (#16218) (overlookmotel)
- 123bffe linter/plugins: Handle zero-token files in `SourceCode#getLastToken()` (#16184) (Arsh)
- 55fcfba linter: Add considerDefaultExhaustiveForUnions option to switch-exhaustiveness-check (#16204) (camc314)
- 9cc20a1 minifier: Avoid merging side effectful expressions to next assignment statement if the side effect may change the left hand side reference (#16165) (sapphi-red)
- 75249e0 linter/plugins: Handle non-UTF8 file paths (#16157) (overlookmotel)
- 86fa667 linter/plugins: Improve type def for `RuleMeta` `defaultOptions` property (#16159) (overlookmotel)
- 91eb3f2 ast/estree: Convert `TSImportType` `argument` field to `Literal` (#16109) (overlookmotel)
- f5cb601 linter/plugins: Perform length checks before continuing loops (#16025) (Arsh)

### ⚡ Performance

- 02bdf90 linter/plugins, napi/parser: Reuse arrays in visitor keys (#16294) (overlookmotel)
- d3a34f8 linter/plugins: Optimize `getTokens()` and other methods (#16188) (Arsh)
- c05db06 linter/plugins: Speed up `initTokensWithComments` (#16117) (overlookmotel)
- 4846886 linter/plugins: Optimize merging of `tokens` and `comments` (#16071) (Arsh)
- e232d35 linter/plugins: Recycle objects in token methods (#16068) (overlookmotel)

### 📚 Documentation

- e928732 linter/plugins: Fix JSDoc comment (#16295) (overlookmotel)
- be36e36 linter/plugins: Fix JSDoc comment for `loadPluginImpl` (#16211) (overlookmotel)
- 0e1d38a linter/plugins: Clarify JSDoc comment for `getTokensBetween` (#16070) (overlookmotel)
- 3ee22b2 linter/plugins: Fix JSDoc comments for tokens methods (#16063) (overlookmotel)
- f257b5c linter/plugins: Clarify JSDoc comments for tokens methods (#16062) (overlookmotel)

## [1.30.0] - 2025-11-24

### 💥 BREAKING CHANGES

- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

### 🚀 Features

- 0c1f82b linter/plugins: Add `tokens` property to `Program` (#16020) (overlookmotel)
- 9e61beb linter/plugins: Implement `SourceCode#getFirstToken()` (#16002) (Arsh)
- 9a548dd linter/plugins: Implement `SourceCode#getLastTokens()` (#16000) (Arsh)
- 0b6cb11 linter/plugins: Implement `SourceCode#getFirstTokens()` (#15976) (Arsh)
- 166781e linter/plugins: Implement `SourceCode#getTokenAfter()` (#15973) (Arsh)
- 6ae232b linter: Expose type errors via tsgolint (#15917) (camc314)
- 2bfdd26 linter/plugins: Implement `SourceCode#getTokensAfter()` (#15971) (Arsh)
- 45fffc1 linter/plugins: Implement `SourceCode#getTokenBefore()` (#15956) (Arsh)
- 776e473 linter/plugins: Implement `SourceCode#getTokensBefore()` (#15955) (Arsh)
- 986cac1 linter/plugins: Token-related `SourceCode` APIs (TS ESLint implementation) (#15861) (Arsh)
- 4b9d8d2 linter/type-aware: Include range with tsconfig diagnostics (#15916) (camc314)

### 🐛 Bug Fixes

- 2bd3cb6 apps, editors, napi: Fix `oxlint-disable` comments (#16014) (overlookmotel)
- a8a2032 linter: Support missing `range` for internal diagnostics (#15964) (camc314)
- 9fa9ef2 linter: Gracefully fail when using import plugin, large file counf and JS plugins (#15864) (camc314)
- c027398 linter/plugins: Correct bindings package names glob in TSDown config (#15871) (overlookmotel)
- 46bd6bd linter/plugins: Pin `@typescript-eslint/scope-manager` dependency (#15807) (overlookmotel)
- fba31fa linter: Patch `@typescript-eslint/scope manager` (#15214) (Arsh)

### ⚡ Performance

- 024b48a linter/plugins: Lazy-load tokens parsing code (#16011) (overlookmotel)
- 15365c9 linter/plugins: Reduce var assignments (#15953) (overlookmotel)
- 84d1f4f linter/plugins: Downgrade some checks to debug-only (#15922) (overlookmotel)

### 📚 Documentation

- 6c72e84 linter: Use backticks for code elements across more rule diagnostics (#15958) (connorshea)
- a63dad7 linter/plugins: Add comment (#15952) (overlookmotel)
- db6a110 linter/plugins: Fix JSDoc comment (#15884) (overlookmotel)
- fbf0fd4 linter/plugins: Add JSDoc comments to `Plugin` and `Rule` types (#15815) (overlookmotel)
- ac5e4b5 linter/plugins: Add JSDoc comments and improve comments (#15814) (overlookmotel)

## [1.29.0] - 2025-11-17

### 🚀 Features

- e01c551 oxlint: Add `--lsp` flag to run the language server (#15611) (Sysix)
- f5d9abb oxlint: Add enabled? column to --rules cli output (#15213) (Wren)
- 6b5205c linter/plugins: Implement deprecated `SourceCode#getJSDocComment` method (#15653) (overlookmotel)
- 5eccff1 linter/plugins: Implement deprecated `SourceCode` tokens methods (#15645) (overlookmotel)
- 0d52a5e linter/plugins: Implement `Context#parserOptions` getter (#15632) (overlookmotel)
- 287a538 linter/plugins: Implement `Context#get*` deprecated methods (#15606) (overlookmotel)

### 🐛 Bug Fixes

- 7c4a916 linter: Restores `oxlint --rules -f=json` functionality. (#15689) (Wren)
- 24d00f4 linter/plugins: Add types for suggested fixes (#15636) (overlookmotel)
- 257360f linter/plugins: Fill in TS type def for `RuleMeta` (#15629) (overlookmotel)
- bb5f8ca oxlint: Fix type annotation for big-endian (#15561) (Sysix)

### ⚡ Performance

- e2a0997 linter/plugins: Recycle empty visitor object in ESLint compat mode (#15693) (overlookmotel)
- 54061e9 linter/plugins: Avoid implicit boolean coercion in `initLines` (#15641) (overlookmotel)

### 📚 Documentation

- a5feebc linter: `oxlint-disable` not `eslint-disable` (#15672) (overlookmotel)
- 3d15805 linter: Reformat doc comments (#15670) (overlookmotel)
- 16fcba6 linter: Remove "experimental" from description of stable features (#15669) (overlookmotel)
- e62fd98 linter: Correct comment on what `EnablePlugins` does (#15668) (overlookmotel)
- a25d31e linter: Fix grammar (#15666) (overlookmotel)
- f5f452f linter: Add missing `perf` category (#15667) (overlookmotel)
- a210b12 linter/plugins: Improve JSDoc comment for `RuleOptionsSchema` (#15642) (overlookmotel)
- 3aabfac linter/plugins: Alter comments on `FILE_CONTEXT` used in ESLint-compat `Context` shim (#15605) (overlookmotel)

## [1.27.0] - 2025-11-10

### 🚀 Features

- 222a8f0 linter/plugins: Implement `SourceCode#isSpaceBetween` (#15498) (overlookmotel)
- 2f9735d linter/plugins: Implement `context.languageOptions` (#15486) (overlookmotel)
- bc731ff linter/plugins: Stub out all `Context` APIs (#15479) (overlookmotel)
- 5822cb4 linter/plugins: Add `extend` method to `FILE_CONTEXT` (#15477) (overlookmotel)
- 7b1e6f3 apps: Add pure rust binaries and release to github (#15469) (Boshen)

### 🐛 Bug Fixes

- 6957fb9 linter/plugins: Do not allow access to `Context#id` in `createOnce` (#15489) (overlookmotel)
- 7409630 linter/plugins: Allow access to `cwd` in `createOnce` in ESLint interop mode (#15488) (overlookmotel)
- a17ca32 linter/plugins: Replace `Context` class (#15448) (overlookmotel)
- fde753e linter/plugins: Block access to `context.settings` in `createOnce` (#15394) (overlookmotel)
- cc403f5 linter/plugins: Return empty object for unimplemented parserServices (#15364) (magic-akari)

### ⚡ Performance

- 3c57291 linter/plugins: Optimize loops (#15449) (overlookmotel)
- 3166233 linter/plugins: Remove `Arc`s (#15431) (overlookmotel)
- 9de1322 linter/plugins: Lazily deserialize settings JSON (#15395) (overlookmotel)
- 3049ec2 linter/plugins: Optimize `deepFreezeSettings` (#15392) (overlookmotel)
- 444ebfd linter/plugins: Use single object for `parserServices` (#15378) (overlookmotel)

### 📚 Documentation

- 97d2104 linter: Update comment in lint.rs about default value for tsconfig path (#15530) (Connor Shea)

## [1.26.0] - 2025-11-05

### 🚀 Features

- 230e34c linter/plugins: Allow js plugins to access settings (#14724) (Arsh)
- 7a00691 linter/no-deprecated: Add rule (#15272) (camc314)
- ab065a9 tsgolint: Improve diagnostic messages with file reference (#15274) (camc314)
- 979ec04 linter: Pretty print tsgolint internal diagnostics (#15131) (camc314)

### 🐛 Bug Fixes

- 40231a6 linter/plugins, napi/parser: Add `parent` field to `FormalParameterRest` and `TSParameterProperty` in TS type defs (#15337) (overlookmotel)
- 861508a linter/plugins: Make `parent` fields in TS type defs non-optional (#15336) (overlookmotel)
- 7f079ab ast/estree: Fix raw transfer deserializer for `AssignmentTargetPropertyIdentifier` (#15304) (overlookmotel)
- 56c6627 linter/plugins: Resolve JS plugins only with conditions Node.js supports (#15248) (sapphi-red)
- f376e61 linter: Bundle `@typescript-eslint/scope-manager` (#15210) (Arsh)
- 80a187c linter: Add offset for parsing error in partial loading files (#15075) (Liang Mi)

### 🚜 Refactor

- 636e7ed linter/plugins: Shorten `ScopeManager` code (#15335) (overlookmotel)
- a7cf856 ast/estree: Shorten raw transfer deserializer for `AssignmentTargetPropertyIdentifier` (#15303) (overlookmotel)

### 📚 Documentation

- a7d9f1d linter/plugins: Reformat and clarify `ScopeManager` JSDoc comments (#15333) (overlookmotel)
- 69e61d4 linter/plugins: Update comment (#15293) (overlookmotel)

### ⚡ Performance

- 8b31daa linter/plugins: Small optimizations to `ScopeManager` (#15334) (overlookmotel)
- 4c0ba92 linter/plugins: Use singleton object for `ScopeManager` (#15332) (overlookmotel)
- c82fab0 ast/estree: Remove pointless assignments from raw transfer deserializers (#15305) (overlookmotel)
- ee9f2f4 linter/plugins: Faster check for `cwd` (#15301) (overlookmotel)

### 🧪 Testing

- 48e646b oxlint/lsp: Update snapshot for invalid syntax test (#15191) (Sysix)
- dbc260b linter: Disable tsgolint exit code specific test on windows (#15117) (camc314)


## [1.25.0] - 2025-10-30

### 💥 BREAKING CHANGES

- 659fd37 linter: [**BREAKING**] `tsgolint`: request fixes when necessary (#15048) (camchenry)

### 🚀 Features

- ed24d60 linter: Expose tsgolint program diagnostics (#15080) (camc314)
- f7bef73 linter/plugins: Scope manager API (#14890) (Arsh)
- 3e15cdd linter/strict-boolean-expression: Add rule (#14930) (camc314)
- bd74603 linter: Add support for vitest/valid-title rule (#12085) (Tyler Earls)

### 🐛 Bug Fixes

- 597340e ast-tools: Use oxfmt to format generated code (#15064) (camc314)
- 2de9f39 linter/plugins: Fall back to package name if meta.name is missing (#14938) (Peter Wagenet)

### 🧪 Testing

- bf898e5 linter: Increase stability of tsgolint test cases (#15063) (camc314)


## [1.24.0] - 2025-10-22

### 🚀 Features

- 54ec8e3 linter: Add `cwd` property to JS plugin `Context` (#14814) (magic-akari)
- 9700a56 linter/plugins: Comment-related APIs (#14715) (Arsh)
- bec7a7d semantic: Add scope to `TSConstructorType` (#14676) (camc314)
- b1a9a03 linter/plugins: Implement `SourceCode#getAllComments` (#14589) (Arsh)

### 🐛 Bug Fixes

- 28e76ec oxlint: Resolving JS plugin failing when `extends` is used (#14556) (camc314)
- 78ee7b8 linter/plugins: Handle utf16 characters within comment spans (#14768) (Arsh)
- 47d8db1 linter/plugins: Prevent `comments` being accessed after file is linted (#14727) (overlookmotel)
- 5238891 linter/plugins: Add `comments` field to TS type def for `Program` (#14626) (overlookmotel)
- 84b2605 linter/plugins: Remove `parent` property from comments (#14624) (overlookmotel)
- 0961c3a oxlint,oxfmt: Skip traversing `.git` directories (#14590) (Boshen)

### 🚜 Refactor

- 4520695 linter/plugins: Reorganise `SourceCode` methods (#14773) (overlookmotel)
- 6942d75 linter/plugins: Shorten import of comment-related `SourceCode` methods (#14772) (overlookmotel)
- b9a3f46 linter/plugins: Move scope-related `SourceCode` methods into separate file (#14771) (overlookmotel)
- cd068aa linter/plugins: Move token-related `SourceCode` methods into separate file (#14770) (overlookmotel)
- ec816ba linter/plugins: Move comments-related code into separate file (#14753) (overlookmotel)
- e9c3b18 linter/plugins: Update outdated comment (#14750) (overlookmotel)
- 14de671 linter/plugins: Simplify `comments` getter (#14728) (overlookmotel)
- b402024 linter/plugins: Rename function (#14726) (overlookmotel)
- 85a2743 linter/plugins, napi/parser: Remove extraneous code from raw transfer deserializers (#14683) (overlookmotel)
- 2b14abc napi/parser: Shorten raw transfer deserializer for `Comment` (#14623) (overlookmotel)

### 📚 Documentation

- 207b62b linter/plugins: Correct JSDoc comments for `SourceCode` tokens methods (#14776) (overlookmotel)
- cd266b4 linter/plugins: Improve docs for comments APIs (#14754) (overlookmotel)

### ⚡ Performance

- 10182e8 linter/plugins: Use binary search (#14778) (Arsh)
- e6f351d linter/plugins: Speed up `SourceCode#getAncestors` (#14747) (overlookmotel)
- 58ba6d6 linter/plugins: Lazy deserialize comments array (#14637) (Arsh)

### 🎨 Styling

- 3029dfb linter/plugins: Reorder code (#14725) (overlookmotel)

### 🧪 Testing

- 5933097 oxlint: Add test for nested configs importing the same plugin 2x (#14646) (camc314)
- 6570f36 linter/custom-plugins: Snapshot test start, end, range, and loc for comments (#14779) (Arsh)


## [1.23.0] - 2025-10-13

### 🐛 Bug Fixes

- 6fce7f4 oxlint/changelog: Remove duplicate changelog entries (#14528) (camc314)
- 74e52f3 linter/plugins: Resolve JS plugins with ESM condition names (#14541) (magic-akari)

### 🚜 Refactor

- 4f301de napi/parser, linter/plugins: Improve formatting of generated code (#14554) (overlookmotel)
- 68c0252 napi/parser, linter/plugins: Shorten generated raw transfer deserializer code (#14553) (overlookmotel)
- 20e884e linter: Store `LintService` in `LintRunner` (#14471) (Sysix)

### ⚡ Performance

- 31766fd linter/plugins: Provide `loc` via prototype (#14552) (overlookmotel)

### 🧪 Testing

- 8d8881d linter/plugins: Expand tests for module resolution of plugins (#14559) (overlookmotel)


## [1.22.0] - 2025-10-08

### 🐛 Bug Fixes

- 0dcdbd1 oxlint: Bundle esquery (#14450) (camc314)


## [1.21.0] - 2025-10-08

### 🚀 Features

- 576be20 linter/plugins: Support selectors DSL (#14435) (overlookmotel)
- b2de44f linter/plugins: Support interpolation in normal diagnostic `message` (#14419) (overlookmotel)
- 382c5be linter/plugins: Support placeholders in messageIds (#14416) (camc314)
- 529e88e linter/plugins: Support `messageId`s (#14415) (camc314)
- 0ec0847 ci: Run napi tests on windows (#14383) (camc314)

### 🐛 Bug Fixes

- 88ec1bd linter/plugins: Fix error messages (#14423) (overlookmotel)
- 18616c2 oxlint: Ignore fixtures dir for vitest (#14414) (camc314)
- ec02fe8 oxlint: Normalize path separators in snapshot tests (#14406) (camc314)
- 96663fb linter/plugins: Do not call `before` hook if empty visitor (#14401) (overlookmotel)
- 52f04bd linter: Use `pathToFileURL` for importing plugins to ensure correct URL format (#14394) (camc314)
- 1ea0d46 oxlint: Resolve tsdown deprecation warning (#14389) (camc314)

### 🚜 Refactor

- 3b26bf3 linter/plugins: Split adding visit function to compiler visitor into multiple functions (#14433) (overlookmotel)
- af3a75e linter/plugins: Track ancestors while walking AST (#14432) (overlookmotel)
- f279f0b linter/plugins: Do not lazy-load visitor keys (#14431) (overlookmotel)
- 5e99ed3 linter/plugins: Allow nullish values as `message` or `messageId` (#14422) (overlookmotel)
- dc30938 linter/plugins: Remove default value from `Context` constructor (#14421) (overlookmotel)
- 28cfae0 oxlint: Use `vitest`s built in file snapshot comparison (#14392) (camc314)
- 06b0e9f linter/plugins: Convert generated files to TS (#14385) (overlookmotel)
- 52f35c6 napi/parser, linter/plugins: Rename `types.js` to `type_ids.js` (#14384) (overlookmotel)

### ⚡ Performance

- 26435a1 linter/plugins: Small perf optimizations (#14420) (overlookmotel)
- d8a8be1 linter/plugins: Avoid private methods (#14418) (overlookmotel)

### 🧪 Testing

- d8da4a4 linter/plugins: Clarify tests for message placeholders (#14417) (overlookmotel)


## [1.20.0] - 2025-10-06

### 🚀 Features

- d16df93 linter: Support disable directives for type aware rules (#14052) (camc314)
- a2914fe linter/plugins: Add `loc` field getter to all AST nodes (#14355) (overlookmotel)
- 07193c2 linter/plugins: Implement `SourceCode#getAncestors` (#14346) (overlookmotel)
- c8de6fe linter/plugins: Add `parent` field to AST nodes (#14345) (overlookmotel)
- 5505a86 linter/plugins: Include `range` field in AST (#14321) (overlookmotel)
- 1347de4 linter/plugins: Accept diagnostics with `loc` (#14304) (overlookmotel)
- aefc8b3 linter/plugins: Implement `SourceCode#getIndexFromLoc` and `getLocFromIndex` (#14303) (overlookmotel)
- 93807db linter/plugins: Implement `SourceCode#lines` property (#14290) (overlookmotel)
- 2f8c985 linter/plugins: Implement `SourceCode#visitorKeys` property (#14289) (overlookmotel)
- b69028f linter/plugins: Implement `SourceCode#ast` property (#14287) (overlookmotel)
- bdf9010 linter/plugins: Add `SourceCode` API (#14281) (overlookmotel)

### 🐛 Bug Fixes

- 9a902c0 linter/plugins: Make `range` field non-optional on AST types (#14354) (overlookmotel)
- 46cceb8 linter/rules-of-hooks: Correctly place primary span to fix disable directive (#14237) (camc314)

### 🚜 Refactor

- 1489376 napi/parser, linter/plugins: Minify walker code (#14376) (overlookmotel)
- c8eeeb5 linter/plugins: Remove build-time dependency on `napi/parser` (#14374) (overlookmotel)
- fb1a067 linter/plugins: Bundle walker and AST types map (#14373) (overlookmotel)
- 93d8164 linter/plugins: Export AST types direct from `oxlint` package (#14353) (overlookmotel)
- 230d996 linter/plugins: `SourceCode#getText` use `range` (#14352) (overlookmotel)
- 6e52bbd linter/plugins: Move location-related code into separate file (#14350) (overlookmotel)
- 13f1003 linter/plugins: Share `ast` between files (#14349) (overlookmotel)
- 79eadf8 linter: Introduce `LintRunner` (#14051) (camc314)
- 65873ba linter/plugins: Add stubs for all `SourceCode` methods (#14285) (overlookmotel)
- 989ce2f linter/plugins: Convert `Node` type to interface (#14280) (overlookmotel)

### ⚡ Performance

- e75d42d napi/parser, linter/plugins: Remove runtime `preserveParens` option from raw transfer deserializers (#14338) (overlookmotel)
- 2e57351 linter/plugins: Initialize `lineStartOffsets` as `[0]` (#14302) (overlookmotel)
- c27a393 linter/plugins: Deserialize AST on demand (#14288) (overlookmotel)
- 95a8cc4 linter/plugins: Use singleton for `SourceCode` (#14286) (overlookmotel)


### 🧪 Testing

- 0061ce7 linter: Add more tests for disable directives in partial loadable files (#14371) (camc314)
- 1387aaa linter/plugins: Test `createOnce` returning no visitor functions (#14279) (overlookmotel)


## [1.19.0] - 2025-09-29

### 🚀 Features

- acd1266 linter/plugins: `oxlint` export types (#14163) (overlookmotel)
- 00954de linter/plugins: Remove `--js-plugins` CLI option (#14134) (overlookmotel)
- b4d716f linter/plugins: Move custom JS plugin config to `jsPlugins` (#14133) (overlookmotel)
- 9c3afea linter/plugins: Support fixes (#14094) (overlookmotel)
- 1472147 linter: Move `no-unused-expressions` to correctness (#14099) (camchenry)
- c796966 linter/plugins: Add `meta` property to rules (#14089) (overlookmotel)

### 🐛 Bug Fixes

- 39a171e linter: Get cli args on JS side, to avoid runtime inconsistencies (#14223) (camc314)
- e045391 linter/plugins: Error on JS plugin with reserved name (#14226) (overlookmotel)
- 37f6b09 linter/plugins: Make `null` a valid value for `meta.fixable` (#14204) (overlookmotel)
- e9a14d1 linter/plugins: Allow `fix` function to return `undefined` (#14182) (overlookmotel)
- ee9ecbe linter/plugins: Fix TS type for fixer methods (#14166) (overlookmotel)
- 03d1684 linter/plugins: Output warning on first JS plugin load (#14165) (overlookmotel)
- 9716f7c linter/plugins: Fix TS types (#14162) (overlookmotel)
- 4a4fce8 linter: Fix cli argument parsing (#14112) (camc314)
- 9f3e2bc linter/plugins: Output errors thrown in JS plugins (#14096) (overlookmotel)
- d8e9cc5 linter/plugins: Validate type of `before` and `after` hooks (#14086) (overlookmotel)

### 🚜 Refactor

- 61ec0a7 linter/plugins: Simplify creation of `context` in `defineRule` ESLint shim (#14206) (overlookmotel)
- 3b1fe6f linter/plugins: Flatten directory structure of `dist` (#14199) (overlookmotel)
- d52cba6 linter: Bump TSDown to latest (#14198) (overlookmotel)
- 983dd1b linter/plugins: Add `Fixer` type (#14180) (overlookmotel)
- 2f8b076 linter/plugins: Remove dead code (#14178) (overlookmotel)
- e69cd86 linter/plugins: `loadPluginImpl` return an object (#14087) (overlookmotel)

### 📚 Documentation

- b19f5bc linter/plugins: Improve JSDoc comments for `definePlugin` and `defineRule` (#14159) (overlookmotel)

### ⚡ Performance

- 2575065 linter/plugins: Store if rule is fixable as boolean (#14205) (overlookmotel)

### 🧪 Testing

- a9b603e linter/plugins: Convert all plugins in tests to TS (#14200) (overlookmotel)
- 6ff3a23 linter/plugins: Add tests for `.ts`, `.mts`, `.cts` plugin files (#14164) (overlookmotel)
- 8988d64 linter/plugins: Add line breaks to plugins files (#14181) (overlookmotel)
- 52db331 linter/plugins: Type-check test fixtures (#14158) (overlookmotel)
- aca083a linter/plugins: Include stderr output in snapshots (#14155) (overlookmotel)
- a3c8f46 linter/plugins: Do not run `pnpm` in tests (#14157) (overlookmotel)
- 0029b7f linter/plugins: Normalize line breaks in snapshots (#14154) (overlookmotel)
- 7f2c101 linter/plugins: Specify path to `node` in tests (#14152) (overlookmotel)
- fc14abc linter/plugins: Format test fixtures (#14125) (overlookmotel)
- a6f965f linter/plugins: Simplify configs in test fixtures (#14124) (overlookmotel)
- b1685f7 linter/plugins: Refactor tests (#14123) (overlookmotel)
- 788e495 linter/plugins: Improve ESLint compat tests (#14119) (overlookmotel)
- 5750077 linter/plugins: Fix file paths in snapshots (#14115) (overlookmotel)
- 5c862f9 linter/plugins: Standardize test fixture structure (#14114) (overlookmotel)


## [1.18.0] - 2025-09-24

### 🐛 Bug Fixes

- 314c27d linter/plugins: `definePlugin` apply `defineRule` to rules (#14065) (overlookmotel)
- 7bd01ed linter/plugins: `defineRule` call `createOnce` lazily (#14062) (overlookmotel)
- fb3e7e3 linter/plugins: `defineRule` accept visitor with no `before` / `after` hooks (#14060) (overlookmotel)

### 🚜 Refactor

- 3a706a7 linter: Rename `LintRunner` to `CliRunner` (#14050) (camc314)

### ⚡ Performance

- ce538c7 linter/plugins: Load methods of globals into local vars (#14073) (overlookmotel)

### 🧪 Testing

- 2fd4b1e linter/plugins: Rename test (#14064) (overlookmotel)
- f2b3934 linter/plugins: Test returning `false` from `before` hook skips visitation in ESLint (#14061) (overlookmotel)
- b109419 linter/plugins: Align ESLint plugin with Oxlint (#14059) (overlookmotel)


## [1.17.0] - 2025-09-23

### 🚀 Features

- f47f978 linter/plugins: Add `definePlugin` API (#14008) (overlookmotel)
- 3e117c6 linter/plugins: Add `defineRule` API (#13945) (overlookmotel)
- 2dc8adb linter/plugins: Add `createOnce` API (#13944) (overlookmotel)
- bef8753 linter/plugins: ESTree-compatible AST for JS plugins (#13942) (overlookmotel)
- a14aa79 npm/oxlint: Convert to ES modules (#13876) (Boshen)
- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- c75ae8c linter/plugins: Add options to `Context` (#13810) (overlookmotel)
- 53d04dd linter: Convert `oxlint` to NAPI app (#13723) (overlookmotel)

### 🐛 Bug Fixes

- 1f4be38 napi/parser: Generate `.d.mts` extension for types (#14038) (Daniel Roe)
- a018756 linter/plugins: Pin `tsdown` dependency to 0.15.1 (#14005) (overlookmotel)
- a34918a linter/plugins: Avoid lint warnings compiling WASM or big-endian (#13968) (overlookmotel)
- dd3843f linter: Set package version in `package.json` (#13890) (overlookmotel)
- fac7624 linter/plugins: Improve error for no JS plugins (#13858) (overlookmotel)

### 🚜 Refactor

- bb040bc parser, linter: Replace `.mjs` files with `.js` (#14045) (overlookmotel)
- 0d48511 linter/plugins: Improve handling `Context` method calls in `createOnce` (#14032) (overlookmotel)
- 6bc7664 oxlint: Run oxlint before tsgolint (#13519) (camc314)
- ac3e9e9 napi/parser: Move JS code into `src-js` directory (#13899) (overlookmotel)
- 7e0d736 linter/plugins: Rename `--experimental-js-plugins` to `--js-plugins` (#13860) (overlookmotel)
- 6245c8c linter/plugins: Make `Context` properties getters (#13809) (overlookmotel)
- a2342a6 linter/plugins: Import named in tests (#13807) (overlookmotel)

### 📚 Documentation

- 601c876 linter: Add comment explaining why Mimalloc is feature-gated (#14037) (overlookmotel)

### ⚡ Performance

- 4d04c6e linter/plugins: Flatten `LintFileResult` fields (#14033) (overlookmotel)
- a79af0a linter: Replace `for (... of ...)` loops (#13913) (overlookmotel)

### 🎨 Styling

- 8083740 linter: Import `Serialize` at top level (#14030) (overlookmotel)

### 🧪 Testing

- f51240e linter/plugins: Tests for different import styles (#13859) (overlookmotel)
- 407c95e linter/plugins: Check `this` is undefined in visit functions (#13811) (overlookmotel)
- f023a22 linter/plugins: Include stack trace in plugin loading errors (#13808) (overlookmotel)

### 💼 Other

- 0630d68 linter: Build `oxlint` locally with Mimalloc in release mode (#14034) (overlookmotel)


## [1.16.0] - 2025-09-16

### 🐛 Bug Fixes

- 3af1e5d linter/no-unsafe-declaration-merging: Always mark first span as primary (#13830) (camc314)
- 12baf5e linter/exhaustive-deps: Respect primary span when identifying disable directive location (#13781) (camc314)
- 09428f6 linter/plugins: Remove outdated comment (#13691) (overlookmotel)
- a294721 linter/plugins: Exit early if JS plugins enabled on unsupported platforms (#13689) (overlookmotel)
- 68a2280 linter/plugins: More graceful exit for `--experimental-js-plugins` CLI option (#13688) (overlookmotel)

### 🚜 Refactor

- 7346099 linter: Move `oxlint` application code into separate module (#13745) (overlookmotel)
- 6dd4107 linter: Remove `#[cfg(test)]` attributes from `tester` module (#13714) (overlookmotel)
- c40c6ef linter/plugins: Directory for JS plugins-related code (#13701) (overlookmotel)
- 1fd993f napi/oxlint: Rename `napi/oxlint2` to `napi/oxlint` (#13682) (overlookmotel)

### 🎨 Styling

- 99a7638 linter: Add comments + re-organise imports (#13715) (overlookmotel)

### 🧪 Testing

- fb2d087 linter: Set CWD for tests (#13722) (overlookmotel)


## [1.15.0] - 2025-09-11

### 💥 BREAKING CHANGES

- edc70ea allocator/pool: [**BREAKING**] Remove `disable_fixed_size` Cargo feature (#13625) (overlookmotel)

### 🐛 Bug Fixes

- 0d867b1 linter: Skip running tsgolint when no files need type aware linting (#13502) (Copilot)

### 🚜 Refactor

- 7775c21 linter/plugins: Remove `oxlint2` Cargo feature (#13648) (overlookmotel)
- 6cd6be2 linter: Add `--experimental-js-plugins` CLI arg (#13658) (overlookmotel)
- 2f02ac6 linter/plugins: Remove `disable_oxlint2` Cargo feature (#13626) (overlookmotel)
- ff9e4fb linter/plugins: Use fixed-size allocators when `ExternalLinter` exists (#13623) (overlookmotel)
- 91759c6 linter/plugins: Only use `RawTransferFileSystem` if JS plugins registered (#13599) (overlookmotel)
- 118020c linter/plugins: Discard `ExternalLinter` if no JS plugins registered (#13598) (overlookmotel)
- 8d30bce linter/tsgolint: Report an error if the tsgolint exe could not be found (#13590) (camc314)
- d245376 oxlint: Remove unused `runner` module (#13561) (camc314)

### 🧪 Testing

- 58e6c94 oxlint: Add test for ignorePatterns whitelist (#13372) (Sysix)


## [1.14.0] - 2025-08-30

### 🚜 Refactor

- 6431033 linter: Move ignore logic to `LintIgnoreMatcher` (#13222) (Sysix)

### 📚 Documentation

- 51d3840 linter: Update oxlint CLI help message on `.oxlintrc.json` config file (#13334) (0xCLARITY)

### 🧪 Testing

- 6eeeb67 oxlint: Add test for ignore patterns referenced by symlink file (#13356) (Sysix)


## [1.13.0] - 2025-08-26

### 💥 BREAKING CHANGES

- 63abd7c oxlint: [**BREAKING**] Do not ignore hidden dot directories by default (#13194) (Sysix)

### 🐛 Bug Fixes

- 648e939 linter: Parse `ignorePatterns` with gitignore syntax (#13221) (Sysix)

### 🚜 Refactor

- c138fad linter: Avoid fs reads in `TsGoLintState` when `--silent` is used (#13199) (Sysix)


## [1.12.0] - 2025-08-17

### 🚀 Features

- da3c7fb oxlint: Oxlint v0.0.3 (#13148) (Boshen)
- aecacae linter: Support `ignorePatterns` for nested configs (#12210) (Sysix)
- 61112a3 linter: Add 36 new TypeScript ESLint rules with comprehensive test fixtures (#12893) (Copilot)

### 🐛 Bug Fixes

- 66a350e oxlint: Should type linting files after ignore (#13149) (Boshen)
- 43b1c5a linter: Do not count type-aware rules, when not enabled (#13062) (Sysix)
- a0ccada tsgolint: Handle non-zero exit status from tsgolint process (#13087) (camc314)

### 🚜 Refactor

- 8459a12 linter: Pass paths to `TsGoLintState.lint` method (#13131) (Sysix)
- f0a517f linter: Pass cwd instead of `LintServiceOptions` into `TsGoLintState` (#13127) (Sysix)
- 34ae2f0 linter: Move `tsgolint.rs` to `oxc_linter` crate (#13126) (Sysix)
- 9f924f6 linter: Always explicitly initialize Rayon thread pool (#13122) (overlookmotel)
- 6c5b8be linter: Create `AllocatorPool` in `Runtime::new` (#13106) (overlookmotel)
- cc2a85b linter: Remove `CliRunResult` from `TsGoLintState` (#13119) (Sysix)
- 23e5642 linter: Move `TsGoLintInput` creation into own function (#13118) (Sysix)

### ⚡ Performance

- 3bfb235 linter: Implement streaming diagnostics for tsgolint instead of waiting for output to finish (#13098) (copilot-swe-agent)

### 🎨 Styling

- 4f2cc96 linter: Add line break (#13061) (overlookmotel)


## [1.11.2] - 2025-08-12

### 🐛 Bug Fixes

- c461a86 oxlint: Fix type-aware linting crash when Vue files are present (#13007) (Copilot)
- 2936545 linter/tsgolint: Report an error if tsgolint executable failed to spawn (#12984) (camc314)
- a13b3ee oxlint: Run `tsgolint.CMD` under windows (#12932) (Sysix)

### 🚜 Refactor

- 69303de oxlint: Pass `DiagnosticService` as a parameter for `TsGoLintState.lint()` (#13004) (Sysix)

### 🧪 Testing

- fb8cbbf oxlint: Enable tsgolint test with config parameter for windows (#13001) (Alexander S.)
- d59f3bb oxlint: Match `x.ys` when replacing var (#12990) (camc314)
- d7cca12 linter: Add test for extended configs and overrides for tsgolint (#12924) (camchenry)


## [1.11.1] - 2025-08-09

### 🐛 Bug Fixes

- 7fc907f linter: Resolve configured rules for every file linted by `tsgolint` (#12886) (camchenry)

### 🚜 Refactor

- c072e01 all: Add missing lifetimes in function return types (#12895) (overlookmotel)

### 🧪 Testing

- 9d946aa oxlint: Skip `--type--aware` test for `big-endian` (#12911) (Sysix)
- 695fbdd oxlint: Fix `--type-aware` test on `big-endian` and skip for `windows` (#12910) (Sysix)
- 38118ab oxlint: Fix `--type-aware` snapshot + add non tsgolint rule (#12909) (Sysix)


## [1.11.0] - 2025-08-07

### 🚀 Features

- ac46347 oxlint: Add `tsgolint` integration (#12485) (camchenry)


## [1.10.0] - 2025-08-06

### 🚀 Features

- 9b35600 linter/jsx-a11y: Add support for mapped attributes in label association checks (#12805) (camc314)

### 🐛 Bug Fixes

- 45206dd linter: Apply fix span offset after fixing the section source text (#12758) (Sysix)

### 🚜 Refactor

- 030e397 linter: Simplify parsing CLI args (#12802) (overlookmotel)
- c0e224a linter: Store `ExternalRuleId` in `OxlintOverrides` not raw names (#12502) (camc314)

### ⚡ Performance

- 693673b linter: Reduce iterations when collecting directories for nested configs (#12329) (overlookmotel)

### 🎨 Styling

- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)

### 🧪 Testing

- d8ccff7 oxlint: Add `Tester::test_fix` mehod (#12754) (Sysix)


## [1.9.0] - 2025-07-29

### 🚜 Refactor

- 543fd53 napi/oxlint: Rename `run` to `lintFile` (#12567) (overlookmotel)
- 491c401 linter: Remove `#[must_use]` from `LintService::with_*` methods (#12560) (overlookmotel)
- d44b0ac linter: Remove `Runner` trait (#12559) (overlookmotel)
- bea652f linter: Add `vue` and `regex` to `BuiltinLintPlugins` (#12542) (Sysix)
- 5c33fc7 diagnostics: Implement `Eq` and `Ord` for `InfoPosition` (#12505) (overlookmotel)
- 7a0da04 diagnostics: Remove Option wrapper from MPSC channel and sender field (#12467) (camc314)

### 🧪 Testing

- d31adcf linter: Improve sorting diagnostics (#12504) (overlookmotel)


## [1.8.0] - 2025-07-22

### 🐛 Bug Fixes

- 46e33d5 linter: Improve error handling in config file lookup (#12391) (camc314)


## [1.7.0] - 2025-07-16

### 🚀 Features

- 5e428a4 linter/eslint-plugin-next: No-html-link-for-pages rule addition (#12194) (Gabriel Díaz Aguilera)
- c551b8f linter: Report diagnostics from custom plugins (#12219) (camc314)
- d387729 linter: JS custom rules config (#12160) (camc314)
- 152e59d napi/oxlint: Read source text into start of allocator (#12122) (overlookmotel)
- d4ebd14 linter: Add `oxlint2`/`disable_oxlint2` feature flags (#12130) (camc314)
- a4dae73 linter: Introduce `LintPlugins` to store builtin + custom plugins (#12117) (camc314)

### 🐛 Bug Fixes

- 9720774 linter: Report implicit config parse errors (#12260) (Simon Buchan)
- 853d2bc linter, language_server: Correctly identify usage of `import` plugin (#12157) (overlookmotel)

### 🚜 Refactor

- 6e54645 language_server: Store `LintService` instead of `Linter` (#12016) (Sysix)
- 113cf8c linter: Move `LintServiceOptions.paths` to `LintService.with_paths` (#12015) (Sysix)
- acfac68 oxlint: Adjust ignore patterns by counting bytes instead of chars (#12209) (Sysix)
- 1d2eaca oxlint2: Introduce `force_test_reporter` feature for consistent graphical outputs (#12133) (camc314)
- f7c675d linter: Rename `LintPlugins` to `BuiltinLintPlugins` (#12116) (camc314)

### 🧪 Testing

- d1194e8 oxlint: Ignore test on windows (#12262) (camc314)


## [1.6.0] - 2025-07-07

### 🚀 Features

- f81d336 linter: Introduce `ExternalLinter` struct (#12052) (camc314)

### 🐛 Bug Fixes

- 5851d2c oxlint: Always follow symlinks; remove cli flag `--symlinks` (#12048) (Boshen)

### 🚜 Refactor

- 2f7cbda linter: Move napi bindings out of oxc_linter (#12072) (camc314)
- 9254252 linter: Move code (#12071) (overlookmotel)


## [1.5.0] - 2025-07-02

### 🐛 Bug Fixes

- 4b2c658 oxlint: Make `--version` exit code be `0` (#11986) (camc314)


## [1.4.0] - 2025-06-30

### 🚀 Features

- 9b19b40 napi: Add basic oxlint napi bindings (#11877) (camc314)
- f102cb1 linter: Add `import/prefer-default-export` rule (#11891) (yefan)

### 🐛 Bug Fixes

- d991fed linter: Fix `jsx-a11y/label-has-associated-control` default values (#11832) (Sysix)

### 🚜 Refactor

- 2cf9fa3 linter: Derive debug for `extensions` (#11938) (camc314)


## [1.3.0] - 2025-06-23

### 🚜 Refactor

- b39d1fa linter: Output smaller spans for unused disable directives with multiple rules (#11781) (Sysix)


## [1.2.0] - 2025-06-19

### 🚀 Features

- 38dc614 oxc_linter: Reuse allocators (#11736) (camc314)


## [1.1.0] - 2025-06-12

### 🚀 Features

- 1181018 linter: Add eslint/no-extra-bind rule (#11588) (yefan)


## [1.0.0] - 2025-06-10

## [0.18.0] - 2025-06-06

- bd9dd88 linter: [**BREAKING**] Add more info to json reporter (#11524) (camc314)

### Features


### Bug Fixes

- 0946dac linter: Correctly inherit categories when plugins are enabled (#11353) (Cameron)

## [0.17.0] - 2025-05-30

- ead5309 linter: [**BREAKING**] Remove react from default plugin set (#11382) (camc314)

### Bug Fixes

- f6424dd linter: Reflect react plugin is disabled by default in cli (#11397) (camc314)

### Documentation

- cd354d4 oxlint: Remove incorrect doc comment (#11326) (camc314)

### Testing

- c4f64aa linter: Explicitly disable correctness for clarity (#11327) (camc314)

## [0.16.12] - 2025-05-25

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- 12b0917 linter: Auto-generate docs for rule configs (#10629) (DonIsaac)

### Bug Fixes

- e8470d9 linter: Delay merging of oxlintrc configs (#10835) (camc314)

### Refactor

- 9f3a14a linter: Cleanup diagnostic and docs for `eslint/no-console` (#11101) (Ulrich Stark)

## [0.16.11] - 2025-05-16

- 4e5c73b span: [**BREAKING**] `SourceType::from_path(".js")` return js instead of jsx (#11038) (Boshen)

### Features

- 466c24a linter: Add gitlab reporter output format (#10927) (Connor Pearson)

### Bug Fixes

- c52a9ba linter: Fix plugins inside overrides not being applied (#11057) (camc314)
- b12bd48 linter: Fix rule config not being correctly applied (#11055) (camc314)
- 0961296 linter: Add `gitlab` to linter `--help` docs (#10932) (camc314)
- 584d8b9 napi: Enable mimalloc `no_opt_arch` feature on linux aarch64 (#11053) (Boshen)

### Refactor

- bb999a3 language_server: Avoid cloning linter by taking reference in LintService (#10907) (Ulrich Stark)

## [0.16.10] - 2025-05-09

### Features

- 4c62348 linter: Regex/no-useless-backreference (#10773) (camc314)

### Refactor

- 79819cc linter: Move around some config store logic (#10861) (camc314)
- e132aba linter: Extract nested config searching to a fn (#10860) (camc314)
- efb4fb8 oxlint: Avoid result unwrap (#10836) (camc314)

## [0.16.9] - 2025-05-02

### Features

- 63f02a8 linter: Add react/forward_ref_uses_ref (#10506) (x6eull)
- eac205f linter: Add unicorn/consistent-assert rule (#10653) (Shota Kitahara)

### Bug Fixes

- e7c2b32 linter: Move `consistent-assert` to `pedantic` (#10665) (camc314)

## [0.16.8] - 2025-04-27

### Bug Fixes

- 723b4c6 linter: Cross_module of LintService not being enabled despite enabled import plugin (#10597) (Ulrich Stark)
- 9a02066 oxlint: Current dir as arg (#9382) (Ben Jones)

## [0.16.7] - 2025-04-21

### Bug Fixes

- 4e1f536 linter: Config path resolution when path contains '..' syntax (#10367) (Florian Bopp)

### Refactor

- 5ab4d40 linter: Simplify error handling (#10404) (camchenry)

## [0.16.6] - 2025-04-14

### Bug Fixes

- 9aaba69 linter: Nested configuration directory resolution (#10157) (Sub)

### Testing

- aa6ccd2 oxlint: Add test for nested and extended configuration with import plugin (#10372) (Sysix)

## [0.16.5] - 2025-04-07

### Features

- 2f6810a editor: Add named fixes for code actions (#10203) (camchenry)

### Bug Fixes

- f2eff56 linter: Fix `rule_id` for some diagnostics formats (#10251) (Alexander S.)
- d691701 various: Unwrap `Result` of `write!` macro (#10228) (overlookmotel)

### Performance

- b34e876 linter: Avoid cloning filters by refactoring functions to take references (#10247) (Ulrich Stark)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

## [0.16.4] - 2025-04-01

### Features

- 370266c semantic: Check redeclaration of variable declaration and function declaration in the block scope (#10074) (Dunqing)

### Bug Fixes

- 2c80858 linter: Enable multi-file analysis for nested configs (#10089) (camchenry)

### Refactor

- d8e49a1 linter: Compute lintable extensions at compile time (#10090) (camchenry)

## [0.16.3] - 2025-03-25

### Refactor

- 6432707 rust: Use `lazy-regex` (#10004) (Boshen)

## [0.16.2] - 2025-03-21

### Bug Fixes

- f649fb3 linter: Reclassify `unicorn/no-document-cookie` as restriction (#9933) (camchenry)

### Documentation

- 46a12c6 linter: Tell about junit `--format` options (#9931) (Sysix)

## [0.16.1] - 2025-03-20

### Features

- 8e3d9be linter: Support `--report-unused-disable-directive` (#9223) (1zumii)

### Bug Fixes

- e6f7c74 linter: Import and fix tests for typescript::no_unnecessary_parameter_property_assignment (#9720) (Ulrich Stark)

### Performance

- 84fa538 minify: Use mimalloc-safe to replace mimalloc (#9810) (LongYinan)

### Refactor

- b34cf94 oxlint: Remove `jemallocator` (#9823) (Boshen)

## [0.16.0] - 2025-03-16

- 225e266 linter: [**BREAKING**] Enable `--experimental-nested-config` by default and add `--disable-nested-config` option (#9760) (camchenry)

### Features


### Bug Fixes

- 22f18ac linter: Improve `jsx-a11y/anchor-ambiguous-text` diagnostic message (#9789) (1zumii)

## [0.15.15] - 2025-03-12

### Features

- 474a57b linter: A new multi-file analysis runtime (#9383) (branchseer)

### Bug Fixes

- ab594f1 linter: Turn oxc/no-redundant-constructor-init into typescript/no-unnecessary-parameter-property-assignment (#9618) (Uli)

## [0.15.14] - 2025-03-11

### Features

- 41f32ea linter: Allow adding more overrides via `extends` configs (#9475) (camchenry)
- fb7cf10 linter: Allowing `plugins` to be extended with `extends` (#9473) (camchenry)
- fc74849 linter: Inherit `rules` via the extended config files (#9308) (camchenry)

### Bug Fixes

- 4ca62ab linter: Output right file line and column for `.vue`, `.astro` and `.svelte` files (#9484) (Sysix)
- 3105159 linter: Do not output number of rules with nested configs (#9476) (camchenry)
- 5ecda01 linter: Support nested extending (#9472) (camchenry)

### Refactor

- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

### Testing

- 934a387 linter: Remove test dependency on oxlint (#9513) (camchenry)

## [0.15.13] - 2025-03-04

### Features

- 4ad328b linter: Add oxc/no-redundant-constructor-init (#9299) (Ben Jones)

## [0.15.12] - 2025-02-23

### Features

- 9bc3017 linter: Add support for nested config files (#9153) (camchenry)
- cc8dd48 linter: Add unicorn/no-invalid-fetch-options rule (#9212) (Mikhail Baev)
- af13b1b linter: Promote `eslint/no-eval` to `correctness` (#9231) (dalaoshu)
- 542bbd7 linter: Support `import-x` plugin name (#9074) (Sysix)
- cded0ad oxlint: Add `--experimental-nested-config` option (#9152) (camchenry)

### Bug Fixes

- 4ed9d76 linter: Do not use nested configs with `--config` option (#9155) (camchenry)

### Refactor

- 63bb214 oxc: Apply `clippy::redundant_clone` (#9252) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

### Testing

- e49c92d linter: Ensure CLI filters take precedence over nested configs (#9156) (camchenry)

## [0.15.11] - 2025-02-16

### Features

- 5d508a4 linter: Support `env` and `globals` in `overrides` configuration (#8915) (Sysix)

### Bug Fixes

- 47c1649 linter: Output line/column for `--format=stylish` instead of offset + length (#9136) (Sysix)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.15.10] - 2025-02-06

### Features

- 7e8568b linter: Junit reporter (#8756) (Tapan Prakash)

### Bug Fixes

- baf3e4e linter: Correctly replace rule severity with duplicate rule name configurations (#8840) (dalaoshu)

## [0.15.9] - 2025-02-01

### Bug Fixes

- 8ce21d1 linter: Can't disable `no-nested-ternary` rule anymore (#8600) (dalaoshu)
- e929f26 linter: Output `LintCommandInfo` for `CliRunResult::LintNoFilesFound` (#8714) (Sysix)
- 9cc9d5f linter: `ignorePatterns` does not work when files are provided as command arguments (#8590) (dalaoshu)

### Refactor

- 194a5ff linter: Remove `LintResult` (#8712) (Sysix)
- 4a2f2a9 linter: Move default `all_rules` output to trait (#8710) (Sysix)
- 741fb40 linter: Move stdout outside LintRunner (#8694) (Sysix)
- 10e5920 linter: Move finishing default diagnostic message to `GraphicalReporter` (#8683) (Sysix)
- 9731c56 oxlint: Move output from `CliRunResult::InvalidOption` to outside and use more Enums for different invalid options (#8778) (Sysix)
- fe45bee oxlint: Create different `CliRunResult` instead of passing `ExitCode` to it (#8777) (Sysix)
- 2378fef oxlint: Move ConfigFileInit output outside CliRunResult, exit code 1 when it fails (#8776) (Sysix)
- f4cecb5 oxlint: Remove unused `CliRunResult::PathNotFound` (#8775) (Sysix)

### Testing

- ad35e82 linter: Use snapshot testing instead of LintResult (#8711) (Sysix)
- bf895eb linter: Add diagnostic format test snapshots (#8696) (Alexander S.)
- 34d3d72 linter: Add snapshot tester for cli (#8695) (Sysix)
- 0bf2bcf oxlint: Test two real rules with same name but from different plugins (#8821) (dalaoshu)
- 2b83b71 oxlint: Improve disabling "no-nested-ternary" tests (#8814) (Alexander S.)
- 45648e7 oxlint: Fix InvalidOptionTsConfig tests for windows (#8791) (Alexander S.)
- 48bfed9 oxlint: Ignore windows path mismatch (Boshen)
- 6f4a023 oxlint: Remove "--print-config" test (#8792) (Sysix)
- 55c2025 oxlint: Add `CliRunResult` to snapshot (#8780) (Sysix)

## [0.15.8] - 2025-01-24

### Features

- 4ae568e linter: Add DiagnosticResult to the Reporters for receiving a sub part result (#8666) (Alexander S.)
- 8a0eb2a oxlint: Add stylish formatter (#8607) (Andrew Powell)

### Bug Fixes

- 40316af linter: Fix github `endColumn` output (#8647) (Alexander S.)
- dc912fa linter: Added missing $schema property to default config (#8625) (Tapan Prakash)

## [0.15.7] - 2025-01-19

### Features

- 4ac2e99 oxlint: Implement `--init` cli option (#8453) (Tapan Prakash)

### Refactor

- b4c87e2 linter: Move DiagnosticsReporters to oxlint (#8454) (Alexander S.)

## [0.15.6] - 2025-01-13

### Refactor

- 43ed3e1 linter: Add output formatter (#8436) (Alexander S.)
- 4e05e66 linter: Remove glob for windows (#8390) (Alexander S.)
- 3c534ae linter: Refactor `LintBuilder` to prep for nested configs (#8034) (camc314)

## [0.15.5] - 2025-01-02

### Bug Fixes

- 2b14a6f linter: Fix `ignorePattern` config for windows  (#8214) (Alexander S.)

### Testing

- cb709c9 linter: Fix some oxlint tests on windows (#8204) (Cameron)

## [0.15.4] - 2024-12-30

### Bug Fixes

- f3050d4 linter: Exclude svelte files from `no_unused_vars` rule (#8170) (Yuichiro Yamashita)

### Refactor

- 6da0b21 oxlint: Remove unused `git.rs` (#7990) (Boshen)
- 58e7777 oxlint: Remove extra if check in `Walkdir` (#7989) (Boshen)

## [0.15.3] - 2024-12-17

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.15.0] - 2024-12-10

- 39b9c5d linter: [**BREAKING**] Remove unmaintained security plugin (#7773) (Boshen)

### Features


## [0.14.1] - 2024-12-06

### Features

- 275d625 linter: Output rules to json array (#7574) (camc314)

### Bug Fixes

- 9761e94 apps/oxlint: Incorrect matching in `.oxlintignore` (#7566) (dalaoshu)
- 29db060 linter: Detect typescript eslint alias rules (#7622) (Alexander S.)
- 810671a linter: Detect vitest jest alias rules (#7567) (Alexander S.)

## [0.14.0] - 2024-12-01

### Features

- 32f860d linter: Add support for ignorePatterns property within config file (#7092) (Nicholas Rayburn)

## [0.13.1] - 2024-11-23

### Features

- 9558087 oxlint: Auto detect config file in CLI (#7348) (Alexander S.)

### Bug Fixes

- 8507464 linter: Hanging when source has syntax/is flow (#7432) (Cameron)
- e88cf1b linter: Make `overrides` globs relative to config path (#7407) (camchenry)

## [0.13.0] - 2024-11-21

- 878189c parser,linter: [**BREAKING**] Add `ParserReturn::is_flow_language`; linter ignore flow error (#7373) (Boshen)

### Features


## [0.12.0] - 2024-11-20

### Features

- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)
- d3a0119 oxlint: Add `cwd` property to `LintRunner` (#7352) (Alexander S.)

### Bug Fixes

- df5c535 linter: Revert unmatched rule error (#7257) (Cameron A McHenry)

## [0.11.0] - 2024-11-03

- 1f2a6c6 linter: [**BREAKING**] Report unmatched rules with error exit code (#7027) (camchenry)

### Features

- 2184588 linter: Do not bail for unmatched rules yet (#7093) (Boshen)

### Bug Fixes

- 38d1f78 linter: Remove confusing help text for now (#7081) (Cam McHenry)

### Refactor

- a8dc75d linter: Remove unused CLI result types (#7088) (camchenry)

## [0.10.3] - 2024-10-26

### Features

- 0acca58 linter: Support `--print-config all` to print config file for project (#6579) (mysteryven)

## [0.10.2] - 2024-10-22

### Refactor

- 6ffdcc0 oxlint: Lint/mod.rs -> lint.rs (#6746) (Boshen)

### Testing

- b03cec6 oxlint: Add `--fix` test case (#6747) (Boshen)

## [0.10.1] - 2024-10-21

### Refactor

- d6609e9 linter: Use `run_on_jest_node` for existing lint rules (#6722) (camchenry)

## [0.10.0] - 2024-10-18

- 80266d8 linter: [**BREAKING**] Support plugins in oxlint config files (#6088) (DonIsaac)

### Features


## [0.9.10] - 2024-10-07

### Bug Fixes

- 9e9808b linter: Fix regression when parsing ts in vue files (#6336) (Boshen)

### Refactor

- ea908f7 linter: Consolidate file loading logic (#6130) (DonIsaac)

## [0.9.7] - 2024-09-23

### Features

- d24985e linter: Add `oxc-security/api-keys` (#5906) (DonIsaac)

## [0.9.6] - 2024-09-18

### Refactor

- 026ee6a linter: Decouple module resolution from import plugin (#5829) (dalaoshu)

## [0.9.4] - 2024-09-12

### Refactor

- 9e9435f linter: Add `LintFilter` (#5685) (DonIsaac)
- 5ae9b48 linter: Start internal/external split of `OxlintOptions` (#5659) (DonIsaac)
- bac03e3 linter: Make fields of `LintServiceOptions` private (#5593) (DonIsaac)
- 20d0068 oxlint: Move cli-related exports to `cli` module (#5139) (DonIsaac)

## [0.9.3] - 2024-09-07

### Features

- 4473779 linter/node: Implement no-exports-assign (#5370) (dalaoshu)

### Styling
- d8b29e7 Add trailing line breaks to JSON files (#5544) (overlookmotel)

## [0.9.0] - 2024-08-26

- b894d3b linter: [**BREAKING**] Make `no-unused-vars` correctness (#5081) (DonIsaac)

### Features


## [0.7.2] - 2024-08-15

### Documentation

- 955a4b4 oxlint: Improve cli doc regarding fix and `-D all` (Boshen)

## [0.7.0] - 2024-08-05

### Features

- b952942 linter: Add eslint/no-unused-vars (⭐ attempt 3.2) (#4445) (DonIsaac)
- 7afa1f0 linter: Support suggestions and dangerous fixes (#4223) (DonIsaac)

### Bug Fixes

- fe1356d linter: Change no-unused-vars to nursery (#4588) (DonIsaac)
- 72337b1 linter: Change typescript-eslint/no-namespace to restriction (#4539) (Don Isaac)
- 732f4e2 linter: Fix `oxlint` allocator cfg (#4527) (overlookmotel)

## [0.6.1] - 2024-07-17

### Features

- 1f8968a linter: Add eslint-plugin-promise rules: avoid-new, no-new-statics, params-names (#4293) (Jelle van der Waa)

## [0.5.1] - 2024-06-29

### Bug Fixes

- 750cb43 oxlint: Gate custom allocators by feature flag (#3945) (Luca Bruno)

## [0.5.0] - 2024-06-27

### Features

- 328445b linter: Support `vitest/no-disabled-tests` (#3717) (mysteryven)

### Bug Fixes

- 5902331 oxlint: Properly report error (#3889) (Luca Bruno)

## [0.4.2] - 2024-05-28

### Refactor

- 21505e8 cli: Move crates/oxc_cli to apps/oxlint (#3413) (Boshen)

