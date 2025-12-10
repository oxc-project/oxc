# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.102.0] - 2025-12-08

### 🚀 Features

- c90f053 minfier: Support `.` separated values for `compress.treeshake.manualPureFunctions` (#16529) (sapphi-red)

## [0.99.0] - 2025-11-24

### 💥 BREAKING CHANGES

- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

## [0.97.0] - 2025-11-11

### 🐛 Bug Fixes

- 8b14ec9 minifier: Handle `{ __proto__: null } instanceof Object` correctly (#15217) (sapphi-red)






## [0.91.0] - 2025-09-22

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)







## [0.86.0] - 2025-08-31

### 🚀 Features

- f97283b ecmascript: Support more cases for IsLiteralValue with `include_functions` (#13425) (sapphi-red)



## [0.84.0] - 2025-08-30

### 🚀 Features

- 95d4311 ecmascript: Check side effects inside static blocks (#13404) (sapphi-red)
- c557854 ecmascript: Support MayHaveSideEffects for Statements (#13402) (sapphi-red)


## [0.83.0] - 2025-08-29

### 💥 BREAKING CHANGES

- e459866 ecmascript: [**BREAKING**] Remove `PropertyReadSideEffects::OnlyMemberPropertyAccess` (#13348) (sapphi-red)

### 🚀 Features

- ac2067b ecmascript: Examine and improve `is_literal_value` (#13363) (sapphi-red)
- 8e34656 ecmascript: Implement MayHaveSideEffects for AssignmentTarget (#13279) (sapphi-red)
- a1b6ad4 minifier: Implement known methods `Math.clz32` and `Math.imul` (#12405) (Ethan Wu)

### 🐛 Bug Fixes

- 5d898e8 ecmascript: Treat shadowed global calls correctly (#13292) (sapphi-red)

### 🚜 Refactor

- 66a5673 ecmascript: Add `ToUint32` trait (#13272) (sapphi-red)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- d27a04b ecmascript: Skip array length evaluation if there are any spread elements (#13162) (Monad)


## [0.82.2] - 2025-08-17

### 🚀 Features

- fbe6663 minifier: Mark more known global methods as side-effect free (#13086) (Boshen)
- 36386e4 ecmascript: Treat `[...arguments]` as side effect free (#13116) (sapphi-red)
- fe4589b minifier: Mark more global constructors as side-effect free (#13082) (Boshen)

### 🚜 Refactor

- e190ee5 minifier: Clean up `remove_unused_expression` (#13080) (Boshen)


## [0.82.1] - 2025-08-13

### 📚 Documentation

- 9c05e2f ecmascript: Correct docs for `GlobalContext::is_global_reference` (#13022) (overlookmotel)


## [0.82.0] - 2025-08-12

### 🚀 Features

- 54d1750 ecmascript: Handle `typeof` guarded global access as side effect free (#12981) (Copilot)
- 33c0e9f ecmascript: Add global `isNaN`, `isFinite`, `parseFloat`, `parseInt` functions support to constant evaluation (#12954) (Copilot)
- 208e6f7 ecmascript: Add URI encoding/decoding support to constant evaluation (#12934) (Copilot)
- 53f7a9f minifier: `new Date()` has `ValueType::Object` (#12951) (Boshen)
- 784796d minifier: Fold `(!0).toString()` to `true` (#12938) (Boshen)
- bc1d716 ecmascript: Add ARM64 FJCVTZS instruction optimization for ToInt32 with function-specific target features and runtime detection (#12823) (copilot-swe-agent)

### 🐛 Bug Fixes

- c72f49e ecmascript: Fix merge error (Boshen)

### 🚜 Refactor

- 8a5c9b9 minifier,ecmascript: Clean up `is_global_reference` (#12953) (Boshen)
- 0c5bffc ecmascript: Change `IsGlobalReference` to `GlobalContext` (#12952) (Boshen)
- c072e01 all: Add missing lifetimes in function return types (#12895) (overlookmotel)

### 📚 Documentation

- 3ce27e9 ecmascript, minifier: Revert changes to changelogs (#12962) (overlookmotel)


## [0.81.0] - 2025-08-06

### 🚜 Refactor

- f3f6012 ecmascript: Avoid redundant checks by splitting up match arms (#12782) (Ulrich Stark)


## [0.80.0] - 2025-08-03

### 🚜 Refactor

- 5f50bc3 minifier: Move string method constant evaluation from minifier to ecmascript crate (#12672) (Copilot)

### 📚 Documentation

- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)


## [0.79.1] - 2025-07-31

### 🚀 Features

- 763a618 minifier: Inline small constant values (#12639) (Boshen)


## [0.79.0] - 2025-07-30

### 🚀 Features

- b877039 minifier: Inline `const` variables that are only used once (#12488) (Boshen)

### 🐛 Bug Fixes

- fe9c8e1 minifier: Do not remove non-plain empty functions (#12573) (Boshen)





## [0.77.1] - 2025-07-16

### 🚜 Refactor

- c68b607 ast: Rename `TemplateLiteral::quasi` to `TemplateLiteral::single_quasi` (#12266) (Dunqing)
- 32c32af ast: Check whether there is a single `quasi` in `TemplateLiteral::quasi` (#12265) (Dunqing)


## [0.77.0] - 2025-07-12

### 🚜 Refactor

- d5c94a8 ecmascript: Move `is_less_than` to its own file (#12189) (Boshen)





## [0.74.0] - 2025-06-23

### 💥 BREAKING CHANGES

- 7a05e71 minifier: [**BREAKING**] Add `Treeshake` options (#11786) (Boshen)

### 🚀 Features

- d462ead minifier: Remove dead code that evaluates to a constant value (#11788) (Boshen)

### 🚜 Refactor

- 5a46641 ecmascript: Move `get_constant_value_for_reference_id` to `IsGlobalReference` trait (#11810) (Boshen)




## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- 40ac186 minifier: Annotate more pure constructors (#11555) (Boshen)

### ⚡ Performance

- 365d8e5 ecmascript: Faster parsing integers (#11565) (overlookmotel)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.69.0] - 2025-05-09

### Refactor

- 7c6ac7c ecmascript: Remove outdated todo comment (#10909) (Ulrich Stark)

## [0.62.0] - 2025-04-01

- 4077868 ecmascript: [**BREAKING**] Introduce MayHaveSideEffectsContext (#10126) (sapphi-red)

### Features

- 11ddda6 ecmascript: Add `unknown_global_side_effects` to `MayHaveSideEffectsContext` (#10130) (sapphi-red)
- ef3451c ecmascript: Add `property_read_side_effects` to `MayHaveSideEffectsContext` (#10129) (sapphi-red)
- ac2ecbb ecmascript: Add `is_pure_call` to `MayHaveSideEffectsContext` (#10128) (sapphi-red)
- e004a3f ecmascript: Add `respect_annotations` to `MayHaveSideEffectsContext` (#10127) (sapphi-red)

### Refactor


## [0.57.0] - 2025-03-11

### Features

- b6deff8 ecmascript: Support integer index access for array and string in `may_have_side_effects` (#9603) (sapphi-red)

### Bug Fixes

- 96eef8b ecmascript: `(foo() + "").length` may have a side effect (#9605) (sapphi-red)

## [0.56.4] - 2025-03-07

### Refactor

- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

## [0.54.0] - 2025-03-04

### Features

- 64f4a82 ecmascript: Handle pure call expression in chain expressions (#9480) (sapphi-red)
- 32139d2 ecmascript: Support `/* @__PURE__ */` in may_have_side_effects (#9409) (sapphi-red)

### Bug Fixes

- f5bbd5d ecmascript: Fix toString for negative numbers (#9508) (sapphi-red)
- d2cd975 ecmascript: Fix may_have_side_effects for `${a() === b}` (#9478) (sapphi-red)
- 584d847 ecmascript: Objects passed to template literals may have side effects (#9425) (sapphi-red)

### Testing

- c187b11 ecmascript: Add comments and tests for cases where `ToPropertyKey` throws an error (#9429) (sapphi-red)

## [0.53.0] - 2025-02-26

### Features

- e10fb97 ecmascript: Improve may_have_side_effects for `.length` (#9366) (sapphi-red)
- 35e5ca9 ecmascript: Improve may_have_side_effects for `instanceof` (#9365) (sapphi-red)
- 11012c6 ecmascript: Improve ValueType for coalesce operator (#9354) (sapphi-red)
- b7998fd ecmascript: To_number for object without toString (#9353) (sapphi-red)
- e51d563 minifier: Concatenate strings with template literals on right side (#9356) (sapphi-red)

### Bug Fixes

- f5c8698 ecmascript: Correct may_have_side_effects for classes (#9367) (sapphi-red)
- d3ed128 minifier: Do not remove `=== 0` if the lhs can be NaN (#9352) (sapphi-red)

### Refactor

- faf966f ecmascript: Don't check side effects in constant_evaluation (#9122) (sapphi-red)

## [0.52.0] - 2025-02-21

### Documentation

- 3414824 oxc: Enable `clippy::too_long_first_doc_paragraph` (#9237) (Boshen)

### Refactor

- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.51.0] - 2025-02-15

### Features

- 36c8640 ecmascript: Support more string concatenation (#9121) (sapphi-red)
- 6936b08 ecmascript: Fold `typeof` with ValueType information (#9086) (sapphi-red)
- 125d610 minifier: Fold String::charAt / String::charCodeAt more precisely (#9082) (sapphi-red)
- 237ffba minifier: Fold bitwise binary expressions with negative BigInts (#9081) (sapphi-red)

### Bug Fixes

- eb7cd62 ecmascript: To_number for shadowed undefined (#9106) (sapphi-red)
- 8cbdf00 ecmascript: To_boolean for shadowed undefined (#9105) (sapphi-red)
- 17c745c ecmascript: To_string for object with toString (#9104) (sapphi-red)
- cfc71f9 ecmascript: To_string for shadowed undefined (#9103) (sapphi-red)
- 2ab2a8f ecmascript: Handle shadowed global variables in `ValueType` (#9085) (sapphi-red)

### Refactor

- 8bd6eef ecmascript: Merge constant evaluation logics (#9120) (sapphi-red)
- b164072 ecmascript: Extract to_numeric (#9111) (sapphi-red)
- fc53cdd ecmascript: Generalize ToPrimitive (#9109) (sapphi-red)
- d951390 ecmascript: Use value_type in to_primitive (#9108) (sapphi-red)
- 8f79012 ecmascript: Pass IsGlobalReference to DetermineValueType instead of extending it (#9107) (sapphi-red)
- db1744c ecmascript: Remove "constant_evaluation" / "side_effects" features (#9114) (sapphi-red)
- 329de94 ecmascript: Extract ToPrimitive (#9102) (sapphi-red)
- d670ec7 ecmascript: Pass IsGlobalReference to MayHaveSideEffects instead of extending it (#9101) (sapphi-red)
- f4e2d4e ecmascript: Allow IsGlobalReference to return None (#9100) (sapphi-red)

## [0.49.0] - 2025-02-10

### Features

- ad1a878 ecmascript: Support BigInt comparison (#9014) (sapphi-red)
- d515cfd ecmascript: Detect objects without overridden `toString`/`valueOf`/`Symbol.toPrimitive` (#8993) (sapphi-red)
- 2a10a99 ecmascript: Support arrays and objects for unary expression may_have_side_effects (#8990) (sapphi-red)
- dd383c0 ecmascript: `ValueType::from` for PrivateInExpression (#8964) (sapphi-red)
- c3eef2f ecmascript: `ValueType::from` for parenthesized expressions (#8962) (sapphi-red)
- 2c3a46d ecmascript: Support more simple expressions by `ValueType::from` (#8961) (sapphi-red)
- 4cec83f ecmascript: `ValueType::from` for bitwise not operator (#8955) (sapphi-red)
- e3e9999 ecmascript: Complete may_have_side_effects (#8855) (sapphi-red)
- ca4831b minifier: Fold `typeof class {}` to `'function'` (#8949) (sapphi-red)
- 9ffe9e9 minifier: Fold `typeof (() => {})` to `'function'` (#8948) (sapphi-red)
- 36007de minifier: Fold typeof `{ foo }` when `foo` is declared (#8947) (sapphi-red)
- 56575b2 minifier: Fold complicated array literals passed to unary `+` (#8944) (sapphi-red)
- 14462be minifier: Fold simple literals passed to unary `+` (#8943) (sapphi-red)
- d6d13dd minifier: Minimize `!!(boolean_expr)` -> `boolean_expr` (#8849) (Boshen)

### Bug Fixes

- 9a5a926 ecmascript: Fix may_have_side_effects for binary expressions (#8991) (sapphi-red)
- 660c314 ecmascript: Fix may_have_side_effects for unary expressions (#8989) (sapphi-red)
- 8ab7204 ecmascript: Fix `ValueType::from` for `AssignmentExpression` (#8959) (sapphi-red)
- aeb122d ecmascript: Fix `ValueType::from` for numeric binary operators (#8956) (sapphi-red)
- 1182c20 ecmascript: `ValueType::from` for unknown value should return Undetermined instead of Number (#8954) (sapphi-red)
- b5a7785 minifier: Fix comparison of strings with unicode characters (#8942) (sapphi-red)

### Refactor

- 9b5d800 minifier: Move equality comparison to ConstantEvaluation (#9009) (sapphi-red)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- cc7bb9c ecmascript: Add test for `ValueType` and update document (#8951) (sapphi-red)

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

