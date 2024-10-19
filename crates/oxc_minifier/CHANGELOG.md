# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

