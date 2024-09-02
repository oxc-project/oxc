# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- 5f4c9ab semantic: [**BREAKING**] Rename `SymbolTable::get_flag` to `get_flags` (#5030) (overlookmotel)

- 58bf215 semantic: [**BREAKING**] Rename `Reference::flag` and `flag_mut` methods to plural (#5025) (overlookmotel)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- c30e2e9 semantic: [**BREAKING**] `Reference::flag` method return `ReferenceFlag` (#5019) (overlookmotel)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 4b49cf8 transformer: Always pass in symbols and scopes (#5087) (Boshen)
- f51d3f9 transformer/nullish-coalescing-operator: Handles nullish coalescing expression in the FormalParamter (#4975) (Dunqing)
- f794870 transformer/nullish-coalescing-operator: Generate the correct binding name (#4974) (Dunqing)
- 72ff2c6 transformer/nullish-coalescing-operator: Add comments in top of file (#4972) (Dunqing)

### Bug Fixes

- 6ffbd78 transformer: Remove an `AstBuilder::copy` call from TS namespace transform (#4987) (overlookmotel)
- a8dfdda transformer: Remove an `AstBuilder::copy` call from TS module transform (#4986) (overlookmotel)
- 1467eb3 transformer: Remove an `AstBuilder::copy` call from TS enum transform (#4985) (overlookmotel)
- 1365feb transformer: Remove an `AstBuilder::copy` call for TS `AssignmentTarget` transform (#4984) (overlookmotel)
- edacf93 transformer: Remove an `AstBuilder::copy` call (#4983) (overlookmotel)
- 3b35332 transformer/logical-assignment-operators: Fix semantic errors (#5047) (Dunqing)

### Documentation

- 178d1bd transformer: Add documentation for exponentiation-operator plugin (#5084) (Dunqing)
- d50eb72 transformer: Add documentation for `optional-catch-binding` plugin (#5064) (Dunqing)
- 4425b17 transformer: Add documentation for `logical-assignment-operators` plugin (#5012) (Dunqing)
- 1bd5853 transformer: Updated README re: order of methods (#4993) (overlookmotel)

### Refactor

- cca7440 ast: Replace `AstBuilder::move_statement_vec` with `move_vec` (#4988) (overlookmotel)
- 96422b6 ast: Make AstBuilder non-exhaustive (#4925) (DonIsaac)
- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)
- 8d15e65 transformer: Use `into_member_expression` (#5006) (overlookmotel)
- 4796ece transformer: TS annotations transform use `move_expression` (#4982) (overlookmotel)
- a9fcf29 transformer/es2016: Move all entry points to implementation of Traverse trait (#5085) (Dunqing)
- deda6ac transformer/es2019: Move all entry points to implementation of Traverse trait (#5065) (Dunqing)
- 9df2f80 transformer/es2020: Move all entry points to implementation of Traverse trait (#4973) (Dunqing)
- 3f9433c transformer/es2021: Move all entry points to implementation of Traverse trait (#5013) (Dunqing)
- c60a50d transformer/exponentiation-operator: Use built-in `ctx.clone_identifier_reference` (#5086) (Dunqing)
- bcc8da9 transformer/logical-assignment-operator: Use `ctx.clone_identifier_reference` (#5014) (Dunqing)
- 38d4434 transformer/nullish-coalescing-operator: Move internal methods to bottom of the file (#4996) (Dunqing)

## [0.24.3] - 2024-08-18

### Features

- d49fb16 oxc_codegen: Support generate range leading comments (#4898) (IWANABETHATGUY)
- f1fcdde transformer: Support react fast refresh (#4587) (Dunqing)
- 0d79122 transformer: Support logical-assignment-operators plugin (#4890) (Dunqing)
- ab1d08c transformer: Support `optional-catch-binding` plugin (#4885) (Dunqing)
- 69da9fd transformer: Support nullish-coalescing-operator plugin (#4884) (Dunqing)
- 3a66e58 transformer: Support exponentiation operator plugin (#4876) (Dunqing)
- f88cbcd transformer: Add `BoundIdentifier::new_uid_in_current_scope` method (#4903) (overlookmotel)
- 1e6d0fe transformer: Add methods to `BoundIdentifier` (#4897) (overlookmotel)

### Bug Fixes

- 2476dce transformer: Remove an `ast.copy` from `NullishCoalescingOperator` transform (#4913) (overlookmotel)
- 248a757 transformer/typescript: Typescript syntax within `SimpleAssignmentTarget` with `MemberExpressions` is not stripped (#4920) (Dunqing)

### Documentation

- 9c700ed transformer: Add README including style guide (#4899) (overlookmotel)

### Refactor

- 1eb59d2 ast, isolated_declarations, transformer: Mark `AstBuilder::copy` as an unsafe function (#4907) (overlookmotel)
- 452187a transformer: Rename `BoundIdentifier::new_uid_in_root_scope` (#4902) (overlookmotel)
- 707a01f transformer: Re-order `BoundIdentifier` methods (#4896) (overlookmotel)
- 117dff2 transformer: Improve comments for `BoundIdentifier` helper (#4895) (overlookmotel)

## [0.24.2] - 2024-08-12

### Bug Fixes

- 62f759c transformer/typescript: Generated assignment for constructor arguments with access modifiers should be injected to the top of the constructor (#4808) (Dunqing)

## [0.24.0] - 2024-08-08

- 75f2207 traverse: [**BREAKING**] Replace `find_scope` with `ancestor_scopes` returning iterator (#4693) (overlookmotel)

- 506709f traverse: [**BREAKING**] Replace `find_ancestor` with `ancestors` returning iterator (#4692) (overlookmotel)

### Bug Fixes

- 4797eaa transformer: Strip TS statements from for in/of statement bodies (#4686) (overlookmotel)
- 5327acd transformer/react: The `require` IdentifierReference does not have a `reference_id` (#4658) (Dunqing)
- 3987665 transformer/typescript: Incorrect enum-related `symbol_id`/`reference_id` (#4660) (Dunqing)
- 4efd54b transformer/typescript: Incorrect `SymbolFlags` for jsx imports (#4549) (Dunqing)

### Refactor

- 83546d3 traverse: Enter node before entering scope (#4684) (overlookmotel)

## [0.23.1] - 2024-08-06

### Bug Fixes

- 5327acd transformer/react: The `require` IdentifierReference does not have a `reference_id` (#4658) (Dunqing)
- 3987665 transformer/typescript: Incorrect enum-related `symbol_id`/`reference_id` (#4660) (Dunqing)
- 4efd54b transformer/typescript: Incorrect `SymbolFlags` for jsx imports (#4549) (Dunqing)

## [0.23.0] - 2024-08-01

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Refactor

- 96602bf transformer/typescript: Determine whether to remove `ExportSpeicifer` by `ReferenceFlags` (#4513) (Dunqing)

## [0.22.1] - 2024-07-27

### Bug Fixes

- c04b9aa transformer: Add to `SymbolTable::declarations` for all symbols (#4460) (overlookmotel)
- ecdee88 transformer/typescript: Incorrect eliminate exports when the referenced symbol is both value and type (#4507) (Dunqing)

### Refactor

- f17254a semantic: Populate `declarations` field in `SymbolTable::create_symbol` (#4461) (overlookmotel)

## [0.22.0] - 2024-07-23

- 85a7cea semantic: [**BREAKING**] Remove name from `reference` (#4329) (Dunqing)

### Refactor


## [0.21.0] - 2024-07-18

### Features

- 7eb960d transformer: Decode xml character entity `&#xhhhh` and `&#nnnn;` (#4235) (Boshen)

### Refactor

- a197e01 transformer/typescript: Remove unnecessary code (#4321) (Dunqing)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)

### Refactor


## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features


## [0.17.2] - 2024-07-08

### Bug Fixes

- 4413e2d transformer: Missing initializer for readonly consructor properties (#4103) (Don Isaac)

## [0.17.0] - 2024-07-05

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)

## [0.16.3] - 2024-07-02

### Bug Fixes

- bdee156 transformer/typescript: `declare class` incorrectly preserved as runtime class (#3997) (Dunqing)
- a50ce3d transformer/typescript: Missing initializer for class constructor arguments with `private` and `protected` modifier (#3996) (Dunqing)

## [0.16.2] - 2024-06-30

### Performance

- 1eac3d2 semantic: Use `Atom<'a>` for `Reference`s (#3972) (Don Isaac)

### Refactor

- 5845057 transformer: Pass in symbols and scopes (#3978) (Boshen)

## [0.16.1] - 2024-06-29

### Refactor

- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.16.0] - 2024-06-26

- 1f85f1a ast: [**BREAKING**] Revert adding `span` field to the `BindingPattern` type. (#3899) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 5501d5c transformer/typescript: Transform `import {} from "mod"` to import `"mod"` (#3866) (Dunqing)

### Bug Fixes

- 08fcfb3 transformer: Fix spans and scopes in TS enum transform (#3911) (overlookmotel)
- 17ad8f7 transformer: Create new scopes for new blocks in TS transform (#3908) (overlookmotel)
- d76f34b transformer: TODO comments for missing scopes (#3837) (overlookmotel)
- e470731 transformer: TS transform handle when type exports first (#3833) (overlookmotel)
- d774e54 transformer: TS transform generate do not copy statements (#3832) (overlookmotel)
- ff1da27 transformer: Correct comment in example (#3831) (overlookmotel)
- 6dcc3f4 transformer: Fix TS annotation transform scopes (#3816) (overlookmotel)
- aea3e9a transformer: Correct spans for TS annotations transform (#3782) (overlookmotel)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- 5ef28b7 transformer: Shorten code (#3912) (overlookmotel)
- d9f268d transformer: Shorten TS transform code (#3836) (overlookmotel)
- 21b0d01 transformer: Pass ref to function (#3781) (overlookmotel)
- 7c44703 transformer: Remove needless `pub` on TS enum transform methods (#3774) (overlookmotel)
- 22c56d7 transformer: Move TSImportEqualsDeclaration transform code (#3764) (overlookmotel)
- cd56aa9 transformer: Simplify TS export assignment transform (#3762) (overlookmotel)
- 512740d transformer: Move and simplify TS enum transform entry point (#3760) (overlookmotel)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

### Features

- 750a534 coverage: Transformer idempotency test (#3691) (Boshen)
- 4f2db46 transformer-dts: `--isolatedDeclarations` dts transform (#3664) (Dunqing)

### Bug Fixes

- 59666e0 transformer: Do not rename accessible identifier references (#3623) (Dunqing)

## [0.14.0] - 2024-06-12

### Bug Fixes

- 35e267b transformer: Arrow function transform use UIDs for `_this` vars (#3634) (overlookmotel)
- 39bdebc transformer: Arrow func transform maintain scope ID (#3633) (overlookmotel)
- 5cb7e6a transformer: Arrow func transform use correct spans (#3630) (overlookmotel)
- 0c4ccb4 transformer: Arrow function transform alter `</this>` (#3627) (overlookmotel)
- 8d237c4 transformer: JSX source calculate correct column when Unicode chars (#3615) (overlookmotel)
- 9e8f4d6 transformer: Do not add `__source` for generated nodes (#3614) (overlookmotel)
- 0fb4c35 transformer: Use UID for JSX source filename var (#3612) (overlookmotel)

### Performance

- 3a59294 transformer: React display name transform reduce Atom allocations (#3616) (overlookmotel)
- f4c1389 transformer: Create `Vec` with capacity (#3613) (overlookmotel)

### Refactor

- 08f1010 ast: Make `AstBuilder` `Copy` (#3602) (overlookmotel)
- 89bcbd5 transformer: Move `BoundIdentifier` into helpers (#3610) (overlookmotel)
- 5793ff1 transformer: Replace `&’a Trivias` with `Rc<Trivias>` (#3580) (Dunqing)
- 509871f transformer: Comment for unimplemented `spec` option in arrow fns transform (#3618) (overlookmotel)
- 4b2e3a7 transformer: Fix indentation (#3617) (overlookmotel)
- 3467e3d transformer: Remove outdated comment (#3606) (overlookmotel)
- a799225 transformer: Flatten file structure for React transform (#3604) (overlookmotel)
- 70f31a8 transformer: Reduce branching in JSX transform (#3596) (overlookmotel)
- 3ae567d transformer: Remove dead code (#3588) (overlookmotel)
- 60cbdec traverse: `generate_uid_in_root_scope` method (#3611) (overlookmotel)

## [0.13.4] - 2024-06-07

### Features

- 646b993 coverage/transformer: Handle @jsx option (#3553) (Dunqing)
- a939ddd transformer/typescript: Remove more typescript ast nodes (#3563) (Dunqing)
- e8a20f8 transformer/typescript: Remove typescript ast nodes (#3559) (Dunqing)
- ee9a215 transformer/typescript: Handle namespace directive correctly (#3532) (Dunqing)

### Bug Fixes

- f6939cb transformer: Store `react_importer` in `Bindings` in JSX transform (#3551) (overlookmotel)
- 7982b93 transformer: Correct spans for JSX transform (#3549) (overlookmotel)
- c00598b transformer: JSX set `reference_id` on refs to imports (#3524) (overlookmotel)

### Performance

- 37cdc13 transformer: Faster checks if JSX plugin enabled (#3577) (overlookmotel)
- 9f467b8 transformer: Avoid fragment update where possible (#3535) (overlookmotel)
- ac394f0 transformer: JSX parse pragma only once (#3534) (overlookmotel)

### Refactor

- f2113ae transformer: Reduce cloning and referencing `Rc`s (#3576) (overlookmotel)
- 0948124 transformer: Pass `Rc`s by value (#3550) (overlookmotel)
- e4d74ac transformer: Remove `update_fragment` from JSX transform (#3541) (overlookmotel)
- 73b7864 transformer: Combine import and usage in JSX transform (#3540) (overlookmotel)
- 6978269 transformer/typescript: Replace reference collector with symbols references (#3533) (Dunqing)

## [0.13.3] - 2024-06-04

### Bug Fixes

- 591c54b transformer: JSX set `symbol_id` on imports (#3523) (overlookmotel)
- 837776e transformer: TS namespace transform do not track var decl names (#3501) (overlookmotel)
- 8d2beff transformer: Use correct scope for TS namespaces (#3489) (overlookmotel)
- 8e4f335 transformer: Output empty file for TS definition files (#3500) (overlookmotel)

### Performance

- 91519d9 transformer: React JSX reduce allocations (#3522) (overlookmotel)
- f3a755c transformer: React JSX reuse same `Atom`s (#3521) (overlookmotel)

### Refactor

- 7bbd3da traverse: `generate_uid` return `SymbolId` (#3520) (overlookmotel)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)
- 574629e tasks/coverage: Turn on idempotency testing for transformer (#3470) (Dunqing)
- 816a782 transformer: Support `targets` option of preset-env (#3371) (Dunqing)
- 92df98b transformer/typescript: Report error that do not allow namespaces (#3448) (Dunqing)
- a6b073a transformer/typescript: Report error for namespace exporting non-const (#3447) (Dunqing)
- 150255c transformer/typescript: If within a block scope, use let to declare enum name (#3446) (Dunqing)
- e80552c transformer/typescript: If binding exists, variable declarations are not created for namespace name (#3445) (Dunqing)
- 241e8d1 transformer/typescript: If the binding exists, the identifier reference is not renamed (#3387) (Dunqing)

### Bug Fixes

- 90b0f6d transformer: Use UIDs for React imports (#3431) (overlookmotel)
- d4371e8 transformer: Use UIDs in TS namespace transforms (#3395) (overlookmotel)
- baed1ca transformer/jsx-source: Add filename statement only after inserting the source object (#3469) (Dunqing)
- b4fd1ad transformer/typescript: Variable declarations are not created when a function has a binding with the same name (#3460) (Dunqing)

### Refactor

- 55bbde2 ast: Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488) (overlookmotel)
- 84feceb transformer: Explicit skip TS statements in TS namespace transform (#3479) (overlookmotel)
- 7f7b5ea transformer: Shorter code in TS namespace transform (#3478) (overlookmotel)
- 7e7b452 transformer: Panic on illegal cases in TS namespace transform (#3477) (overlookmotel)
- 8e089a9 transformer: Rename var (#3476) (overlookmotel)
- 0f69ffd transformer: Shorten code in TS namespace transform (#3468) (overlookmotel)
- deef86a transformer: Remove unreachable code from TS namespace transform (#3475) (overlookmotel)
- 9dc58d5 transformer/typescript: Use a memory-safe implementation instead (#3481) (Dunqing)
- 1a50b86 typescript/namespace: Reuse TSModuleBlock's scope id (#3459) (Dunqing)

## [0.13.1] - 2024-05-22

### Features

- e2c6fe0 transformer: Report errors when options have unknown fields (#3322) (Dunqing)
- 9ee962a transformer: Support `from_babel_options` in TransformOptions (#3301) (Dunqing)
- b9d69ad transformer: Do not add self attribute in react/jsx plugin (#3287) (Dunqing)
- 421107a traverse: Pass `&mut TraverseCtx` to visitors (#3312) (overlookmotel)

### Bug Fixes

- b4fa27a transformer: Do no add __self when the jsx is inside constructor (#3258) (Dunqing)

### Refactor

- c9d84af diagnostics: S/warning/warn (Boshen)
- e7a6595 transformer: Correct spelling of var name (#3369) (overlookmotel)
- dad47a5 transformer: Improve indentation (#3282) (overlookmotel)
- 05c71d2 traverse: `Traverse` produce scopes tree using `Semantic` (#3304) (overlookmotel)

## [0.13.0] - 2024-05-14

### Features

- f1ccbd4 syntax: Add `ToJsInt32` trait for f64 (#3132) (Boshen)
- 870d11f syntax: Add `ToJsString` trait for f64 (#3131) (Boshen)
- 34dd53c transformer: Report ambient module cannot be nested error (#3253) (Dunqing)
- 1b29e63 transformer: Do not elide jsx imports if a jsx element appears somewhere (#3237) (Dunqing)
- 905ee3f transformer: Add arrow-functions plugin (#3083) (Dunqing)
- 78875b7 transformer: Implement typescript namespace (#3025) (Boshen)
- a52e321 transformer/jsx-source: Get the correct lineNumber and columnNumber from the span. (#3142) (Dunqing)
- 18d853b transformer/react: Support development mode (#3143) (Dunqing)
- be8fabe transformer/react: Enable jsx plugin when development is true (#3141) (Dunqing)

### Bug Fixes

- 9590eb0 transform: Implement `transform-react-display-name` with bottom-up lookup (#3183) (overlookmotel)
- 6ac8a84 transformer: Correctly jsx-self inside arrow-function (#3224) (Dunqing)
- b589496 transformer/arrow-functions: Should not transform `this` in class (#3129) (Dunqing)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- be958ce transform: Transformer use `Traverse` (#3182) (overlookmotel)
- 7067f9c transformer: Clean up more diagnostics (Boshen)
- d351f2d transformer: Unify diagnostics (Boshen)
- 9525653 transformer: Remove no-op scopes code (#3210) (overlookmotel)
- a63a45d transformer: Remove the requirement of `Semantic` (#3140) (Boshen)
- 843318c transformer/typescript: Reimplementation of Enum conversion based on Babel (#3102) (Dunqing)- d8173e1 Remove all usages of `Into<Error>` (Boshen)

## [0.12.5] - 2024-04-22

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)
- 383b449 ast: Box `ImportDeclarationSpecifier` enum variants (#3061) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- b6b63ac transform_conformance: Skip tests with plugin.js (#2978) (Boshen)
- ef602af transform_conformance: Skip plugins we don't support yet (#2967) (Boshen)
- 85a3653 transformer: Add "_jsxFileName" variable in jsx source plugin (#3000) (Boshen)
- e43c245 transformer: Add import helpers to manage module imports (#2996) (Boshen)
- c211f1e transformer: Add diagnostics to react transform (#2974) (Boshen)
- 3a6eae1 transformer: Apply jsx self and source plugin inside jsx transform (#2966) (Boshen)
- bd9fc6d transformer: React jsx transform (#2961) (Boshen)
- e673550 transformer: Start on TypeScript annotation removal (#2951) (Miles Johnson)
- e651e50 transformer: Add the most basic plugin toggles (#2950) (Boshen)
- 1475477 transformer: Implement react-jsx-source (#2948) (Boshen)
- f903a22 transformer: Implement react-jsx-self (#2946) (Boshen)
- 0c04bf7 transformer: Transform TypeScript namespace (#2942) (Boshen)
- 3419306 transformer: Add filename (#2941) (Boshen)
- b72bdca transformer/react: Reports duplicate __self/__source prop error (#3009) (Dunqing)
- 3831147 transformer/typescript: Report error for export = <value> (#3021) (Dunqing)
- 7416de2 transformer/typescript: Reports error for import lib = require(...); (#3020) (Dunqing)
- e14ac17 transformer/typescript: Insert this assignment after the super call (#3018) (Dunqing)
- afb1dd4 transformer/typescript: Support for transform TSImportEqualsDeclaration (#2998) (Dunqing)
- 6732e8b transformer/typescript: Support for transform enum (#2997) (Dunqing)
- 6a53fa3 transformer/typescript: Correct elide imports/exports statements (#2995) (Dunqing)

### Bug Fixes

- 722d4c2 transformer: `TypeScriptOptions` deserialize should fallback to default (#3012) (Boshen)
- 6704546 transformer: React `development` default value should be false (#3002) (Boshen)
- c7e70c8 transformer: Deserialize ReactJsxRuntime with camelCase (#2972) (Boshen)
- 10814d5 transformer: Turn on react preset by default (#2968) (Boshen)
- 35e3b0f transformer: Fix incorrect jsx whitespace text handling (#2969) (Boshen)
- 99e038c transformer/typescript: Modifiers should not be removed (#3005) (Dunqing)

### Refactor

- 82e00bc transformer: Remove boilerplate code around decorators to reduce noise (#2991) (Boshen)
- 60ccbb1 transformer: Clean up some code (#2949) (Boshen)

## [0.12.3] - 2024-04-11

### Features

- 02adc76 transformer: Implement plugin-transform-react-display-name top-down (#2937) (Boshen)
- 255c74c transformer: Add transform context to all plugins (#2931) (Boshen)
- 79ca6fe transformer: Add transform callback methods (#2929) (Boshen)
- d65eab3 transformer: Add react preset (#2921) (Boshen)

## [0.12.1] - 2024-04-03

### Features

- 7710d8c transformer: Add compiler assumptions (#2872) (Boshen)
- 7034bcc transformer: Add proposal-decorators (#2868) (Boshen)
- ffadcb0 transformer: Add react plugins (#2867) (Boshen)
- 293b9f4 transformer: Add `transform-typescript` boilerplate (#2866) (Boshen)

### Bug Fixes

- 21a5e44 transformer: Add serde "derive" feature to fix compile error (Boshen)

## [0.11.0] - 2024-03-30

### Features

- 243131d transformer: Numeric separator plugin. (#2795) (Ali Rezvani)
- 56493bd transformer: Add transform literal for numeric literals. (#2797) (Ali Rezvani)
- 398a034 transformer/typescript: Remove `verbatim_module_syntax` option (#2796) (Dunqing)

### Bug Fixes

- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)
- 528744c transformer: Optional-catch-binding unused variable side effect (#2822) (Ali Rezvani)

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- 813226b ast: Get rid of unsafe transmutation in VisitMut trait. (#2764) (Ali Rezvani)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)
- fe12617 transformer: Pass options via context. (#2794) (Ali Rezvani)

## [0.10.0] - 2024-03-14

### Features

- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)
- 308b780 transformer/decorators: Handling the coexistence of class decorators and member decorators (#2636) (Dunqing)

### Bug Fixes

- 2a235d3 ast: Parse `with_clause` in re-export declaration (#2634) (magic-akari)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Features

- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- f760108 transformer: Call build module record (#2529) (Dunqing)
- 6d43e85 transformer/typescript: Support transform constructor method (#2551) (Dunqing)

### Bug Fixes

- 258b9b1 ast: Support FormalParameter.override (#2577) (Arnaud Barré)
- 7a12514 transformer/decorators: Missing check private function (#2607) (Dunqing)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 2c2256a transformer/typescript: Improve implementation of remove import/export (#2530) (Dunqing)

## [0.8.0] - 2024-02-26

### Features

- 70295a5 ast: Update arrow_expression to arrow_function_expression (#2496) (Dunqing)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)
- cd75c1c transformer/decorators: Insert only one private in expression (#2486) (Dunqing)
- 3d008ab transformer/decorators: Insert instanceBrand function (#2480) (Dunqing)
- 2628c97 transformer/decorators: Transform getter function (#2473) (Dunqing)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)
- 27b2c21 transformer/decorators: If it is a private method definition, transform it (#2427) (Dunqing)
- 4b11183 transformer/decorators: Move get_decorator_info inside the decorators (#2426) (Dunqing)

## [0.6.0] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- 165f948 ast: Remove expression property from Function (#2247) (Dunqing)
- 9e598ff transformer: Add decorators plugin (#2139) (Dunqing)
- 02c18d8 transformer/decorators: Support for static and private member decorators (#2246) (Dunqing)
- ba85b09 transformer/decorators: Support method decorator and is not static (#2238) (Dunqing)
- a79988d transformer/decorators: Support static member (#2235) (Dunqing)
- 3b85e18 transformer/decorators: Ensure property key consistency (#2233) (Dunqing)
- e5719e9 transformer/decorators: Support transform member decorators (#2171) (Dunqing)
- 7f89bfe transformer/decorators: Support version 2023-05 (#2152) (Dunqing)
- 04b401c transformer/decorators: Support transform the class decorators in export declaration (#2145) (Dunqing)
- b5b2ef3 transformer/typescript: Improve function parameters name (#2079) (Dunqing)
- 7711f7a transformer/typescript: Support only_remove_type_imports option (#2077) (Dunqing)
- f5bf5de transformer/typescript: Support transform exported TSModuleBlock (#2076) (Dunqing)
- 56ca8b6 transformer/typescript: Support transform namespace (#2075) (Dunqing)
- b89e84c transformer/typescript: Keep imports if import specifiers is empty (#2058) (Dunqing)
- 3413bb3 transformer/typescript: Remove type-related exports (#2056) (Dunqing)
- 95d741a transformer/typescript: Remove type only imports/exports correctly (#2055) (Dunqing)
- 6c7f983 transformer/typescript: Remove export specifier that import_kind is type (#2015) (Dunqing)
- ead4e8d transformer/typescript: Remove import if only have type reference (#2001) (Dunqing)
- 2794064 transfrom: Transform-json-strings (#2168) (underfin)

### Bug Fixes

- 777352e transformer: Always create valid identifiers (#2131) (overlookmotel)

### Refactor

- b261e86 ast: Improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153) (Dunqing)
- ee949fc transformer: Use `is_identifier_part` (overlookmotel)
- 040ee19 transformer: Use `is_identifier_name` from `oxc_syntax` (overlookmotel)
- de6d2f5 transformer/decorators: Optimizing code with ast.private_field (#2249) (Dunqing)
- 51cecbb transformer/decorators: Align the implementation of all versions (#2159) (Dunqing)
- 2e78b91 transformer/typescript: Move the ExportNamedDeclaration logic to its function (#2074) (Dunqing)

## [0.5.0] - 2024-01-12

### Features

- 78b427b transform: Support es2015 new target (#1967) (underfin)
- 6a7e4be transformer: Call enter_node/leave_node in visit_xxx (#1990) (Dunqing)
- afb2c50 transformer: Support for transform TSImportEqualsDeclaration (#1994) (Dunqing)
- ae27a8d transformer: Add partial support for babel-plugin-transform-instanceof (#1802) (秦宇航)
- f58b627 transformer: Add arrow_functions plugin (#1663) (Dunqing)
- e331cc2 transformer: Duplicate keys (#1649) (Ken-HH24)
- 864176a transformer/react-jsx: Returns ThisExpression when identifier is this (#1661) (Dunqing)

### Refactor

- a2858ed ast: Introduce `ThisParameter` (#1728) (magic-akari)

## [0.4.0] - 2023-12-08

### Features

- c6ad660 semantic: Support scope descendents starting from a certain scope. (#1629) (Miles Johnson)
- 92c1d9d transform: TypeScript Enum (#1173) (magic-akari)
- 6cbc5dd transformer: Start on `function_name` transform. (#1510) (Miles Johnson)
- c034eee transformer: Handle invalid react jsx  runtime (#1502) (IWANABETHATGUY)
- f66e4d8 transformer: Add transform property-literal plugin (#1458) (IWANABETHATGUY)
- f0e452a transformer: Support importSource option in react_jsx (#1115) (Dunqing)
- b6393f0 transformer/react: Handle babel 8 breaking removed-options (#1489) (IWANABETHATGUY)
- 7f01d48 transformer/react-jsx: Set `automatic` to the default value for `runtime` (#1270) (Dunqing)
- 1eef241 transformer/react-jsx: Support for throwing SpreadChildrenAreNotSupported error (#1234) (Dunqing)
- 39e6087 transformer/react-jsx: Support for throwing ImportSourceCannotBeSet error (#1224) (Dunqing)
- b7e8feb transformer/react-jsx: Support throw valueless-key error (#1221) (Dunqing)
- a22ced7 transformer/react-jsx: Implement `throwIfNamespace` option (#1220) (Dunqing)
- d9b4504 transformer/react-jsx: When the source type is a script, use require to import the react (#1207) (Dunqing)
- 8c624ab transformer/react-jsx: Throw the `pragma and pragmaFrag cannot be set when runtime is automatic` error (#1196) (Dunqing)
- 7d85492 transformer/react-jsx: Support the `sourceType` is a `script` (#1192) (Dunqing)
- 28c0b85 transformer/react-jsx: Support `@jsxFrag` annotation (#1189) (Dunqing)
- 633c469 transformer/react-jsx: Support `@jsx` annotation (#1182) (Dunqing)
- 3cb7c0b transformer/react-jsx: Support `pragmaFrag` option (#1181) (Dunqing)
- 4ed0813 transformer/react-jsx: Support `pragma` option (#1180) (Dunqing)
- bf23d87 transformer/react-jsx: Support `@jsxImportSource` annotation (#1179) (Dunqing)

### Bug Fixes

- 4824236 transformer/react-jsx: Missing import jsxs in nested fragment (#1218) (Dunqing)
- a0f40cb transformer/react-jsx: Missing default options when plugin without config (#1219) (Dunqing)
- 3e15fa6 transformer/react-jsx: Undetectable comments in multiline comments (#1211) (Dunqing)
- b65094b transformer/react-jsx: No need to wrap the Array when there is only one correct child element (#1205) (Dunqing)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)
- d62631e transformer/react-jsx: Use extend instead of for-in with push (#1236) (Dunqing)
- 47ba874 transformer/react-jsx: Improve SpreadChildrenAreNotSupported error implementation (#1235) (Dunqing)

## [0.3.0] - 2023-11-06

### Features

- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- f60fd65 transfomer: Implement react has_key_after_props_spread (#1075) (Boshen)
- f71cb9f transform: Support TemplateLiteral of babel/plugin-transform-template-literals (#1132) (Wenzhe Wang)
- b5bfc36 transform: Transform jsx element name (#1070) (Wenzhe Wang)
- 09df8e6 transform: Sticky-regex (#968) (Wenzhe Wang)
- ce79bc1 transform_conformance: Move Formatter to codegen (#986) (Boshen)
- 46d2623 transform_conformance: Add jsx and ts tests (Boshen)
- e8a4e81 transformer: Implement some of jsx decode entities (#1086) (Boshen)
- 0856111 transformer: Implement more of react transform attributes (#1081) (Boshen)
- 96332c8 transformer: Import jsxs when children is static (#1080) (Boshen)
- d411258 transformer: Finish transform jsx attribute value (#1078) (Boshen)
- 5fb27fb transformer: Implement key extraction for react automatic (#1077) (Boshen)
- 394ed35 transformer: Implement react get_attribute_name (#1076) (Boshen)
- d6ba891 transformer: Add props `null` to React.createElement (#1074) (Boshen)
- e16e7e4 transformer: Implement react transform attributes (#1071) (Boshen)
- d8f1a7f transformer: Start implementing react jsx transform (#1057) (Boshen)
- 1b64e48 transformer: Strip implicit type import for typescript (#1058) (magic-akari)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- d31a667 transformer: Drop `this` parameter from typescript functions (#1019) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- c060621 transformer: Add unit tests and test coverage (#1001) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)
- dc08c94 transformer: RegexpFlags (#977) (magic-akari)
- 9ad2634 transformer: Class Static Block (#962) (magic-akari)
- 21066a9 transformer: Shorthand Properties (#960) (magic-akari)
- 5973e5a transformer: Setup typescript and react transformers (#930) (Boshen)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)
- f4cea34 transformer: Add babel conformance test suite (#920) (Boshen)
- 419d5aa transformer: Transformer prototype (#918) (Boshen)
- 1051f15 transformer/jsx: Escape xhtml in jsx attributes (#1088) (Boshen)
- 203cf37 transformer/react: Read comment pragma @jsxRuntime classic / automatic (#1133) (Boshen)
- 262631d transformer/react: Implement fixup_whitespace_and_decode_entities (#1091) (Boshen)
- 1b3b100 transformer_conformance: Read plugins options from babel `options.json` (#1006) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)
- fe4a5ed transformer: Fix position of inserted react import statement (#1082) (Boshen)
- 1ad2dca transformer/react_jsx: Add imports to the top body (#1087) (Boshen)

### Refactor

- 4787220 ast: Clean up some methods (Boshen)
- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- 70189f9 ast: Change the arguments order for some `new` functions (Boshen)
- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 052661d transform_conformance: Improve report format (Boshen)
- 69150d8 transformer: Move Semantic into Transformer (#1130) (Boshen)
- c7a04f4 transformer: Remove returning None from transform functions (#1079) (Boshen)
- d9ba532 transformer: Add an empty SPAN utility for creating AST nodes (#1067) (Boshen)
- 46a5c42 transformer: Add TransformerCtx struct for easier access to symbols and scopes (Boshen)
- 1e1050f transformer: Clean up the transformer constructor code (Boshen)

