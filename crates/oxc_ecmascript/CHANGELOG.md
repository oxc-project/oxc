# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

