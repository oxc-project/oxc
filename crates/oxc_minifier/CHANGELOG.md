# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.102.0] - 2025-12-08

### 🚀 Features

- d6d2bcd minifier: Remove unused function calls that are marked by `manual_pure_functions` (#16534) (sapphi-red)
- c90f053 minfier: Support `.` separated values for `compress.treeshake.manualPureFunctions` (#16529) (sapphi-red)

## [0.100.0] - 2025-12-01

### 🐛 Bug Fixes

- 6b54dab minifier: Incorrect non-null object condition simplification with `&&` and `||` (#16161) (sapphi-red)
- 9cc20a1 minifier: Avoid merging side effectful expressions to next assignment statement if the side effect may change the left hand side reference (#16165) (sapphi-red)

## [0.99.0] - 2025-11-24

### 🐛 Bug Fixes

- f386efc minifier: Avoid generating invalid spans (#15778) (sapphi-red)

## [0.98.0] - 2025-11-17

### 🚀 Features

- 56e7e44 minifier: Disable removal of unnecessary `use strict` directives for DCE (#15691) (sapphi-red)
- 68703b9 minifier: Rotate binary expressions to remove parentheses (#15473) (sapphi-red)

### 🐛 Bug Fixes

- 440a977 ast: Include rest properties when using `get_binding_identifiers` (#15710) (camc314)

## [0.97.0] - 2025-11-11

### 🐛 Bug Fixes

- 8b14ec9 minifier: Handle `{ __proto__: null } instanceof Object` correctly (#15217) (sapphi-red)

## [0.96.0] - 2025-10-30

### 🚀 Features

- c51c036 minifier: Handle direct eval calls (#15067) (sapphi-red)
- d09c7ee minifier: Add `drop_labels` feature (#14634) (sapphi-red)


## [0.95.0] - 2025-10-15

### 🚀 Features

- bce31b5 napi/playground: Call `with_private_member_mappings()` for private class member mangling (#14380) (copilot-swe-agent)

### 🐛 Bug Fixes

- 1bf83eb minifier: Bail out `arguments` copy loop substitution if the temporary variables are referenced outside the for loop (#14613) (sapphi-red)


## [0.94.0] - 2025-10-06

### 🚀 Features

- 6123684 minifier: Inline single-use variable past read-only variables (#14184) (sapphi-red)

### 🐛 Bug Fixes

- c257b41 mangler: Avoid reusing same mangled names in the outer class (#14362) (sapphi-red)
- fc519c8 mangler: Mangle private class members in subsequent classes correctly (#14361) (sapphi-red)
- 6b2daa8 minifier: Don't inline single use variable in conditional logical expressions (#14185) (sapphi-red)
- e4e963b minifier: Remove `continue` in the end of for-in / for-of (#14186) (sapphi-red)
- 353c001 minifier: Keep private class members used in nested classes (#14217) (sapphi-red)
- b83ffe5 mangler: Mangle private class members used in nested classes properly (#14218) (sapphi-red)

### 🚜 Refactor

- 11dd63b minifier: Use `oxc_ast::NONE` (#14322) (sapphi-red)


## [0.93.0] - 2025-09-28

### 🚀 Features

- 402e6c7 minifier: Inline single use variables within the same variable declarations (#13959) (sapphi-red)
- 1337811 minifier: Evaluate `.concat` calls that has subsequent method calls (#14074) (sapphi-red)

### 🐛 Bug Fixes

- 358f2fc minifier: Remove continue in for-in / for-of (#14151) (夕舞八弦)
- d02d750 mangler: Mangle non top-level `exports` variable (#14169) (sapphi-red)


## [0.92.0] - 2025-09-24

### 🚀 Features

- 0fe4d95 mangler: Mangle private class members (#14027) (sapphi-red)
- aac45ef minifier: Remove unused private class members (#14026) (sapphi-red)

### ⚡ Performance

- c0ef5f3 minifier: Use oxc_data_structures::stack::Stack for ClassSymbolsStack (#14029) (sapphi-red)


## [0.91.0] - 2025-09-22

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🚀 Features

- b2b2037 minifier: Only apply `arguments` copy loop transformation in functions (#13952) (sapphi-red)
- fa76365 minifier: Only apply `arguments` copy loop transformation in strict mode (#13951) (sapphi-red)
- 638416e tasks/coverage: Add node compat table tests for minifier (#13925) (sapphi-red)

### 🐛 Bug Fixes

- 5198a01 minifier: Handle __proto__ when inlining single-use variables (#13926) (sapphi-red)

### 📚 Documentation

- 4817021 minifier: Clarify assumptions (#13950) (sapphi-red)
- f1862c4 minifier: Add comprehensive documentation for oxc_minifier (#13938) (Boshen)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-21

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🚀 Features

- b2b2037 minifier: Only apply `arguments` copy loop transformation in functions (#13952) (sapphi-red)
- fa76365 minifier: Only apply `arguments` copy loop transformation in strict mode (#13951) (sapphi-red)
- 638416e tasks/coverage: Add node compat table tests for minifier (#13925) (sapphi-red)

### 🐛 Bug Fixes

- 5198a01 minifier: Handle __proto__ when inlining single-use variables (#13926) (sapphi-red)

### 📚 Documentation

- 4817021 minifier: Clarify assumptions (#13950) (sapphi-red)
- f1862c4 minifier: Add comprehensive documentation for oxc_minifier (#13938) (Boshen)



## [0.89.0] - 2025-09-15

### 🐛 Bug Fixes

- f0af9a4 minifier: Don't inline single use variables that are not literals to for statement initializers (#13769) (sapphi-red)

### 🧪 Testing

- fcc3663 minifier: Merge variable declarations into for statement initializers (#13770) (sapphi-red)


## [0.88.0] - 2025-09-15

### 🚀 Features

- a49d7cf minifier: Remove `typeof` guarded global access expressions (#13751) (sapphi-red)
- c364ad1 minifier: Support ForStatements for single use variable inlining (#13755) (sapphi-red)
- c868796 minifier: Remove unused variable declarations in dead code (#13754) (sapphi-red)

### 🐛 Bug Fixes

- 3d895cf minifier: Remove unused long array expressions (#13752) (sapphi-red)
- f9fd65b minifier: Disallow merging assignments to let declarations when TDZ error would be introduced (#13635) (sapphi-red)


## [0.87.0] - 2025-09-08

### 🚀 Features

- 2cd4c7b minifier: Inline constant that is declared in normal for statement initializer (#13509) (sapphi-red)
- 52f3e89 minifier: Remove unused variables in for init (#13508) (sapphi-red)
- b4dfddd minifier: Store symbol information for for init variables (#13507) (sapphi-red)
- 05def8c minifier: Constant fold `RegExp.prototype.source` (#13472) (sapphi-red)
- 78dcfc6 minifier: Return total iterations ran for DCE as well (#13476) (sapphi-red)
- ecf69bb minifier: Respect `--max-iterations` for DCE as well (#13475) (sapphi-red)

### 🐛 Bug Fixes

- 34d3cde rust: Fix clippy issues (#13540) (Boshen)
- 946669b minifier: Inline multiple variable declarations at once (#13477) (sapphi-red)

### ⚡ Performance

- 60dd9c9 minifier: Prealloc template exprs vec (#13410) (camchenry)


## [0.86.0] - 2025-08-31

### 🚀 Features

- 2931356 minifier: Inline single use functions past sideeffectful expressions (#13426) (sapphi-red)
- f97283b ecmascript: Support more cases for IsLiteralValue with `include_functions` (#13425) (sapphi-red)

### 🐛 Bug Fixes

- 68b9b33 minifier: Set proper scope information for injected if blocks (#13444) (sapphi-red)
- 73b93ce minifier: Set reference_id when removing `window.` from `window?.Object` (#13442) (sapphi-red)
- 6d0e355 minifier: Avoid inlining single use variables when the name needs to be preserved (#13422) (sapphi-red)

### 🚜 Refactor

- 46fc83d minifier: Use `.reference_id()` instead of `.reference_id` (#13443) (sapphi-red)



## [0.84.0] - 2025-08-30

### 🚀 Features

- 95d4311 ecmascript: Check side effects inside static blocks (#13404) (sapphi-red)
- 2d0414d minifier: Remove pure function calls when the return value is not used (#13403) (sapphi-red)
- c557854 ecmascript: Support MayHaveSideEffects for Statements (#13402) (sapphi-red)

### 🐛 Bug Fixes

- 3e902a0 minifier: Keep argument spread side effects against empty functions (#13401) (sapphi-red)


## [0.83.0] - 2025-08-29

### 💥 BREAKING CHANGES

- e459866 ecmascript: [**BREAKING**] Remove `PropertyReadSideEffects::OnlyMemberPropertyAccess` (#13348) (sapphi-red)

### 🚀 Features

- a40140a minifier: Use `is_literal_value` in `substitute_single_use_symbol_in_expression` (#13364) (sapphi-red)
- ac2067b ecmascript: Examine and improve `is_literal_value` (#13363) (sapphi-red)
- 56d2da3 minifier: Inline top-level variables for module scripts (#13361) (sapphi-red)
- 593f54c minifier: Add `--max-iterations` for debugging (#13291) (sapphi-red)
- a08dd5a minifier: Merge assignments in sequence exprs to variable declaration (#13270) (sapphi-red)
- a19e84f minifier: Merge assignment to variable declaration (#13269) (sapphi-red)
- 07afa70 minifier: Support remaining expressions for single use variable inlining (#13280) (sapphi-red)
- 8e34656 ecmascript: Implement MayHaveSideEffects for AssignmentTarget (#13279) (sapphi-red)
- a951eee minifier: Support more statements for single use variable inlining (#13277) (sapphi-red)
- dcda3a2 minifier: Support more expressions for single use variable inlining (#13276) (sapphi-red)
- 66d6005 minifier: Implement basic single use variable inlining (#13275) (sapphi-red)
- 53f55a4 minifier: Remove unnecessary parenthesis from nested optional chaining (#13268) (sapphi-red)
- 8ca9909 minifier: Remove unused assignments for vars (#13231) (sapphi-red)
- cae222c minifier: Compress `return void foo()` => `foo(); return` (#13271) (sapphi-red)
- a1b6ad4 minifier: Implement known methods `Math.clz32` and `Math.imul` (#12405) (Ethan Wu)

### 🐛 Bug Fixes

- 72468f9 minifier: Keep variables that are modified by `typeof foo === 'object' && foo !== null` compression (2) (#13362) (sapphi-red)
- 5d898e8 ecmascript: Treat shadowed global calls correctly (#13292) (sapphi-red)
- 87e68b5 minifier: Keep variables that are modified by `typeof foo === 'object' && foo !== null` compression (#13290) (sapphi-red)
- 0e804aa minifier: Keep variables that are modified by combined assignments made by minification (#13267) (sapphi-red)
- 6003285 minifier: Keep property access before call expressions as-is to preserve `this` value (#13278) (sapphi-red)

### ⚡ Performance

- d7c9169 minifier: Exit early in `is_closest_function_scope_an_async_generator` (#13273) (sapphi-red)

### 🧪 Testing

- d0f1d63 minifier: Share test helper functions between unit tests and integration tests (#13360) (sapphi-red)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- d27a04b ecmascript: Skip array length evaluation if there are any spread elements (#13162) (Monad)
- f10ac33 codegen: Remove end sourcemaps for `}`, `]`, `)` (#13180) (Boshen)


## [0.82.2] - 2025-08-17

### 🚀 Features

- fbe6663 minifier: Mark more known global methods as side-effect free (#13086) (Boshen)
- 36386e4 ecmascript: Treat `[...arguments]` as side effect free (#13116) (sapphi-red)
- 5dfb40e minifier: Drop `var r = [...arguments]` if `r` is not used (#13115) (sapphi-red)
- 3d0d31a minifier: Rewrite `arguments` copy loops to spread syntax (#13114) (sapphi-red)
- dea41dc minifier: Compress `Object(expr)(args)` to `(0, expr)(args)` (#13092) (sapphi-red)
- fe4589b minifier: Mark more global constructors as side-effect free (#13082) (Boshen)

### 🐛 Bug Fixes

- 896c3ba minifier: Keep `Array` reference id when compressing `Array()` (#13113) (sapphi-red)
- 6686cc4 minifier: Do not remove `using x = ` (#13052) (Boshen)

### 🚜 Refactor

- a36c3ce minfier: Consistent method names (#13060) (Boshen)
- e190ee5 minifier: Clean up `remove_unused_expression` (#13080) (Boshen)

### ⚡ Performance

- 2625bdf minifier: No need to collect references if AST is not changed (#13078) (Boshen)


## [0.82.1] - 2025-08-13

### 🚀 Features

- 993db89 minifier: `.minify` and `.dce` methods; run dce in loop (#13026) (Boshen)

### 🚜 Refactor

- 73a6f25 minifier: Inline statement methods (#13044) (Boshen)
- 53c51f9 minifier: Remove clippy allow directives and fix needless_pass_by_ref_mut warnings in oxc_minifier (#13030) (Copilot)
- 3a8a3ce minifier: Remove clippy::unused_self allow directive by converting methods to associated functions (#13029) (Copilot)
- 7223686 minifier: Use the original vec in-place in `minimize_statements` (#13028) (Boshen)


## [0.82.0] - 2025-08-12

### 🚀 Features

- 54d1750 ecmascript: Handle `typeof` guarded global access as side effect free (#12981) (Copilot)
- 7b9a781 minifier: Return total number of iterations for debugging (#12995) (Boshen)
- 33c0e9f ecmascript: Add global `isNaN`, `isFinite`, `parseFloat`, `parseInt` functions support to constant evaluation (#12954) (Copilot)
- 577d742 minifier: Logical assignment with sequential (#12957) (sapphi-red)
- 96a1a4f minifier: Remove optional chaining calls when the function is `undefined` or `null` (#12956) (sapphi-red)
- 208e6f7 ecmascript: Add URI encoding/decoding support to constant evaluation (#12934) (Copilot)
- 53f7a9f minifier: `new Date()` has `ValueType::Object` (#12951) (Boshen)
- e0e835b minifier: `f(!a === !1)` -> `f(!!a)` (#12942) (Boshen)
- 6e393ec minifier: Evaluate String constructor (#12943) (Boshen)
- 784796d minifier: Fold `(!0).toString()` to `true` (#12938) (Boshen)
- 2c303f5 minifier: Fold `({ ...!0 })` into `({})` (#12935) (Boshen)

### 🐛 Bug Fixes

- 64326c1 minifier: Improve quoted property key handling for class methods and properties (#12999) (Copilot)
- ad4aeaf minifier: Evaluate `e ? consequent : alternate` (#12940) (Boshen)

### 🚜 Refactor

- ca91a26 minfier: Single match expression (#13000) (Boshen)
- 70f742a minifier: Change AST in-place instead of returning `Option<Expression>` (#12993) (Boshen)
- d3ba782 minifier: Fix double mut in DerefMut for Ctx by correcting Target type (#12994) (Copilot)
- 451bc07 minifier: Change AST in-place instead of returning `Option<Expression>` (#12969) (Boshen)
- e5e2496 minifier: Clean up `try_compress_typeof_undefined` (#12958) (Boshen)
- 8a5c9b9 minifier,ecmascript: Clean up `is_global_reference` (#12953) (Boshen)
- 0c5bffc ecmascript: Change `IsGlobalReference` to `GlobalContext` (#12952) (Boshen)

### 📚 Documentation

- 3ce27e9 ecmascript, minifier: Revert changes to changelogs (#12962) (overlookmotel)

### ⚡ Performance

- 96b4009 minifier: Remove late optimization pass (#12670) (Boshen)

### 🧪 Testing

- 2c06eda minifier: Add comprehensive test coverage for unary plus removal (#12860) (Copilot)


## [0.81.0] - 2025-08-06

### 🐛 Bug Fixes

- 44b37f7 minifier: Keep classes with static properties + side effect initializer (#12848) (Boshen)
- 00fda91 minifier: Fix `KATAKANA MIDDLE DOT` syntax error for unicode 4.1 to 15 (#12829) (Boshen)


## [0.80.0] - 2025-08-03

### 🐛 Bug Fixes

- 7dae2e4 minifier: Keep class if class has decorators (#12669) (Boshen)

### 🚜 Refactor

- 5f50bc3 minifier: Move string method constant evaluation from minifier to ecmascript crate (#12672) (Copilot)

### 📚 Documentation

- 514322c rust: Add minimal documentation to example files in crates directory (#12731) (Copilot)

### 🎨 Styling

- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)

### 🧪 Testing

- 16312d7 minifier: Add more tests (#12722) (Copilot)


## [0.79.1] - 2025-07-31

### 🚀 Features

- a286dd4 minifier: Remove unnecessary 'use strict' directive (#12642) (Boshen)
- 763a618 minifier: Inline small constant values (#12639) (Boshen)
- f46818a minifier: Remove unused class expression (#12618) (Boshen)

### 🐛 Bug Fixes

- 08a7379 minifier: Do not read constant value from for loop init (#12654) (Boshen)
- 5642b29 minifier: Initialize constant value in DCE (#12610) (Boshen)


## [0.79.0] - 2025-07-30

### 🚀 Features

- 23f7f82 minifier: Remove unused assignment expression (#12509) (Boshen)
- b877039 minifier: Inline `const` variables that are only used once (#12488) (Boshen)

### 🐛 Bug Fixes

- fe9c8e1 minifier: Do not remove non-plain empty functions (#12573) (Boshen)


## [0.78.0] - 2025-07-24

### 🚀 Features

- c135beb codegen: Keep function expression PIFEs (#12470) (sapphi-red)
- f6e2f29 minifier: Remove unused class declaration (#12419) (Boshen)

### 🚜 Refactor

- 6838948 minifier: Remove change detection based on function changes (#12429) (Boshen)
- 1cf08c0 minifier: Make DCE remove more code to align with rollup (#12427) (Boshen)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)
- 998c67b minifier: Remove no-op function call (#12373) (Boshen)

### ⚡ Performance

- 8bae417 codegen: Remove the useless tokens generated by some expressions (#12394) (Boshen)


## [0.77.2] - 2025-07-17

### 🐛 Bug Fixes

- ed696b5 minifier: Remove more unused assignment expressions (#12364) (Boshen)
- 8777839 minifier: Improve remove unused variable declaration (#12351) (Boshen)

### 🚜 Refactor

- cc3bea4 minifier: Do not remove unused assignment expression yet (#12367) (Boshen)
- eb12132 minifier: Unify access `CompressOptions` through `ctx.state` (#12346) (Boshen)
- 7aea02c minifier: Move `State` to a separate file (#12322) (Boshen)


## [0.77.1] - 2025-07-16

### 🚀 Features

- 1b80633 minifier: Remove unused function declaration (#12318) (Boshen)
- 3f33e8c minifier: Remove unused assignment expression (#12314) (Boshen)
- fb8289c minifier: Remove unused variable declaration (#11796) (Boshen)
- 2cdf722 minifier: Constant fold small integer multiplication (n <= 255) (#12236) (Boshen)
- 314f970 minifier: Remove unused `-1n` (#12235) (Boshen)

### 🚜 Refactor

- 30e8690 minifier: Move tests around (#12237) (Boshen)



## [0.76.0] - 2025-07-08

### 💥 BREAKING CHANGES

- 8b30a5b codegen: [**BREAKING**] Introduce `CommentOptions` (#12114) (Boshen)

### 🚀 Features

- 35c6d48 minifier: Implement `Number` known methods (#12078) (Ethan Wu)


## [0.75.1] - 2025-07-03

### 🚀 Features

- b44386e minifier: Constant evaluate `.toLowerCase().startsWith('prod');` (#12027) (Boshen)

### 🐛 Bug Fixes

- 50fd11e minifier: Always escape `$` when concatenating template literals (#12029) (Boshen)


## [0.75.0] - 2025-06-25

### 🚀 Features

- 1b3f909 minifier: Apply `TreeShakeOptions::annotations` (#11856) (Boshen)


## [0.74.0] - 2025-06-23

### 💥 BREAKING CHANGES

- e81be6e semantic: [**BREAKING**] Rename `symbol_is_used` to `symbol_is_unused` (#11802) (Boshen)
- 7a05e71 minifier: [**BREAKING**] Add `Treeshake` options (#11786) (Boshen)
- 8ef1be2 traverse: [**BREAKING**] Introduce `TraverseCtx<'a, State>` (#11770) (Boshen)

### 🚀 Features

- d462ead minifier: Remove dead code that evaluates to a constant value (#11788) (Boshen)

### 🚜 Refactor

- 5a46641 ecmascript: Move `get_constant_value_for_reference_id` to `IsGlobalReference` trait (#11810) (Boshen)
- d5a8f18 minifier: Make `Ctx` take `&mut TraverseCtx` (#11771) (Boshen)




## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- 40ac186 minifier: Annotate more pure constructors (#11555) (Boshen)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Bug Fixes

- 0d27e58 minifier: Fix incorrect span when performing collapse variable declarations (#11492) (Boshen)

## [0.72.1] - 2025-05-28

### Features

- f88f666 minifier: Normalize `Number.NaN` as `NaN` (#11275) (Boshen)
- d6fc750 minifier: Add `pure` to side-effect free global constructor during DCE (#11270) (Boshen)

## [0.72.0] - 2025-05-24

### Features

- 03390ad allocator: `TakeIn` trait with `AllocatorAccessor` (#11201) (Boshen)

### Refactor

- cef5452 allocator: `TakeIn::take_in_box` accept any `AllocatorAccessor` (#11216) (overlookmotel)

## [0.71.0] - 2025-05-20

- 65a6419 mangler: [**BREAKING**] `Mangler::build_with_semantic` take mut ref to `Semantic` (#11132) (overlookmotel)

### Performance


## [0.70.0] - 2025-05-15

### Refactor

- 751876b parser: Rewrite parse class element (#11035) (Boshen)

## [0.68.1] - 2025-05-04

### Features

- d04ab1f minifier: Inline object literal including __proto__ with spread operator (#10795) (sapphi-red)
- 41c928f minifier: Remove function expressions passed to object spreads (#10794) (sapphi-red)
- 54840d5 minifier: Inline nested spread object literals (#10792) (sapphi-red)

### Bug Fixes

- 7a4babe minifier: Keep string spread in object literals as-is (#10793) (sapphi-red)

## [0.68.0] - 2025-05-03

- a0a37e0 ast: [**BREAKING**] `AstBuilder` methods require an `Atom` with correct lifetime (#10735) (overlookmotel)

- 315143a codegen: [**BREAKING**] Remove useless `CodeGenerator` type alias (#10702) (Boshen)

### Features

- b01cb45 codegen: A way to keep legal comments after minification (#10689) (Boshen)

### Performance

- c279f16 minifier: Avoid temp `String`s and copying string data (#10733) (overlookmotel)

### Refactor


## [0.64.0] - 2025-04-17

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

### Bug Fixes

- 48c711a minifier: Panic when compressing `a ? b() : b()` (#10311) (翠 / green)

### Performance

- 8db9dc5 minifier/minimize_statements: Reduce allocations of `Vec` (#10435) (Dunqing)

### Refactor


## [0.63.0] - 2025-04-08

### Features

- 603abdc minifier: Compress assignment to prefix increment (#10244) (Ulrich Stark)
- aaafdbf minifier: Fold `String::trimStart` / `String::trimEnd` (#10261) (sapphi-red)
- af7a298 minifier: Support advanced `String::indexOf` evaluation (#10186) (sapphi-red)
- bc5411d minifier: Compress `if (foo, !!bar)` to `if (foo, bar)` (#10164) (sapphi-red)
- 395b545 minifier: Compress `foo == true` into `foo == 1` (#10163) (sapphi-red)

### Bug Fixes

- c35d01a minifier: Keep side effect in `String::replace` argument (#10185) (sapphi-red)
- bac8ca0 minifier: Non-constant argument passed to `String::substring` (#10184) (sapphi-red)

### Performance

- 876b6ea minifier: Reduce allocations (#10301) (overlookmotel)
- 0186383 minifier: Avoid cloning `Cow` and allocating `Atom` (#10177) (overlookmotel)

### Refactor

- 9ffb613 minifier: Prefer `is_empty()` over `len() == 0` (#10199) (overlookmotel)
- bcdbd38 transformer, minifier: Replace `AstBuilder::move_xxxx` methods with `TakeIn` trait (#10170) (Dunqing)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

### Testing

- 3a64819 minifier: `String::replace` for objects with `Symbol.replace` (#10260) (sapphi-red)

## [0.62.0] - 2025-04-01

- 4077868 ecmascript: [**BREAKING**] Introduce MayHaveSideEffectsContext (#10126) (sapphi-red)

### Features

- 11ddda6 ecmascript: Add `unknown_global_side_effects` to `MayHaveSideEffectsContext` (#10130) (sapphi-red)
- ef3451c ecmascript: Add `property_read_side_effects` to `MayHaveSideEffectsContext` (#10129) (sapphi-red)
- ac2ecbb ecmascript: Add `is_pure_call` to `MayHaveSideEffectsContext` (#10128) (sapphi-red)
- e004a3f ecmascript: Add `respect_annotations` to `MayHaveSideEffectsContext` (#10127) (sapphi-red)

### Refactor


## [0.61.2] - 2025-03-23

### Features

- ea3de06 mangler: Support `keep_names` option (#9898) (sapphi-red)

### Refactor

- 5ff50e6 minifier: Add `State` for checking changes (#9949) (Boshen)
- fbb268a minifier, transformer: Replace `vec_from_iter` with `vec_from_array` for array (#9906) (Dunqing)

## [0.61.0] - 2025-03-20

- eef7eb6 minifier: [**BREAKING**] Rename `CompressOptions::all_true`/`all_false` to `smallest`/`safest` (#9866) (sapphi-red)

### Features

- dcd356e minifier: Support `keep_names` option (#9867) (sapphi-red)

### Performance

- b272893 mangler, minifier: Initialize a Vec with a specific value using `Vec::from_iter_in` combined with `repeat_with` (#9908) (Dunqing)

### Refactor


## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

### Bug Fixes

- f707d1f parser: Set kind of var_declarator correctly for using decl (#9753) (camc314)

### Refactor


## [0.58.0] - 2025-03-13

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 7331656 semantic: [**BREAKING**] Rename `SymbolTable` and `ScopeTree` methods (#9613) (Boshen)

- 23738bf semantic: [**BREAKING**] Introduce `Scoping` (#9611) (Boshen)

### Features

- b6deff8 ecmascript: Support integer index access for array and string in `may_have_side_effects` (#9603) (sapphi-red)
- 047fb01 minifier: Place `void 0` on right hand side if possible (#9606) (sapphi-red)
- 36f8703 minifier: Compress `[] + string` to `string` (#9602) (sapphi-red)
- 554c4ce minifier: Compress constant integer index access (#9604) (sapphi-red)
- e3c2015 minifier: Allow compressing computed __proto__ more precisely (#9595) (sapphi-red)
- 6a57198 minifier: Allow compressing computed constructor/prototype keys precisely (#9594) (sapphi-red)

### Bug Fixes

- 96eef8b ecmascript: `(foo() + "").length` may have a side effect (#9605) (sapphi-red)
- 24d9261 minifier: Remove names from functions / classes in normal pass to make the minifier idempotent (#9608) (sapphi-red)

### Refactor


## [0.56.2] - 2025-03-07

### Refactor

- 2cbfacb minifier: Remove `clippy::needless_pass_by_ref_mut` (Boshen)

## [0.56.1] - 2025-03-07

### Testing

- 6fd11ce minifier: Test var decl in catch clause edge case (#9577) (翠 / green)

## [0.56.0] - 2025-03-06

### Features

- a92b863 minifier: Keep indirect access more precisely (#9562) (sapphi-red)

### Bug Fixes

- 7a220a3 minifier: Keep indirect access for `delete` and `typeof` (#9563) (sapphi-red)

## [0.55.0] - 2025-03-05

### Features

- 9321439 minifier: Merge throw statements at the end (#9539) (sapphi-red)
- 803f061 minifier: Apply `__NO_SIDE_EFFECTS__` (#9533) (Boshen)

### Testing

- dc1465e minifier: Enable some tests in minimize_conditions (#9543) (sapphi-red)
- 0a5c73b minifier: Enable some tests in peephole directory (#9542) (sapphi-red)
- 55e7ee9 minifier: Enable some esbuild tests (#9540) (sapphi-red)

## [0.54.0] - 2025-03-04

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 64f4a82 ecmascript: Handle pure call expression in chain expressions (#9480) (sapphi-red)
- 32139d2 ecmascript: Support `/* @__PURE__ */` in may_have_side_effects (#9409) (sapphi-red)
- f5453f6 minifier: Flatten spread args in new expressions (#9512) (sapphi-red)
- f8073f3 minifier: Support if with side effects in dead code elimination (#9502) (sapphi-red)
- 10eb8f7 minifier: Use `remove_unused_expression` in `try_fold_sequence_expression` in `remove_dead_code` (#9467) (sapphi-red)
- 70916db minifier: Remove unused expression in for init and update (#9466) (sapphi-red)
- 26fde56 minifier: Inline simple IIFEs in `remove_unused_expression` (#9465) (sapphi-red)
- ec2193e minifier: Support extracting arguments in pure calls in `remove_unused_expression` (#9463) (sapphi-red)
- 007051c minifier: Compress `a != null && a.b()` to `a?.b()` in `remove_unused_expression` (#9459) (sapphi-red)
- 50fce20 minifier: Support binary expression in `remove_unused_expression` (#9456) (sapphi-red)
- ed9ede3 minifier: Support conditional expression in `remove_unused_expression` (#9432) (sapphi-red)
- 3520538 minifier: Support object expression in `remove_unused_expression` (#9430) (sapphi-red)
- fb8a93d minifier: Improve array expression handling in `remove_unused_expression` (#9427) (sapphi-red)
- ff477cd minifier: Support template literals in `remove_unused_expression` (#9426) (sapphi-red)
- 0d26113 minifier: Compress `/* @__PURE__ */ a() ? b : b` to `b` (#9410) (sapphi-red)
- 7d7f16c parser: Apply pure to rhs of binary expression (#9492) (Boshen)

### Bug Fixes

- f5bbd5d ecmascript: Fix toString for negative numbers (#9508) (sapphi-red)
- d2cd975 ecmascript: Fix may_have_side_effects for `${a() === b}` (#9478) (sapphi-red)
- 584d847 ecmascript: Objects passed to template literals may have side effects (#9425) (sapphi-red)
- 1fff993 minifier: Correctly remove dead code in `try` with `finally` (#9503) (sapphi-red)
- 0b96ebe minifier: Don't inline IIFE with parameters (#9477) (sapphi-red)
- 071c84c minifier: Skip `try_fold_stmt_in_boolean_context` on `ExpressionStatement` (#9458) (sapphi-red)
- 306284d minifier: Call mark_current_function_as_changed in remove_unused_expression (#9457) (sapphi-red)

### Refactor

- bbb450c minifier: Move `a != null && b` -> `a ?? b` compression to `remove_unused_expression` (#9468) (sapphi-red)
- b93774c minifier: Move `try_fold_iife` to `remove_unused_expression` (#9464) (sapphi-red)
- 96a9719 minifier: Use `may_have_side_effects` in `remove_unused_expression` (#9413) (sapphi-red)

### Testing

- c187b11 ecmascript: Add comments and tests for cases where `ToPropertyKey` throws an error (#9429) (sapphi-red)

## [0.53.0] - 2025-02-26

### Features

- e10fb97 ecmascript: Improve may_have_side_effects for `.length` (#9366) (sapphi-red)
- 35e5ca9 ecmascript: Improve may_have_side_effects for `instanceof` (#9365) (sapphi-red)
- 11012c6 ecmascript: Improve ValueType for coalesce operator (#9354) (sapphi-red)
- b7998fd ecmascript: To_number for object without toString (#9353) (sapphi-red)
- e51d563 minifier: Concatenate strings with template literals on right side (#9356) (sapphi-red)
- 9d7db54 minifier: Concatenate strings with template literals (#9355) (sapphi-red)

### Bug Fixes

- f5c8698 ecmascript: Correct may_have_side_effects for classes (#9367) (sapphi-red)
- d3ed128 minifier: Do not remove `=== 0` if the lhs can be NaN (#9352) (sapphi-red)

### Refactor

- faf966f ecmascript: Don't check side effects in constant_evaluation (#9122) (sapphi-red)

## [0.52.0] - 2025-02-21

### Features

- dde05e3 mangler: Opt-out of direct eval (#9191) (Boshen)
- 857f901 minifier: Inline constant values in template literals (#9201) (sapphi-red)

### Bug Fixes

- d2ab0fe minifier: Fix `clippy::suspicious_operation_groupings` warning (#9238) (Boshen)

### Refactor

- 3b1497b minifier: Improve minimize_if_statement (#9177) (Boshen)
- 814eab6 minifier: `Math.pow(a,b)` -> `a ** (+b)` instead of `(+a) ** (+b)` (#9154) (Boshen)
- ef856f5 oxc: Apply `clippy::needless_pass_by_ref_mut` (#9253) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.51.0] - 2025-02-15

### Features

- 36c8640 ecmascript: Support more string concatenation (#9121) (sapphi-red)
- 6936b08 ecmascript: Fold `typeof` with ValueType information (#9086) (sapphi-red)
- b25b84b minifier: Substitute redundant assignment target bindings (#9096) (camchenry)
- 125d610 minifier: Fold String::charAt / String::charCodeAt more precisely (#9082) (sapphi-red)
- 237ffba minifier: Fold bitwise binary expressions with negative BigInts (#9081) (sapphi-red)
- 24830e6 minifier: Fold `a + 'a' + 1` to `a + 'a1'` (#9080) (sapphi-red)
- b5eb6e5 minifier: Improve `remove_unused_expression` (#9071) (Boshen)

### Bug Fixes

- eb7cd62 ecmascript: To_number for shadowed undefined (#9106) (sapphi-red)
- 8cbdf00 ecmascript: To_boolean for shadowed undefined (#9105) (sapphi-red)
- 17c745c ecmascript: To_string for object with toString (#9104) (sapphi-red)
- cfc71f9 ecmascript: To_string for shadowed undefined (#9103) (sapphi-red)
- 2ab2a8f ecmascript: Handle shadowed global variables in `ValueType` (#9085) (sapphi-red)
- 2fd1589 minifier: Compress computed string literals in method/property definitions (#9126) (camchenry)

### Refactor

- 8bd6eef ecmascript: Merge constant evaluation logics (#9120) (sapphi-red)
- b164072 ecmascript: Extract to_numeric (#9111) (sapphi-red)
- 8f79012 ecmascript: Pass IsGlobalReference to DetermineValueType instead of extending it (#9107) (sapphi-red)
- db1744c ecmascript: Remove "constant_evaluation" / "side_effects" features (#9114) (sapphi-red)
- d670ec7 ecmascript: Pass IsGlobalReference to MayHaveSideEffects instead of extending it (#9101) (sapphi-red)
- f4e2d4e ecmascript: Allow IsGlobalReference to return None (#9100) (sapphi-red)
- 29be94d minifier: Inline more minification methods (#9088) (Boshen)
- 80f719e minifier: Clean up minimize_statements.rs (#9076) (Boshen)
- d5edde0 minifier: Minimize `if (!foo) foo = bar;` -> `foo ||= bar` in the same AST pass (#9075) (Boshen)

## [0.50.0] - 2025-02-12

### Features

- 4d2b0d5 minifier: Port esbuild `SimplifyUnusedExpr` (#9036) (Boshen)
- df6941d minifier: Fold unary not (#9031) (Boshen)

## [0.49.0] - 2025-02-10

- bbb075d ast: [**BREAKING**] Name `AstBuilder` enum builders after variant name not type name (#8890) (overlookmotel)

- b7ff7e1 span: [**BREAKING**] Export `ContentEq` trait from root of `oxc_span` crate (#8869) (overlookmotel)

### Features

- ad1a878 ecmascript: Support BigInt comparison (#9014) (sapphi-red)
- d515cfd ecmascript: Detect objects without overridden `toString`/`valueOf`/`Symbol.toPrimitive` (#8993) (sapphi-red)
- 2a10a99 ecmascript: Support arrays and objects for unary expression may_have_side_effects (#8990) (sapphi-red)
- dd383c0 ecmascript: `ValueType::from` for PrivateInExpression (#8964) (sapphi-red)
- c3eef2f ecmascript: `ValueType::from` for parenthesized expressions (#8962) (sapphi-red)
- 2c3a46d ecmascript: Support more simple expressions by `ValueType::from` (#8961) (sapphi-red)
- 4cec83f ecmascript: `ValueType::from` for bitwise not operator (#8955) (sapphi-red)
- e3e9999 ecmascript: Complete may_have_side_effects (#8855) (sapphi-red)
- cd84a05 minifier: Minimize expression statement `!x` -> `x` (#9012) (Boshen)
- b416334 minifier: Remove useless string addition (#9011) (sapphi-red)
- beeb2fb minifier: Implement `optimizeImplicitJump` (#8984) (Boshen)
- ce3b744 minifier: Remove name from function / class expressions (#8997) (sapphi-red)
- 3f7faed minifier: Remove unused function expression with name by remove_dead_code (#8996) (sapphi-red)
- ec601f2 minifier: Improve `mangleFor` (#8901) (Boshen)
- ca4831b minifier: Fold `typeof class {}` to `'function'` (#8949) (sapphi-red)
- 9ffe9e9 minifier: Fold `typeof (() => {})` to `'function'` (#8948) (sapphi-red)
- 36007de minifier: Fold typeof `{ foo }` when `foo` is declared (#8947) (sapphi-red)
- 56575b2 minifier: Fold complicated array literals passed to unary `+` (#8944) (sapphi-red)
- 14462be minifier: Fold simple literals passed to unary `+` (#8943) (sapphi-red)
- 4a86467 minifier: Remove unnecessary unary `+` inside numeric binary operators (#8957) (sapphi-red)
- 4b4d543 minifier: Minimize block statements (#8857) (Boshen)
- d6d13dd minifier: Minimize `!!(boolean_expr)` -> `boolean_expr` (#8849) (Boshen)
- 20f2c46 minifier: `for (;;) { var x }` -> `for (;;) var x;` (#8847) (Boshen)
- e623745 minifier: Minify `String::concat` into template literal (#8443) (sapphi-red)
- 84b62c7 minifier: Implement minimize for statement (#8846) (Boshen)

### Bug Fixes

- 9a5a926 ecmascript: Fix may_have_side_effects for binary expressions (#8991) (sapphi-red)
- 660c314 ecmascript: Fix may_have_side_effects for unary expressions (#8989) (sapphi-red)
- 8ab7204 ecmascript: Fix `ValueType::from` for `AssignmentExpression` (#8959) (sapphi-red)
- aeb122d ecmascript: Fix `ValueType::from` for numeric binary operators (#8956) (sapphi-red)
- 1182c20 ecmascript: `ValueType::from` for unknown value should return Undetermined instead of Number (#8954) (sapphi-red)
- b5a7785 minifier: Fix comparison of strings with unicode characters (#8942) (sapphi-red)
- 4a723f1 minifier: Should not merge conditional function calls if referencing the function has a side-effect (#8922) (sapphi-red)

### Refactor

- 9b5d800 minifier: Move equality comparison to ConstantEvaluation (#9009) (sapphi-red)
- 9193217 minifier: Remove duplicated `typeof` comparison to non-strict equality compression (#9010) (sapphi-red)
- 85b8ea4 minifier: Extract `symbols().symbol_is_used(symbol_id)` (#8995) (sapphi-red)
- 9c84c6d minifier: Break up methods into different files (#8843) (Boshen)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- e1fd3e8 ecmascript: Add tests for `ValueType` for undetermined cases (#8960) (sapphi-red)
- cc7bb9c ecmascript: Add test for `ValueType` and update document (#8951) (sapphi-red)
- cebb350 minfier: Clean up some esbuild tests (Boshen)
- 8495c21 minifier: Enable passed tests (Boshen)
- f6d43f5 minifier: Enable esbuild constant evaluation tests (#8941) (sapphi-red)

## [0.48.2] - 2025-02-02

### Features

- 86b6219 mangler: Use characters in the order of their likely frequency (#8771) (sapphi-red)
- d553318 minifier: Complete `MangleIf` (#8810) (Boshen)
- 5cfea76 minifier: Compress `(a = _a) != null ? a : b` and `(a = _a) != null ? a.b() : undefined` (#8823) (sapphi-red)
- f02d9e9 minifier: Merge single var declarations without init into for-of (#8813) (sapphi-red)
- 99b47ed minifier: Merge single var declarations without init into for-in (#8812) (sapphi-red)
- d9f1d0d minifier: Merge expressions in for-in statement head (#8811) (sapphi-red)
- 18f1b15 minifier: Implement known method `Array.of` (#8805) (7086cmd)
- e525e60 minifier: Compress `a != null ? a.b : undefined` to `a?.b` (#8802) (#8808) (Boshen)
- e353a01 minifier: Compress `a != null ? a.b : undefined` to `a?.b` (#8802) (sapphi-red)
- 72d74a2 minifier: Compress `a != null ? a : b` into `a ?? b` (#8801) (sapphi-red)
- 249895f minifier: Implement variadic Math methods in known methods (#8783) (Ethan Goh)
- 7ea99f4 minifier: Compress array of string literals to `'str1,str2'.split(',')` (#8786) (sapphi-red)
- 6c627df minifier: Implement unary Math functions in known methods (#8781) (7086cmd)
- ad14403 minifier: Compress `typeof a.b === 'undefined'` to `a.b === void 0` (#8751) (sapphi-red)
- f7f2d2f minifier: Compress `a == null && b` to `a ?? b` when return value is ignored (#8749) (sapphi-red)
- 3c1c92c minifier: Support `a[0]` and `this.a` in `has_no_side_effect_for_evaluation_same_target` (#8748) (sapphi-red)
- 29417dd minifier: Minimize `!(a, b)` -> `a, !b` (#8746) (Boshen)
- 3ece991 minifier: Remove unused `import.meta` statement (#8744) (Boshen)
- 3ef980a minifier: Remove unreachable statements after `break` and `continue` (#8743) (Boshen)

### Bug Fixes

- 831928d minifier: Mark as changed when `else if` was converted to `if` (#8837) (翠 / green)
- f8548ec minifier: Unreachable error when compressing string literal arrays with `.split()` (#8806) (sapphi-red)
- 2eac9c0 minifier: Fix `var undefined = 1; foo === null || foo === undefined` should not be compressed (#8803) (sapphi-red)
- ae7f670 minifier: Avoid minifying `+void unknown` to `NaN` and fix typo (#8784) (7086cmd)
- 8781537 minifier: `{ let foo; const bar = undefined; }` -> `{ let foo, bar; }` (#8764) (Boshen)
- 8a6ae8a minifier: Do not change `const` to `let` if assignment to constant variable. (#8761) (Boshen)
- 66c33ed minifier: Remove incorrect not + conditional expression compression (#8759) (翠 / green)
- a3b078a minifier: Fix crash with `[]['concat'](1)` (#8750) (sapphi-red)

### Refactor

- 6aa2dde codegen: Accept SymbolTable instead of Mangler (#8829) (Daniel Bulant)
- 3abf2f7 minifier: Extract `extract_id_or_assign_to_id` method (#8822) (sapphi-red)
- a861d93 minifier: Port esbuild's `mangleStmts` (#8770) (Boshen)
- 0fcff20 minifier: Remove `EmptyStatement` in a single place (#8745) (Boshen)

### Testing

- dc4c388 minifier: Fail tests when parse fails (#8836) (sapphi-red)
- 3ac5020 minifier: Enable more passed esbuild tests (#8804) (Boshen)
- 0c4c739 minifier: Cleanup some tests in substitute_alternate_syntax (#8768) (sapphi-red)
- 79d5481 minifier: Add and enable some tests in fold_constants (#8769) (sapphi-red)
- ef55e7c minifier: Check idempotency for all tests (#8754) (Boshen)
- d072f09 minifier: Enable more ignored tests (#8753) (Boshen)
- e78e468 minifier: Cleanup some tests in minimize_conditions (#8752) (sapphi-red)

## [0.48.1] - 2025-01-26

### Features

- 6589c3b mangler: Reuse variable names (#8562) (翠 / green)
- 29bd215 minifier: Minimize `Infinity.toString(radix)` to `'Infinity'` (#8732) (Boshen)
- e0117db minifier: Replace `const` with `let` for non-exported read-only variables (#8733) (sapphi-red)
- 9e32f55 minifier: Evaluate `Math.sqrt` and `Math.cbrt` (#8731) (sapphi-red)
- 360d49e minifier: Replace `Math.pow` with `**` (#8730) (sapphi-red)
- 2e9a560 minifier: `NaN.toString(radix)` is always `NaN` (#8727) (Boshen)
- cbe0e82 minifier: Minimize `foo(...[])` -> `foo()` (#8726) (Boshen)
- e9fb5fe minifier: Dce pure expressions such as `new Map()` (#8725) (Boshen)

### Bug Fixes

- 33de70a mangler: Handle cases where a var is declared in a block scope (#8706) (翠 / green)
- d982cdb minifier: `Unknown.fromCharCode` should not be treated as `String.fromCharCode` (#8709) (sapphi-red)

### Performance

- e472ced mangler: Optimize handling of collecting lived scope ids (#8724) (Dunqing)
- 8587965 minifier: Normalize `undefined` to `void 0` before everything else (#8699) (Boshen)

### Refactor

- 58002e2 ecmascript: Remove the lifetime annotation on `MayHaveSideEffects` (#8717) (Boshen)
- 6bc906c minifier: Allow mutating arguments in methods called from `try_fold_known_string_methods` (#8729) (sapphi-red)
- bf8be23 minifier: Use `Ctx` (#8716) (Boshen)
- 0af0267 minifier: Side effect detection needs symbols resolution (#8715) (Boshen)
- 32e0e47 minifier: Clean up `Normalize` (#8700) (Boshen)

### Testing

- 03229c5 minifier: Fix broken tests (#8722) (Boshen)

## [0.48.0] - 2025-01-24

### Features

- 343690e minifier: Replace `Number.*_SAFE_INTEGER`/`Number.EPSILON` (#8682) (sapphi-red)
- 0c5bb30 minifier: Replace `Number.POSITIVE_INFINITY`/`Number.NEGATIVE_INFINITY`/`Number.NaN` (#8681) (sapphi-red)
- 835b258 minifier: Compress `typeof foo === 'object' && foo !== null` to `typeof foo == 'object' && !!foo` (#8638) (sapphi-red)
- 2bcbed2 minifier: Compress `(a = b) === null || a === undefined` to `(a = b) == null` (#8637) (sapphi-red)

### Bug Fixes

- 883d25b minifier: Keep esm in dce (#8677) (Boshen)
- 878ce10 minifier: `void 0` equals to `undefined` (#8673) (Boshen)
- ba201a6 minifier: Remove "non esbuild optimizations" which is incorrect (#8668) (Boshen)
- 8c8b5fa minifier: Avoid minifing `String(a)` into `"" + a` for symbols (#8612) (翠 / green)
- 4ff6e85 minifier: Remove expression statement `void 0` (#8602) (Boshen)
- 93d643e minifier: Keep side effects when folding const conditional exprs (#8591) (camc314)

### Performance

- 9953ac7 minifier: Add `LatePeepholeOptimizations` (#8651) (Boshen)
- 00dc63f minifier: Only substitute typed array constructor once (#8649) (Boshen)
- 3e19e4e minifier: Remove the useless empty statement removal code in statement fusion (#8646) (Boshen)
- 5b3c412 minifier: Only run optimizations on local changes (#8644) (Boshen)

### Refactor

- e66da9f isolated_declarations, linter, minifier, prettier, semantic, transformer: Remove unnecessary `ref` / `ref mut` syntax (#8643) (overlookmotel)
- ce2b9da minifier: Remove `wrap_to_avoid_ambiguous_else` (#8676) (Boshen)
- 75a579b minifier: Clean up `has_no_side_effect_for_evaluation_same_target` (#8675) (Boshen)
- 1bb2539 minifier: Move more code into `minimize_conditions` local loop (#8671) (Boshen)
- 13e4a45 minifier: Move conditional assignment to `minimize_conditions` (#8669) (Boshen)
- ae895d8 minifier: Use `NonEmptyStack` for function stack (#8661) (Boshen)
- 3802d28 minifier: Clean up `try_minimize_conditional` (#8660) (Boshen)
- dcc1f2b minifier: Rename `ast_passes` to `peephole` (#8635) (Boshen)
- 52458de minifier: Remove unused code and traits (#8632) (Boshen)
- 6f95cd5 minifier: Remove all the unnecessary fake ast passes (#8618) (Boshen)
- 712cae0 minifier: Run the compressor on all test cases (#8604) (Boshen)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

### Testing

- d9f5e7f minifier: Enable passed esbuild tests (Boshen)

## [0.47.1] - 2025-01-19

### Bug Fixes

- 7b219a9 minifier: Fix dce shadowed undefined (#8582) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- 4d4e805 minifier: Collapse if stmt with empty consequent (#8577) (camc314)
- 991a22f minifier: Fold `Array::concat` into literal (#8442) (sapphi-red)
- 3dc2d8b minifier: Fold string concat chaining (#8441) (sapphi-red)
- a4ae450 minifier: Fold array concat chaining (#8440) (sapphi-red)
- 7cc81ef minifier: Fold invalid typeof comparisons (#8550) (camc314)
- 927f43f minifier: Improve `.charCodeAt(arg)` when arg is valid (#8534) (Boshen)
- 06f14d5 minifier: Remove empty class static block `class Foo { static {} }` (#8525) (Boshen)
- 1860411 minifier: Remove last redundant return statement (#8523) (Boshen)

### Bug Fixes

- 65c596d minifer: Keep idents if not in scope when minimizing array exprs (#8551) (camc314)
- f57aac2 minifier: Incorrect folding of expr in bool ctx (#8542) (camc314)
- 946ad76 minifier: `(-Infinity).toString()` -> `'-Infinity'` (#8535) (Boshen)
- b1d0186 minifier: Do not fold `!!void b` (#8533) (Boshen)
- 53adde5 minifier: `x['-2147483648']` -> `x[-2147483648]` (#8528) (Boshen)
- 405b73d minifier: Do not change `delete undefined` to `delete void 0` (#8527) (Boshen)
- 92e44cb minifier: Do not remove `undefined` in `var x = undefined` (#8526) (Boshen)
- 209e313 minifier: `class C { ['-1']() {} }` cannot be minifized (#8516) (Boshen)
- 6585463 minifier: Always keep the last value of sequence expression (#8490) (Boshen)

### Refactor

- 8f57929 minifier: Merge `try_compress_type_of_equal_string` into `try_minimize_binary` (#8561) (sapphi-red)

### Testing

- e0f5d6c minifier: Update esbuild test (Boshen)
- 629c417 minifier: Port esbuild minification tests (#8497) (Boshen)

## [0.46.0] - 2025-01-14

### Features

- 8accfef minifier: Minify `var x; void x` -> `void 0` (#8466) (Boshen)
- 870a583 minifier: Fold `false['toString']` (#8447) (Boshen)
- 4ad695d napi/minify: Implement napi (#8478) (Boshen)

### Bug Fixes

- 4c6675c minifier: Do not convert while to fors in DCE (#8484) (Boshen)
- 1d6e84d minifier: Fix incorrect `null.toString()` and `1n.toString()` (#8464) (Boshen)
- 25d4bf9 minifier: Remove usage of empty spans (#8462) (Boshen)
- dd64340 minifier: Keep `return undefined` in async generator function (#8439) (Boshen)

### Performance

- 8fc238a minifier: Merge `Normalize` and `RemoveSyntax` pass (#8467) (Boshen)
- 372eb09 minifier: Preallocate mangler's semantic data (#8451) (Boshen)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 41ddf60 minfier: Add `CompressOptions::target` (#8179) (Boshen)
- d56020b minifier: Drop `0` from `new Int8Array(0)` and other TypedArrays (#8431) (sapphi-red)
- f935d94 minifier: Remove `new` from NativeErrors / `AggregateError` (#8430) (sapphi-red)
- dab7a51 minifier: Minimize not `!(x === undefined)` -> `x !== undefined` (#8429) (Boshen)
- 0e7bab8 minifier: Remove `if(false){}` in a single pass (#8421) (Boshen)
- 5b5b844 minifier: Fold `ambiguous if else` (#8415) (Boshen)
- 438a6e7 minifier: Minimize conditions in boolean context (#8381) (Boshen)
- 793cb43 minifier: `a != null ? a : b` -> `a ?? b` (#8352) (camc314)
- 814da55 minifier: Compress `x = x || 1` to `x ||= 1` (#8368) (sapphi-red)
- a596821 minifier: Compress `a.b = a.b + c` to `a.b += c` (#8367) (sapphi-red)
- 579eb60 minifier: Compress `a.b || (a.b = c)` to `a.b ||= c` (#8366) (sapphi-red)
- f367a16 minifier: Port esbuild conditional expr minification (#8351) (camc314)
- 8d52cd0 minifier: Merge assign expression in conditional expression (#8345) (sapphi-red)
- a69d15f minifier: Compress `new Array(2)` -> `[,,]` (#8344) (sapphi-red)
- 819c475 minifier: Compress `new Array(7n)` -> `[7n]` (#8343) (sapphi-red)
- e085d66 minifier: Remove empty IIFE (#8340) (Boshen)
- 2c2e483 minifier: Fold object spread `({ ...null })` -> `({})` (#8339) (Boshen)
- 6220e05 minifier: Remove empty if statment `if (test) {}` -> `test` (#8336) (Boshen)
- a76dfae minifier: Remove label statement with empty body (#8333) (Boshen)
- e88a6bd minifier: Minimize `!0 + null !== 1` -> `!0 + null != 1` (#8332) (Boshen)
- ec88c68 minifier: Compress `a || (a = b)` to `a ||= b` (#8315) (sapphi-red)
- e6fe84d minifier: Compress `a = a + b` to `a += b` (#8314) (sapphi-red)
- 9ea4e31 minifier: Remove `new` from `new Error`/`new Function`/`new RegExp` (#8313) (sapphi-red)
- 051fbb6 minifier: Minimize `x["0"]` -> x[0] (#8316) (Boshen)
- a542013 minifier: Minimize `do{}while(true)` -> `do;while(true)` (#8311) (Boshen)
- e3ff81e minifier: Minimize `(x = 1) === 1` -> `(x = 1) == 1` (#8310) (Boshen)
- 4b68cc0 minifier: Minimize empty `try` statement (#8309) (Boshen)
- 922c514 minifier: Fold `.toString()` (#8308) (Boshen)
- 66a2443 minifier: Minify sequence expressions (#8305) (camc314)
- af65c36 minifier: Minimize double negated binary expressions (#8304) (camc314)
- 76c778b minifier: Remove logical nots when arg is a delete expression (#8303) (camc314)
- 5ed439b minifier: Minify typeof in binary expressions (#8302) (camc314)
- 6afc590 minifier: Compress typeof addition string (#8301) (camc314)
- ecc789f minifier: Fold `if(x >> y == 0){}` -> `if(!(x >> y)){}` (#8277) (Boshen)
- 0e3b79a minifier: Fold `String()` -> `''`, `Number()` -> `false` (#8274) (Boshen)
- c9cf593 minifier: Compress  property key `{[1]: _}`  -> {1: _} (#8272) (Boshen)
- b92b2ab minifier: Fold `BigInt(1n)` -> `1n` (#8270) (Boshen)
- a4df387 minifier: Compress loose equals undefined (#8268) (camc314)
- f000596 minifier: Minify call expressionsto `Number` (#8267) (camc314)
- 092aeaf minifier: Flatten spread args in call expressions (#8266) (camc314)
- 04ec38d minifier: Remove unused arrow function expressions (#8262) (camc314)
- e446c15 minifier: Improve minimizing unary not expressions (#8261) (camc314)
- 7f19211 minifier: Minimize unary expression statements (#8256) (camc314)
- cec63e2 minifier: `{}` evals to `f64::NaN` (Boshen)
- 4d8a08d minifier: Improve constant evaluation (#8252) (Boshen)
- e84f267 minifier: Compress more property keys (#8253) (Boshen)
- d1224f9 minifier: Improve minimizing conditional expressions (#8251) (camc314)
- 65f46f5 minifier: Constant fold `String.fromCharCode` (#8248) (Boshen)
- bd8d677 minifier: Minimize `~undefined`, `~null`, `~true`, `~false` (#8247) (Boshen)
- f73dc9e minifier: Constant fold `'x'.toString()` and `true.toString()` (#8246) (Boshen)
- fd5af73 minifier: Minimize `Number` constructor (#8245) (Boshen)
- 2f52f33 minifier: Minsize `!!!foo ? bar : baz` -> `foo ? baz : bar` (#8244) (Boshen)
- ccdc039 minifier: Always put literals on the rhs of equal op `1==x` => `x==1` (#8240) (Boshen)
- 39353b2 minifier: Improve minimizing conditionals (#8238) (Cameron)
- c90fc16 minifier: Restore conditional minification and fix edge case (#8235) (camc314)
- 6c8ee9f minifier: Remove last redundant `return` statement (#8234) (Boshen)
- 51f4792 minifier: Minimize `foo ? foo : bar` and `foo ? bar : foo` (#8229) (Boshen)
- 6e2ec17 minifier: Statement fusion switch cases; improved minimize exit poitns (#8228) (Boshen)
- 574a242 minifier: Minimize all variants of `typeof x == 'undefined'` (#8227) (Boshen)
- 2041477 minifier: Fold `if(x)return;y` -> `if(!x)y` (#8226) (Boshen)
- 9c1afa4 minifier: Optional catch binding when catch param is unused (#8221) (Boshen)
- 4a29845 minifier: Add `ConvertToDottedProperties` (#8212) (Boshen)
- 2786dea minifier: Add `RemoveUnusedCode` (#8210) (Boshen)
- cd274ee minifier: Minimize logical exprs (#8209) (Cameron)
- 4ae15df minifier: Imprve more conditional expr minification with boolean lit (#8208) (camc314)
- 3202b4f minifier: Imprve conditional expr minification with boolean lit (#8207) (camc314)
- 3b45011 minifier: Handle conditional expr with boolean lit (#8206) (camc314)
- 4c2059a minifier: Reverse negated conditional exprs (#8205) (camc314)
- 4804933 minifier: Add `MinimizeExitPoints` and ExploitAssigns` boilerplate (#8203) (Boshen)
- bf266e1 minifier: Try collapse conditional to logical or expr (#8197) (Cameron)
- 06e1780 minifier: Improve `StatementFusion` (#8194) (Boshen)
- 42e211a minifier: Only constant fold numbers when result is smaller (#8092) (Boshen)
- d0de560 minifier: Change `NaN` to `f64::NAN` (#8191) (Boshen)
- cef8eb8 minifier: Change `foo?.['bar']` to `foo?.bar` (#8176) (翠 / green)
- 8149e34 minifier: Optional catch binding when es target >= es2019 (#8180) (Boshen)
- fc43ec5 minifier: Fold `string.length` / `array.length` (#8172) (sapphi-red)
- 29dc0dc minifier: Change `foo['bar']` -> foo.bar (#8169) (Boshen)
- 3c5718d minifier: Fold `typeof foo == undefined` into `foo == undefined` when possible (#8160) (翠 / green)
- f3a36e1 minifier: Fold `typeof foo != "undefined"` into `typeof foo < "u"` (#8159) (翠 / green)
- 37c9959 minifier: Normalize `Infinity` into `f64::Infinity` (#8148) (Boshen)
- 8fb71f5 minifier: Minify string `PropertyKey` (#8147) (Boshen)
- 6615e1e minifier: Constant fold `instanceof` (#8142) (翠 / green)
- 2b2a373 minifier: Minimize `a + 'b' + 'c'` -> `a + 'bc'` (#8137) (Boshen)
- 213364a minifier: Minimize `if (x) if (y) z` -> `if (x && y) z` (#8136) (Boshen)
- 6b51e6d minifier: Minimize `if(foo) bar else baz` -> `foo ? bar : baz` (#8133) (Boshen)
- f615bfa minifier: Minimize `if (x) return; return 1` -> `return x ? void 0 : 1` (#8130) (Boshen)
- f0b1ee5 minifier: Minimize `if(!x) foo()` -> `x || foo()` (#8122) (Boshen)
- f8200a8 minifier: Minimize `if(foo) bar` -> `foo && bar` (#8121) (Boshen)
- 72d9967 minifier: Add `Normalize` ast pass (#8120) (Boshen)
- fef0b25 minifier: Collapse `var` into for loop initializer (#8119) (Boshen)
- 2331ea8 minifier: `typeof foo === 'number'` => `typeof foo == 'number'` (#8112) (Boshen)
- ad9a0a9 mininifier: Minimize variants of `a instanceof b == true` (#8241) (Boshen)

### Bug Fixes

- 74572de ecmascript: Incorrect `to_int_32` value for Infinity (#8144) (翠 / green)
- 5c63414 mangler: Keep exported symbols for `top_level: true` (#7927) (翠 / green)
- 3c93549 minifier: Dce if statement should keep side effects and vars (#8433) (Boshen)
- 52f88c0 minifier: Rotate associative operators to make it more idempotent (#8424) (camc314)
- a80460c minifier: Correctly set `self.changed` when minimizing if stmts (#8420) (camc314)
- d4ca8d4 minifier: `!!x` is not idempotent in `RemoveDeadCode` (#8419) (Boshen)
- 357b61d minifier: Do not minify `Object.defineProperty` in sequence expressions (#8416) (Boshen)
- 0efc845 minifier: `+0n` produces `TypeError` (#8410) (Boshen)
- 7ce6a7c minifier: `a in b` has error throwing side effect (#8406) (Boshen)
- 2f3a9dc minifier: Cannot transform property key `#constructor` (#8405) (Boshen)
- c0a3dda minifier: `instanceof` has error throwing side effect (#8378) (Boshen)
- 5516f7f minifier: Do not fold object comparisons (#8375) (Boshen)
- cb098c7 minifier: Computed property key `prototype` cannot be changed (#8373) (Boshen)
- 82ee77e minifier: Do not remove shadowned `undefined` in return statement (#8371) (Boshen)
- f87da16 minifier: Do not fold literals in `-0 != +0` (#8278) (Boshen)
- 62a2644 minifier: Handle arrow fn expressions correctly in `is_in_boolean_context` (#8260) (camc314)
- d2f8eaa minifier: Fix panic in `peephole_minimize_conditions` (#8242) (Boshen)
- a698def minifier: Fix incorrect return value for `(x ? true : y)` (#8233) (Boshen)
- 56b7f13 minifier: Do not constant fold `0 instanceof F` (#8199) (Boshen)
- 75d5f17 minifier: Minify string `PropertyKey` (#8177) (sapphi-red)

### Documentation

- aaa009d minifier: Clarify assumptions for compressor (#8404) (翠 / green)

### Refactor

- fb2acd8 minifier: Change minimize conditionals into a loop (#8413) (Boshen)
- baaec60 minifier: Remove the buggy `??` transform (#8411) (Boshen)
- 1c4658d minifier: Change ast passes order, `!in_fixed_loop` happen last (#8380) (Boshen)
- 09f0f48 minifier: Remove the buggy `minimize_exit_points` implementation (#8349) (Boshen)
- 9a5c66a minifier: Clean up (#8346) (Boshen)
- 98f2b1c minifier: Clean up `peephole_substitute_alternate_syntax` (#8327) (Boshen)
- fc662b7 minifier: Handle big int values later (#8324) (Boshen)
- d16e598 minifier: Clean up `peephole_replace_known_methods` (#8306) (Boshen)
- b8d26ea minifier: Move optional catch param to peephole_substitute_alternate_syntax (#8282) (Boshen)
- 0845162 minifier: Clean up `ReplaceKnownMethods` (Boshen)
- 7c7f5d7 minifier: Clean up `peephole_fold_constants` (Boshen)
- bf0fbce minifier: Improve constant fold numbers (#8239) (Boshen)
- 62f8fba minifier: Move all conditional minification logic to minimze_conditions (#8231) (camc314)
- cfb51f2 minifier: Fuse ast passes (#8184) (Boshen)
- bf9cafe minifier: Clean up `peephole_substitute_alternate_syntax` a little bit (Boshen)
- 75264ed minifier: Clean up `try_optimize_block` (#8139) (Boshen)
- c22062b minifier: Cleanup peephole_minimize_conditions (#8114) (Boshen)
- e594c39 minifier: Clean up `peephole_substitute_alternate_syntax.rs` (#8111) (Boshen)

### Testing

- 3149fe0 minifier: Add anonymous function test case for logical expression to logical assignment compression (#8347) (sapphi-red)
- 91b42de minifier: Enable some passing tests (#8250) (camc314)
- 1fa5341 minifier: Port tests from ConvertToDottedPropertiesTest (#8175) (sapphi-red)

## [0.44.0] - 2024-12-25

### Features

- 5397fe9 minifier: Constant fold `undefined?.bar` -> `undefined` (#8075) (Boshen)
- 1932f1e minifier: Fold `foo === undefined || foo === null` (#8063) (翠 / green)

### Bug Fixes

- b605baa minifier: Constant fold strings with tab char (#8096) (Boshen)

### Refactor

- 8b54d89 minifier: Remove parens must happen on enter (#8060) (Boshen)
- 7cb84f3 minifier: Only minify on ast node exit (#8059) (Boshen)
- 77d845a minifier: Fuse DCE AST passes (#8058) (Boshen)
- 6123f5e minifier: Fold statements on exit (#8057) (Boshen)

## [0.42.0] - 2024-12-18

### Features

- db9e93b mangler: Mangle top level variables (#7907) (翠 / green)
- 075bd16 minifier: Fold bitwise operation (#7908) (翠 / green)

### Bug Fixes

- 4799471 minfier: Bigint bitwise operation only works with bigint (#7937) (Boshen)
- de8a86e minifier: Incorrect minification in `try_fold_left_child_op` (#7949) (翠 / green)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)
- 1314c97 minifier: Expose dce as an API instead of an option (#7957) (Boshen)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.40.0] - 2024-12-10

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Refactor


## [0.39.0] - 2024-12-04

- f2f31a8 traverse: [**BREAKING**] Remove unsound APIs (#7514) (overlookmotel)

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features

- 97af341 minifier: Minify alternated one child if block (#7231) (7086cmd)
- ac0d25c minifier: Minify one child if statement expression (#7230) (Ethan Goh)

### Bug Fixes

- 896ff86 minifier: Do not fold if statement block with lexical declaration (#7519) (Boshen)

### Performance

- c133693 minifier: Fuse ast passes (#7493) (Boshen)

### Refactor

- 63a66cf minifier: Remove unused ast pass from DCE (#7540) (Boshen)
- 625a5ba minifier: Improve ast passes (#7518) (Boshen)

### Testing

- 9d6e14b ecmascript: Move tests to `oxc_minifier` due to cyclic dependency with `oxc_parser` (#7542) (Boshen)

## [0.37.0] - 2024-11-21

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)

### Bug Fixes

- cf99be0 minifier: Do not compare bigint with object (#7294) (7086cmd)

### Testing

- 0d6a66a minifier: Fix minimize condition tests (#7222) (7086cmd)

## [0.36.0] - 2024-11-09

### Refactor

- a297765 minifier: Use `map` and `and_then` instead of let else (#7178) (7086cmd)

## [0.35.0] - 2024-11-04

### Refactor

- 97caae1 minifier: Do not use `AstBuilder::*_from_*` methods (#7072) (overlookmotel)
- 2c7ac29 minifier: Remove `Tri`, use `Option<bool>` instead (#6912) (Boshen)

## [0.34.0] - 2024-10-26

### Features

- 4429754 ecmascript: Constant eval `null` to number (#6879) (Boshen)
- fd57e00 ecmascript: Add abstract_relational_comparison to dce (#6846) (Boshen)
- 8bcaf59 minifier: Late peeophole optimization (#6882) (Boshen)
- 860cbca minifier: Implement folding simple arrow fns (#6875) (camc314)
- c26020e minifier: Implement folding String.prototype.replaceAll (#6871) (camc314)
- 50744f3 minifier: Implement folding String.prototype.replace (#6870) (camc314)
- fccf82e minifier: Implement folding `substring` string fns (#6869) (camc314)
- e6a5a1b minifier: Implement folding `charCodeAt` string fns (#6475) (camc314)

### Bug Fixes

- a47c70e minifier: Fix remaining runtime bugs (#6855) (Boshen)
- 686727f minifier: Reference read has side effect (#6851) (Boshen)
- c658d93 minifier: Keep template literals with expressions (#6849) (Boshen)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

### Features

- b4bc300 minifier: Improve folding block stmts (#6793) (camc314)
- 34fe7c0 minifier: Dce meaningless labeled statements (#6688) (7086cmd)

### Bug Fixes

- 2f6ad42 codegen: Print negative bigint `1n- -1n` correctly after constant folding (#6798) (Boshen)
- ca79993 minifier: Do not dce object literals yet (#6839) (Boshen)
- ec5a19b minifier: Do not remove binary expressions (#6829) (Boshen)
- 22355f7 minifier: Do not remove `undefined` for destructuring patterns (#6828) (Boshen)

### Refactor

- 8b25131 minifier: Binary operations use `ConstantEvaluation` (#6700) (Boshen)

## [0.32.0] - 2024-10-19

### Features

- e5ed6a5 codegen: Print negative numbers (#6624) (Boshen)
- 15c04e5 ecmascript: Add feature flag for constant evaluation (Boshen)
- e561880 ecmascript: Add constant_evaluation and side_effects code (#6550) (Boshen)
- 3556062 ecmascript: Add `ConstantEvaluation` (#6549) (Boshen)
- 39c2e66 ecmascript: Add `ToBigInt` and `StringToBigInt` (#6508) (Boshen)
- 6f22538 ecmascript: Add `ToBoolean`, `ToNumber`, `ToString` (#6502) (Boshen)
- 071e564 minifier: Finish implementing folding object expressions (#6586) (camc314)
- 590925a minifier: Finish implementing folding array expressions (#6575) (camc314)
- ef237cf minifier: Complete implementation of statement fusion (#6566) (camc314)
- 97c8a36 minifier: Implement `collapse-variable-declarations` (#6464) (dalaoshu)
- 096e590 minifier: Implement folding `charAt` string fns (#6436) (camc314)
- e5a6f5d minifier: Implement converting template literals to strings (#6486) (camc314)
- 14d0590 minifier: Implement folding of simple function calls (`Boolean`) (#6484) (camc314)
- 7fbc7b6 minifier: Implement folding of simple function calls (`String`) (#6483) (camc314)
- a4f57a4 minifier: Implement folding `indexOf` and `lastIndexOf` string fns (#6435) (camc314)
- 3677ef8 minifier: Dce ExpressionStatements with no side effect (#6457) (7086cmd)
- 06ea121 minifier: Fold for statement (#6450) (7086cmd)
- a9544ae minifier: Partially implement minification for some known string methods (#6424) (camc314)
- 9dc4ee9 minifier: Implement block stmt support for `StatementFusion` (#6422) (camc314)
- ebbf77d minifier: Implement calculations for NumberValue (#6419) (7086cmd)
- 97ac179 minifier: Arithmetic operations for infinity. (#6332) (7086cmd)
- 13b0b0b minifier: Fold literal object constructors on window (#6379) (dalaoshu)

### Bug Fixes

- 389d261 minifier: `~~` operator should only work on numbers (#6598) (Boshen)
- 16bea12 minifier: Use `to_js_string()` instead of `fs64::to_string` (#6597) (Boshen)
- a71e8a0 minifier: Preserve init variable declarations when removing `for` statements during DCE (#6551) (magic-akari)

### Refactor

- 6d041fb ecmascript: Remove `NumberValue` (#6519) (Boshen)
- 856cab5 ecmascript: Move ToInt32 from `oxc_syntax` to `oxc_ecmascript` (#6471) (Boshen)
- f4cdc56 minifier: Use constant folding unary expression from `oxc_ecmascript` (#6647) (Boshen)
- 67ad08a minifier: Unify `ValueType` (#6545) (Boshen)
- bbca743 minifier: Move string methods to `oxc_ecmascript` (#6472) (Boshen)
- 702c049 minifier: Move compress block to dce (#6468) (7086cmd)
- 46a38c6 minifier: Remove allow `clippy::unused_self` (#6441) (Boshen)
- 994b60b minifier: Use builtin get_number_value. (#6335) (7086cmd)
- 435a89c oxc: Remove useless `allocator.alloc(program)` calls (#6571) (Boshen)
- 1a90ec4 rust: Backport v1.82.0 changes to main branch first (#6690) (Boshen)

### Testing

- c5deb32 minifier: Port the rest of tests (#6420) (7086cmd)
- e59da61 minifier: Add all test cases for `collapse_variable_declarations` (#6421) (dalaoshu)
- 73d6a4a minifier: Port all replace_known_methods tests. (#6418) (7086cmd)

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

- 82ab689 transformer,minifier: [**BREAKING**] Move define and inject plugin from minifier to transformer (#6199) (Boshen)

### Features

- e304e8c minifier: Minify exponential arithmetic operation. (#6281) (7086cmd)
- f9ae70c minifier: Minify basic arithmetic calculations. (#6280) (7086cmd)
- 4008afe minifier: Fold array and object constructors (#6257) (camchenry)
- 115ccc9 minifier: Bitwise not in exceeded value. (#6235) (7086cmd)
- ee6c850 minifier: Scaffold peephole replace known methods. (#6245) (7086cmd)
- c32af57 minifier: Fold demical bitwise not for bigint. (#6233) (7086cmd)
- 23b6464 minifier: Fold true / false comparison. (#6225) (7086cmd)
- 585ccda minifier: Support subtraction assignment. (#6214) (7086cmd)
- cca0034 minifier: Handle positive `NaN` and `Infinity`. (#6207) (7086cmd)
- dac8f09 minifier: Minify unary plus negation. (#6203) (7086cmd)
- 3b79e1b minifier: Evaluate bigint in fold constant (#6178) (Boshen)
- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

### Bug Fixes

- d953a6b minifier: Correct the reference link (#6283) (dalaoshu)
- 37cbabb minifier: Should not handle the strict operation for bool comparison. (#6261) (7086cmd)
- e29c067 minifier: Handle exceeded shifts. (#6237) (7086cmd)

### Refactor

- ac5a23f minifier: Use ctx.ast.vec instead of Vec::new. (#6331) (7086cmd)
- 1cee207 minifier: Some boilerplate work for PeepholeFoldConstants (#6054) (Boshen)

### Testing

- 964d71e minifier: Add arithmetic tests for fold constants. (#6269) (7086cmd)
- fcb4651 minifier: Enable null comparison with bigint. (#6252) (7086cmd)

## [0.30.2] - 2024-09-27

### Features

- 60c52ba ast: Allow passing span to `void_0` method (#6065) (Dunqing)

### Bug Fixes

- e0a8959 minifier: Compute `void number` as `undefined` (#6028) (Boshen)

## [0.30.1] - 2024-09-24

### Features

- 5c323a2 minifier: Loop compressor passes (#6013) (Boshen)

### Refactor

- 0a2f687 minifier: Move dce conditional expression to `RemoveDeadCode` (#5971) (Boshen)

## [0.30.0] - 2024-09-23

### Features

- 9076dee minifier: Implement part of `StatementFusion` (#5936) (Boshen)

### Bug Fixes

- 362c427 mangler,codegen: Do not mangle top level symbols (#5965) (Boshen)

### Refactor

- 943bd76 minifier: Move tests to their src files (#5912) (Boshen)
- cbaeea6 minifier: Clean up some tests (#5910) (Boshen)
- 144611e minifier: Align ast pass names with closure compiler (#5908) (Boshen)

## [0.29.0] - 2024-09-13

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)
- e968e9f minifier: Constant fold nullish coalescing operator (#5761) (Boshen)
- 6bc13f6 minifier: Add `MinimizeConditions` pass (#5747) (Boshen)

### Bug Fixes

- 8ff013a minifier: Handle dce CallExpression::callee (#5752) (Boshen)

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

### Refactor

- 2890c98 minifier: Add tests for `remove_syntax` (#5749) (Boshen)
- 9a9d8f6 minifier: Replace `self.ast` with `ctx.ast` (#5748) (Boshen)
- 746f7b3 minifier: Align code with closure compiler (#5717) (Boshen)
- 21e2df5 minifier: Replace `VisitMut` with `Traverse` for inject and define plugins (#5705) (Boshen)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Features

- 68c3cf5 minifier: Fold `void 1` -> `void 0` (#5670) (Boshen)
- c6bbf94 minifier: Constant fold unary expression (#5669) (Boshen)
- 86256ea minifier: Constant fold `typeof` (#5666) (Boshen)

### Bug Fixes

- b8f8dd6 minifier/replace_global_defines: Do not replace shadowed identifiers (#5691) (Boshen)

### Performance


### Refactor

- 067f9b5 semantic: Introduce `IsGlobalReference` trait (#5672) (Boshen)

## [0.27.0] - 2024-09-06

### Features

- ba4b68c minifier: Remove parenthesized expression for dce (#5439) (Boshen)

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features

- 2b21be3 oxc_minifier: Define plugin with postfix wildcard (#4979) (IWANABETHATGUY)

### Refactor

- 0f64d10 minifier: Remove duplicated helper `move_out_expression` (#5007) (IWANABETHATGUY)
- b4407c4 oxc,mangler: `oxc` crate add mangler; mangler use options API (Boshen)

## [0.24.3] - 2024-08-18

### Bug Fixes

- 46cb1c1 minifier: Handle `Object.definedPropert(exports` for @babel/types/lib/index.js (#4933) (Boshen)
- 81fd637 minifier: Do not fold `0 && (module.exports = {})` for `cjs-module-lexer` (#4878) (Boshen)
- 879a271 minifier: Do not join `require` calls for `cjs-module-lexer` (#4875) (Boshen)

## [0.24.2] - 2024-08-12

### Performance

- 504ac0b minifier: `InjectGlobalVariables` only add to `replaced_dot_defines` once for each (#4803) (overlookmotel)
- 35f2742 minifier: Avoid repeated `Atom` creation in `InjectGlobalVariables` (#4802) (overlookmotel)

## [0.24.1] - 2024-08-10

### Features

- c519295 minifier: Add `InjectGlobalVariables` plugin (`@rollup/plugin-inject`) (#4759) (Boshen)

## [0.24.0] - 2024-08-08

### Features

- 229a0e9 minifier: Implement dot define for member expressions (#3959) (camc314)

### Bug Fixes

- 94d3c31 minifier: Avoid removing function declaration from `KeepVar` (#4722) (Boshen)
- bf43148 minifier: Do not `remove_syntax` in dead_code_elimination (Boshen)
- bf48c7f minifier: Fix `keep_var` keeping vars from arrow functions (#4680) (Boshen)
- 9be29af minifier: Temporarily fix shadowed `undefined` variable (#4678) (Boshen)
- e8b662a minifier: Various fixes to pass minifier conformance (#4667) (Boshen)

### Performance

- 0f5e982 minifier: Only visit arrow expression after dropping `console.log` (#4677) (Boshen)

### Refactor

- fbfd852 minifier: Add `NodeUtil` trait for accessing symbols on ast nodes (#4734) (Boshen)
- e0832f8 minifier: Use `oxc_traverse` for AST passes (#4725) (Boshen)
- 17602db minifier: Move tests and files around (Boshen)
- 3289477 minifier: Clean up tests (#4724) (Boshen)
- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.1] - 2024-08-06

### Features

- 229a0e9 minifier: Implement dot define for member expressions (#3959) (camc314)

### Bug Fixes

- bf48c7f minifier: Fix `keep_var` keeping vars from arrow functions (#4680) (Boshen)
- 9be29af minifier: Temporarily fix shadowed `undefined` variable (#4678) (Boshen)
- e8b662a minifier: Various fixes to pass minifier conformance (#4667) (Boshen)

### Performance

- 0f5e982 minifier: Only visit arrow expression after dropping `console.log` (#4677) (Boshen)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.0] - 2024-08-01

### Features

- a558492 codegen: Implement `BinaryExpressionVisitor` (#4548) (Boshen)

### Bug Fixes

- 6a94e3f codegen: Fixes for esbuild test cases (#4503) (Boshen)

## [0.22.0] - 2024-07-23

### Features

- 0deb027 minfier: Dce `if (xxx) else if (false) { REMOVE }` (#4407) (Boshen)
- e33ec18 minifier: Compress `typeof foo == "undefined"` into `typeof foo > "u"` (#4412) (Boshen)

### Bug Fixes

- 267f7c4 minifier: Skip `Object.defineProperty(exports, ...)` for `cjs-module-lexer` (#4409) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- 5d17675 mangler: Add debug mode (#4314) (Boshen)
- e3e663b mangler: Initialize crate and integrate into minifier (#4197) (Boshen)
- c818472 minifier: Dce conditional expression `&&` or `||` (#4190) (Boshen)

### Bug Fixes

- e167ef7 codegen: Print parenthesis properly (#4245) (Boshen)
- f144082 minifier: RemoveDeadCode should visit nested expression (#4268) (underfin)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)

## [0.20.0] - 2024-07-11

### Features

- 54cd04a minifier: Implement dce with var hoisting (#4160) (Boshen)
- 44a894a minifier: Implement return statement dce (#4155) (Boshen)

## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features

- c6c16a5 minifier: Dce all conditional expressions (#4135) (Boshen)

## [0.17.1] - 2024-07-06

### Bug Fixes

- 719fb96 minifier: Omit dce `undefined` which can be a shadowed variable (#4073) (Boshen)

## [0.17.0] - 2024-07-05

### Features

- 0da9dfb minifier: Add constant folding to remove dead code (#4058) (Boshen)

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)

### Refactor

- edb557c minifier: Add a folder struct for constant folding (#4057) (Boshen)

## [0.16.2] - 2024-06-30

### Performance

- 1eac3d2 semantic: Use `Atom<'a>` for `Reference`s (#3972) (Don Isaac)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

### Features

- dd540c8 minifier: Add skeleton for ReplaceGlobalDefines ast pass (#3803) (Boshen)
- f3c3970 minifier: Add skeleton for RemoveDeadCode ast pass (#3802) (Boshen)

### Bug Fixes


### Refactor

- 8027b1e minifier: Change prepass to ast_passes::remove_parens (#3801) (Boshen)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 534242a codegen: [**BREAKING**] Remove `CodegenOptions::enable_typescript` (#3674) (Boshen)

### Features

- 38a75e5 coverage: Improve codegen (#3729) (Boshen)

### Bug Fixes

- 8f64d99 minifier: Respect `join_vars: false` option (#3724) (mysteryven)

## [0.14.0] - 2024-06-12

### Refactor

- e90e6a2 minifier: Make `Prepass` `Copy` (#3603) (overlookmotel)

## [0.13.4] - 2024-06-07

### Bug Fixes

- affb2c8 codegen: Print indentation before directive (#3512) (Dunqing)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

## [0.13.0] - 2024-05-14

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)

## [0.11.0] - 2024-03-30

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.10.0] - 2024-03-14

### Features

- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)

### Refactor
- cbc2f5f Remove unused dependencies (#2718) (Boshen)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 903f17c span: Move base54 method to mangler (#2523) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)

### Refactor

- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)- a2c173d Remove `panic!` from examples (#2454) (Boshen)- 70a0076 Remove global allocator from non-user facing apps (#2401) (Boshen)

## [0.7.0] - 2024-02-09

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 1ee6d8c codegen: Move string test to codegen (#2150) (Wenzhe Wang)
- 18a58d4 minifier: Handle more expressions for side effects (#2062) (Bradley Farias)

### Bug Fixes

- 29dc5e6 codegen: Add parenthesis in binary expression by precedence (#2067) (Wenzhe Wang)

## [0.4.0] - 2023-12-08

### Features

- c6ad660 semantic: Support scope descendents starting from a certain scope. (#1629) (Miles Johnson)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 809f050 codegen: Move minifying printer to codegen crate (#985) (Boshen)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 14e1dac minifier: Reenable minifier tests (#969) (Boshen)
- f0029d5 minifier: Reenable mangler (Boshen)
- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- 5b1e1e5 parser: TypeScript 5.2 (#811) (Cameron)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- ce79bc1 transform_conformance: Move Formatter to codegen (#986) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)- 0e91044 Adjust the order of print semicolon (#1003) (Wenzhe Wang)

### Refactor

- 4787220 ast: Clean up some methods (Boshen)
- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- db5417f clippy: Allow clippy::too_many_lines (Boshen)
- eaeb630 clippy: Allow struct_excessive_bools (Boshen)
- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 110059f rust: Change `RefCell.clone().into_inner()` to `RefCell.get()` (Boshen)

## [0.2.0] - 2023-09-14

### Features

- 027a67d minifier: Constant addition expression folding (#882) (Don Isaac)
- e090b56 minifier: Initialize conditions folding (#658) (阿良仔)
- c5ff534 semantic: Add `node_id` to `Reference` (#689) (Makoto Tateno)

### Refactor

- 3516759 ast: Use `atom` for `Directive` and `Hashbang` (#701) (Yunfei He)

