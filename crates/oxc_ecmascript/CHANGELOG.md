# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.2] - 2025-02-02

### Bug Fixes

- ae7f670 minifier: Avoid minifying `+void unknown` to `NaN` and fix typo (#8784) (7086cmd)

## [0.48.1] - 2025-01-26

### Refactor

- 58002e2 ecmascript: Remove the lifetime annotation on `MayHaveSideEffects` (#8717) (Boshen)
- 0af0267 minifier: Side effect detection needs symbols resolution (#8715) (Boshen)

### Testing

- 03229c5 minifier: Fix broken tests (#8722) (Boshen)

## [0.48.0] - 2025-01-24

### Bug Fixes

- 4ff6e85 minifier: Remove expression statement `void 0` (#8602) (Boshen)

### Performance

- 9953ac7 minifier: Add `LatePeepholeOptimizations` (#8651) (Boshen)

## [0.47.1] - 2025-01-19

### Bug Fixes

- 7b219a9 minifier: Fix dce shadowed undefined (#8582) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- 927f43f minifier: Improve `.charCodeAt(arg)` when arg is valid (#8534) (Boshen)

### Bug Fixes

- b1d0186 minifier: Do not fold `!!void b` (#8533) (Boshen)

### Refactor

- 8f57929 minifier: Merge `try_compress_type_of_equal_string` into `try_minimize_binary` (#8561) (sapphi-red)

## [0.46.0] - 2025-01-14

### Bug Fixes

- 1d6e84d minifier: Fix incorrect `null.toString()` and `1n.toString()` (#8464) (Boshen)

## [0.45.0] - 2025-01-11

### Features

- 438a6e7 minifier: Minimize conditions in boolean context (#8381) (Boshen)
- e88a6bd minifier: Minimize `!0 + null !== 1` -> `!0 + null != 1` (#8332) (Boshen)
- 922c514 minifier: Fold `.toString()` (#8308) (Boshen)
- 66a2443 minifier: Minify sequence expressions (#8305) (camc314)
- f000596 minifier: Minify call expressionsto `Number` (#8267) (camc314)
- cec63e2 minifier: `{}` evals to `f64::NaN` (Boshen)
- 4d8a08d minifier: Improve constant evaluation (#8252) (Boshen)
- bd8d677 minifier: Minimize `~undefined`, `~null`, `~true`, `~false` (#8247) (Boshen)
- f73dc9e minifier: Constant fold `'x'.toString()` and `true.toString()` (#8246) (Boshen)
- fc43ec5 minifier: Fold `string.length` / `array.length` (#8172) (sapphi-red)
- 6615e1e minifier: Constant fold `instanceof` (#8142) (翠 / green)
- ad9a0a9 mininifier: Minimize variants of `a instanceof b == true` (#8241) (Boshen)

### Bug Fixes

- 74572de ecmascript: Incorrect `to_int_32` value for Infinity (#8144) (翠 / green)
- 0efc845 minifier: `+0n` produces `TypeError` (#8410) (Boshen)
- 7ce6a7c minifier: `a in b` has error throwing side effect (#8406) (Boshen)
- c0a3dda minifier: `instanceof` has error throwing side effect (#8378) (Boshen)
- 5516f7f minifier: Do not fold object comparisons (#8375) (Boshen)
- 05be1fc minifier: Remove incorrect fold `Expression::AssignmentExpression` (#8211) (Boshen)
- 56b7f13 minifier: Do not constant fold `0 instanceof F` (#8199) (Boshen)

### Refactor

- 1835687 ecmascript: Remove unnecessary `use` statement (#8284) (overlookmotel)
- 9a5c66a minifier: Clean up (#8346) (Boshen)

## [0.44.0] - 2024-12-25

### Features

- 5397fe9 minifier: Constant fold `undefined?.bar` -> `undefined` (#8075) (Boshen)

## [0.42.0] - 2024-12-18

### Features

- 075bd16 minifier: Fold bitwise operation (#7908) (翠 / green)

### Bug Fixes

- 4799471 minfier: Bigint bitwise operation only works with bigint (#7937) (Boshen)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.39.0] - 2024-12-04

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features

- 24189f2 ecma: Implement array join method (#6936) (7086cmd)

### Testing

- 9d6e14b ecmascript: Move tests to `oxc_minifier` due to cyclic dependency with `oxc_parser` (#7542) (Boshen)

## [0.35.0] - 2024-11-04

### Bug Fixes

- da199c7 ecmascript: Allow getting PropName for object methods (#6967) (camchenry)

## [0.34.0] - 2024-10-26

### Features

- 4429754 ecmascript: Constant eval `null` to number (#6879) (Boshen)
- fd57e00 ecmascript: Add abstract_relational_comparison to dce (#6846) (Boshen)
- 8bcaf59 minifier: Late peeophole optimization (#6882) (Boshen)
- fccf82e minifier: Implement folding `substring` string fns (#6869) (camc314)
- e6a5a1b minifier: Implement folding `charCodeAt` string fns (#6475) (camc314)

### Bug Fixes

- a47c70e minifier: Fix remaining runtime bugs (#6855) (Boshen)
- 686727f minifier: Reference read has side effect (#6851) (Boshen)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

### Refactor

- 8b25131 minifier: Binary operations use `ConstantEvaluation` (#6700) (Boshen)

## [0.32.0] - 2024-10-19

### Features

- 15c04e5 ecmascript: Add feature flag for constant evaluation (Boshen)
- d11770d ecmascript: Add `StringToNumber` (#6576) (Boshen)
- e561880 ecmascript: Add constant_evaluation and side_effects code (#6550) (Boshen)
- 3556062 ecmascript: Add `ConstantEvaluation` (#6549) (Boshen)
- 39c2e66 ecmascript: Add `ToBigInt` and `StringToBigInt` (#6508) (Boshen)
- 6f22538 ecmascript: Add `ToBoolean`, `ToNumber`, `ToString` (#6502) (Boshen)
- 071e564 minifier: Finish implementing folding object expressions (#6586) (camc314)
- 096e590 minifier: Implement folding `charAt` string fns (#6436) (camc314)

### Refactor

- aa6ba24 ecmascript: Improve string to number conversion (#6577) (magic-akari)
- 6d041fb ecmascript: Remove `NumberValue` (#6519) (Boshen)
- 856cab5 ecmascript: Move ToInt32 from `oxc_syntax` to `oxc_ecmascript` (#6471) (Boshen)
- 1ba2a24 ecmascript: Remove `HasProto` which is not part of the spec (#6470) (Boshen)
- f4cdc56 minifier: Use constant folding unary expression from `oxc_ecmascript` (#6647) (Boshen)
- bbca743 minifier: Move string methods to `oxc_ecmascript` (#6472) (Boshen)

## [0.31.0] - 2024-10-08

### Features

- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

