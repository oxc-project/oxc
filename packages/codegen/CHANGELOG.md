# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.155.0] - 2026-10-19

### ⚡ Performance

- 2ab6b63 packages/codegen: Remove redundant `printSpaceBeforeIdentifier` and `last` updates in `printParams` (#27652) (overlookmotel)
- e533278 codegen: Reduce writes in `printDecorators` (#27651) (overlookmotel)

### 📚 Documentation

- 3de03c1 packages/codegen: Correct outdated comments (#27838) (overlookmotel)

## [0.154.0] - 2026-10-13

### 🐛 Bug Fixes

- 760b60b codegen, packages/codegen: Remove extraneous space before `RestElement` in `ArrayPattern` (#27613) (overlookmotel)
- 0353d37 packages/codegen: Support printing `V8IntrinsicExpression`s (#27597) (overlookmotel)
- f50257e codegen: Avoid extra indent before empty do-while bodies (#27407) (camc314)
- 5461c1a packages/codegen: Fix printing negative `BigIntLiteral` after `-` (#27377) (overlookmotel)
- bebef88 packages/codegen: Fix debug assert fail printing `TSJSDocNonNullableType` (#27373) (overlookmotel)

### ⚡ Performance

- e1c97f8 packages/codegen: Remove all `printSpaceBeforeIdentifier` calls (#27620) (overlookmotel)
- 70ecfdd packages/codegen: Skip redundant `printSpaceBeforeIdentifier` call (#27619) (overlookmotel)
- 3c289f3 packages/codegen: Remove branches from loops printing arrays (#27614) (overlookmotel)
- 27f5b45 packages/codegen: Remove branch printing `RestElement` in `ArrayPattern` (#27608) (overlookmotel)
- adc90ed packages/codegen: Reduce branches in `printArguments` (#27607) (overlookmotel)
- 262dd21 packages/codegen: Remove unnecessary `printSpaceBeforeIdentifier` calls (#27606) (overlookmotel)
- 4bb9722 packages/codegen: Print `SpreadElement` arguments via `printExpression` (#27599) (overlookmotel)
- 0c98dc5 packages/codegen: Reduce megamorphic `type` reads in `printParams` (#27595) (overlookmotel)
- e3780b5 packages/codegen: Reduce writes in `printForOfStatement` (#27594) (overlookmotel)
- 8d0ba20 packages/codegen: Reduce megamorphic `type` reads in `printForOfStatement` (#27593) (overlookmotel)
- b29f35e packages/codegen: Reduce megamorphic `type` reads in `printDirectivesAndStatements` (#27592) (overlookmotel)
- 256259d packages/codegen: Reduce megamorphic `type` reads in `printConditionalExpression` (#27591) (overlookmotel)
- a9bd942 packages/codegen: Reduce megamorphic `type` reads in `printObjectProperty` (#27590) (overlookmotel)
- 0a386ff packages/codegen: Ensure `shorthand` var is always a boolean (#27589) (overlookmotel)
- 8d4c633 packages/codegen: Reduce branches in `printObjectProperty` (#27588) (overlookmotel)
- b4c8c04 packages/codegen: Reduce megamorphic `type` reads in `printMemberExpression` (#27587) (overlookmotel)
- ff5eae8 packages/codegen: Reduce megamorphic `type` reads (#27586) (overlookmotel)
- 961f318 packages/codegen: Reduce megamorphic `type` reads in `binCheckAndPrepare` (#27460) (overlookmotel)
- 3d2f93f packages/codegen: Reduce megamorphic property accesses for `BinaryExpression`s (#27459) (overlookmotel)
- 675ec5e packages/codegen: Reduce megamorphic property accesses in `printBinaryish` (#27458) (overlookmotel)
- b0dbfff packages/codegen: Reduce writes and branches printing functions (#27455) (overlookmotel)
- 5f3865c packages/codegen: Reduce writes for optional `MemberExpression`s (#27454) (overlookmotel)

### 📚 Documentation

- 59472c5 packages/codegen: Add `@param`s to JSDoc comments which had none (#27601) (overlookmotel)
- 5ee0fc9 packages/codegen: Complete `@param` lists in partly documented JSDoc comments (#27600) (overlookmotel)
- 12dfa25 packages/codegen: Correct doc comment for `printBinaryish` (#27456) (overlookmotel)

## [0.151.0] - 2026-09-21

### 🚀 Features

- 1fac655 coverage: Support new TypeScript fixture layout (#26226) (camc314)

## [0.150.0] - 2026-09-14

### 🐛 Bug Fixes

- 32d00c5 codegen: Preserve instantiation expression precedence (#26424) (camc314)
- 5986187 packages/codegen: Preserve private-in right operand precedence (#26420) (camc314)
- f8e6c6c packages/codegen: Preserve in restriction through yield arguments (#26421) (camc314)
- 6e15ad5 packages/codegen: Print matching quoted import names as identifiers (#26404) (camc314)
- bbbb4bc packages/codegen: Preserve private-in left operand precedence (#26403) (camc314)
- a111b5b packages/codegen: Print accessibility modifiers before abstract (#26402) (camc314)

## [0.148.0] - 2026-08-31

### ⚡ Performance

- 07a0793 packages/codegen: Flatten output in chunks (#26109) (overlookmotel)
- 487427a packages/codegen: Faster string flattening (#26108) (overlookmotel)
- 7785583 packages/codegen: Ensure indent strings are flattened (#26107) (overlookmotel)
- 0dd4db3 packages/codegen: Store mapping positions in an `Int32Array` (#26085) (overlookmotel)

## [0.147.0] - 2026-08-24

### 🐛 Bug Fixes

- 6c5ad1b codegen: Only add mapping names for `PrivateIdentifier`s which have changed (#25958) (overlookmotel)
- a185ac3 codegen: Print `#` before private identifier in TS signature key (#25938) (overlookmotel)

### ⚡ Performance

- 53f6270 packages/codegen: Eagerly initialize `mapNames` (#25979) (overlookmotel)
- 5b43c60 packages/codegen: Split recording mappings into multiple functions (#25978) (overlookmotel)
- cc947b5 packages/codegen: Reduce property lookups (#25977) (overlookmotel)
- c22a9cc packages/codegen: Combine condition checks (#25976) (overlookmotel)
- 215a6a5 packages/codegen: Add functions for writing/marking unnamed mappings (#25975) (overlookmotel)
- 1004439 packages/codegen: Merge `mark` and `write` calls in `printImportAttributes` (#25972) (overlookmotel)
- 94c8a34 packages/codegen: Remove `node` param from `printString` and `printNonNegativeFloat` (#25971) (overlookmotel)
- 7e2b4b3 packages/codegen: `printNonNegativeFloat` write mapping directly (#25968) (overlookmotel)
- 53ff080 packages/codegen: Remove `lastWasPostfixClose` (#25888) (overlookmotel)

### 📚 Documentation

- 07de646 packages/codegen: Fix JSDoc comments on `write*` methods (#25966) (overlookmotel)
- 3b09c04 packages/codegen: Clarify comment (#25959) (overlookmotel)

## [0.146.0] - 2026-08-19

### 🐛 Bug Fixes

- 526c2b3 codegen: Remove deprecated test APIs (#25889) (camc314)
- 344a9b2 codegen: Align enum template sourcemaps (#25872) (camc314)

### ⚡ Performance

- 8783524 packages/codegen: Remove loop from `growMappingBuffer` (#25886) (overlookmotel)
- b6b14c0 packages/codegen: Remove redundant branch (#25885) (overlookmotel)
- be6adf0 packages/sourcemap: Make `location` passed to `recordSourceMapping` always a number (#25884) (overlookmotel)
- f720f15 packages/codegen: Take named-mapping only code off hot path in `recordSourceMapping` (#25883) (overlookmotel)
- a734af5 packages/codegen: Remove redundant setting of `last` in `printImportAttributes` and `printIf` (#25882) (overlookmotel)
- c6e44c6 packages/codegen: Combine 2 writes in `printJSXElement` (#25881) (overlookmotel)
- 8801662 packages/codegen: Remove redundant mapping from `printParams` (#25880) (overlookmotel)

### 📚 Documentation

- 5ad330e packages/codegen: Reformat comments (#25876) (overlookmotel)

## [0.145.0] - 2026-08-18

### 💥 BREAKING CHANGES

- 365274e packages/codegen: [**BREAKING**] `printSync` return an object (#25720) (overlookmotel)

### 🚀 Features

- 6a7eb60 packages/codegen: Alter capitalization of `sourceFilename` option (#25854) (overlookmotel)
- 300763f packages/codegen: Add full source map support (#25585) (camc314)
- a4478e9 codegen: Add `oxc-codegen` package (#25488) (overlookmotel)

### 🐛 Bug Fixes

- 0fbcf64 codegen: Validate sourcemap options (#25860) (camc314)
- 4cc7ea4 codegen: Reject invalid indent options (#25807) (camc314)
- 0c68b7f estree: Emit `decorators` on `FormalParameterRest` (#25582) (camc314)
- 59e8895 codegen: Validate starting indent level (#25550) (camc314)

### ⚡ Performance

- 5444cbf codegen: Skip escaping harmless `<` tokens (#25564) (camc314)

### 📚 Documentation

- 58f7ab9 packages/codegen: Reformat docs and comments (#25848) (overlookmotel)
- ce03ac1 packages/codegen: Fix JSDoc comments for `printSync` (#25722) (overlookmotel)
- 386a699 packages/codegen: Correct JSDoc comment (#25716) (overlookmotel)
- fd62354 codegen: Revamp package readme (#25560) (camc314)
- ffa3153 codegen: Clarify raw transfer Node requirement (#25565) (camc314)
- c5d4063 codegen: Clarify binary walk allocations (#25562) (camc314)
- 5703d7a codegen: Correct stale printer comments (#25561) (camc314)

