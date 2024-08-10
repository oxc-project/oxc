# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

