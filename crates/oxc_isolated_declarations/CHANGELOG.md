# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

### Features

- 9e62396 syntax_operations: Add crate `oxc_syntax_operations` (#6202) (Boshen)

### Bug Fixes

- e9eeae0 isolated-declarations: False positive for function with a type asserted parameters (#6181) (Dunqing)

### Refactor


## [0.30.3] - 2024-09-27

### Bug Fixes

- a8338dd isolated-declarations: Accidentally collected references of original ast (#6102) (Dunqing)

## [0.30.2] - 2024-09-27

### Bug Fixes

- 418ae25 isolated-declarations: Report uninferrable types in arrays (#6084) (michaelm)
- c8682e9 semantic,codegen,transformer: Handle definite `!` operator in variable declarator (#6019) (Boshen)

### Performance

- 6b7d3ed isolated-declarations: Should clone transformed AST rather than original AST (#6078) (Dunqing)

## [0.30.1] - 2024-09-24

### Bug Fixes

- 9ca202a codegen: Preserve newlines between comments (#6014) (Boshen)
- 97a2c41 isolated-declarations: False positive for class private getter with non-inferrable return type (#5987) (michaelm)

## [0.30.0] - 2024-09-23

### Features

- 4a62703 isolated-declarations: Handle `export` in the `namespace` correctly (#5950) (Dunqing)
- 84a5816 isolated_declarations: Add `stripInternal` (#5878) (Boshen)
- dfbde2c isolated_declarations: Print jsdoc comments (#5858) (Boshen)

### Bug Fixes

- 5901d2a codegen: Various spacing issues (#5820) (Boshen)
- fd1c46c isolated-declarations: Infer failed if there are two setter/getter methods that need to be inferred (#5967) (Dunqing)
- 6df82ee isolated-declarations: False positive for class private method that has arguments without type annotations (#5964) (Dunqing)
- 6a9e71d isolated-declarations: Wrap TSFunctionType in parentheses if it is inside the `TSUnionType` (#5963) (Dunqing)
- ea32d5b isolated-declarations: Should print constructor assignments first (#5934) (Dunqing)
- 0f96b59 isolated-declarations: Missing print comments in class's private method (#5931) (Dunqing)
- 8780c54 isolated-declarations: Do not union a undefined when the param type is any or unknown (#5930) (Dunqing)
- f07ff14 isolated-declarations: Should not transform signature that has type annotation (#5927) (Dunqing)
- b6a9178 isolated-declarations: Don't collect references when `ExportNamedDeclaration` has source (#5926) (Dunqing)
- 756a571 isolated-declarations: Missing empty export when has an export declare (#5925) (Dunqing)
- e148c80 isolated_declarations: Try fix fixtures (Boshen)
- 9b3f763 isolated_declarations: Try fix new line issue (Boshen)
- ee748b0 isolated_declarations: Fix fixture spacing (Boshen)

### Performance

- cd34f07 isolated-declarations: Combine type/value bindings and type/value references into one (#5968) (Dunqing)

### Refactor

- c84bd28 isolated-declarations: Simplify to infer the getter and setter methods (#5966) (Dunqing)
- 67b4220 isolated-declarations: Simplify handling VariableDeclaration transform (#5916) (Dunqing)
- 2fd5c2a isolated-declarations: Pre-filter statements that do not need to be transformed (#5909) (Dunqing)
- 1c1353b transformer: Use AstBuilder instead of using struct constructor (#5778) (Boshen)

### Testing

- d6cbbe7 isolated-declarations: Arrow function unions in return signature (#5973) (DonIsaac)

## [0.29.0] - 2024-09-13

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

### Features


### Bug Fixes

- b9bf544 isolated-declarations: False positive for setter method in `interface` (#5681) (Dunqing)
- 6e8409a isolated-declarations: Bindings referenced in `TSModuleDeclaration` are removed incorrectly (#5680) (Dunqing)

### Performance


### Testing
- dc92489 Add trailing line breaks to conformance fixtures (#5541) (overlookmotel)

## [0.26.0] - 2024-09-03

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

### Features

- 180b1a1 ast: Add `Function::name()` (#5361) (DonIsaac)
- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 49cd5db ast,parser: Add `definite` flag to `AccessorProperty` node (#5182) (DonIsaac)
- c2fa725 ast,parser: Parse `TSTypeAnnotations` on `AccessorProperty` (#5179) (DonIsaac)

### Bug Fixes


### Refactor

- 946c867 ast: Box `TSThisParameter` (#5325) (overlookmotel)

## [0.25.0] - 2024-08-23

### Bug Fixes

- 185eb20 isolated_declarations: Namespaces that are default exported should be considered for expando functions (#4935) (michaelm)

## [0.24.3] - 2024-08-18

### Bug Fixes

- d3bbc62 isolated-declarations: Declare modifier of PropertyDefinition should not be retained (#4941) (Dunqing)
- 8e80f59 isolated_declarations: Class properties should still be lifted from private constructors (#4934) (michaelm)
- b3ec9e5 isolated_declarations: Always emit module declarations that perform augmentation (#4919) (michaelm)
- 0fb0b71 isolated_declarations: Always emit module declarations (#4911) (michaelm)
- 4a16916 isolated_declarations: Support expando functions (#4910) (michaelm)

### Refactor

- 1eb59d2 ast, isolated_declarations, transformer: Mark `AstBuilder::copy` as an unsafe function (#4907) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- e12bd1e ast: Allow conversion from TSAccessibility into &'static str (#4711) (DonIsaac)

### Refactor

- 475266d ast: Use correct lifetimes for name-related methods (#4712) (DonIsaac)

## [0.23.0] - 2024-08-01

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

## [0.22.0] - 2024-07-23

### Bug Fixes

- aece1df ast: Visit `Program`s `hashbang` field first (#4368) (overlookmotel)
- 3d88f20 codegen: Print shorthand for all `{ x }` variants (#4374) (Boshen)

### Performance
- a207923 Replace some CompactStr usages with Cows (#4377) (DonIsaac)

### Refactor

- 0e1ea90 isolated-declarations: Remove useless code from scope (#4420) (Dunqing)

## [0.21.0] - 2024-07-18

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- 20cdb1f semantic: Align class scope with typescript (#4195) (Dunqing)

### Bug Fixes

- 3df9e69 mangler: No shorthand `BindingProperty`; handle var hoisting and export variables (#4319) (Boshen)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)
- ace4f1f semantic: Update the order of `visit_function` and `Visit` fields in the builder to be consistent (#4248) (Dunqing)

## [0.20.0] - 2024-07-11

### Features

- 67fe75e ast, ast_codegen: Pass the `scope_id` to the `enter_scope` event. (#4168) (rzvxa)

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)

## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features


### Bug Fixes

- cb1af04 isolated-declarations: Remove the `async` and `generator` keywords from `MethodDefinition` (#4130) (Dunqing)

## [0.17.2] - 2024-07-08

### Bug Fixes

- 5c31236 isolated-declarations: Keep literal value for readonly property (#4106) (Dunqing)
- e67c7d1 isolated-declarations: Do not infer type for private parameters (#4105) (Dunqing)
- 3fcad5e isolated_declarations: Remove nested AssignmentPatterns from inside parameters (#4077) (michaelm)
- f8d77e4 isolated_declarations: Infer type of template literal expressions as string (#4068) (michaelm)

### Performance

- 7ed27b7 isolated-declarations: Use `FxHashSet` instead of `Vec` to speed up the `contain` (#4074) (Dunqing)

## [0.17.1] - 2024-07-06

### Bug Fixes

- adee728 isolated_declarations: Don't report an error for parameters if they are ObjectPattern or ArrayPattern with an explicit type (#4065) (michaelm)
- 1b8f208 isolated_declarations: Correct emit for private static methods (#4064) (michaelm)

### Refactor

- 65aee19 isolated-declarations: Reorganize scope tree (#4070) (Luca Bruno)

## [0.17.0] - 2024-07-05

- c98d8aa ast: [**BREAKING**] Rename `visit_arrow_expression` to `visit_arrow_function_expression`. (#3995) (rzvxa)

### Features

- 7768d23 isolated-declarations: Support optional class methods (#4035) (Egor Blinov)

### Bug Fixes

- 3d29e9c isolated-declarations: Eliminate imports incorrectly when they are used in `TSInferType` (#4043) (Dunqing)
- 02ea19a isolated-declarations: Should emit `export {}` when only having `ImportDeclaration` (#4026) (Dunqing)
- 7c915f4 isolated-declarations: Binding elements with export should report an error (#4025) (Dunqing)
- 05a047c isolated-declarations: Method following an abstract method gets dropped (#4024) (Dunqing)
- c043bec isolated_declarations: Add mapped-type constraint to the scope (#4037) (Egor Blinov)
- b007553 isolated_declarations: Fix readonly specifier on class constructor params (#4030) (Egor Blinov)
- da62839 isolated_declarations: Inferring literal types for readonly class fileds (#4027) (Egor Blinov)

### Refactor


## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)

### Bug Fixes

- bd1141d isolated-declarations: If declarations is referenced in `declare global` then keep it (#3982) (Dunqing)

## [0.16.1] - 2024-06-29

### Bug Fixes

- 51e54f9 codegen: Should print `TSModuleDeclarationKind` instead of just `module` (#3957) (Dunqing)
- 31e4c3b isolated-declarations: `declare global {}` should be kept even if it is not exported (#3956) (Dunqing)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- 1f85f1a ast: [**BREAKING**] Revert adding `span` field to the `BindingPattern` type. (#3899) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 497769c ast: Add some visitor functions (#3785) (Dunqing)
- 2821e0e codegen: Print readonly keyword for TSIndexSignature (#3791) (Dunqing)
- 97575d8 codegen: Print TSClassImplements and TSThisParameter (#3786) (Dunqing)
- 5e2baf3 isolated-declarations: Report error for expando functions (#3872) (Dunqing)
- 2cdb34f isolated-declarations: Support for class function overloads (#3811) (Dunqing)
- 231b8f0 isolated-declarations: Support for export default function overloads (#3809) (Dunqing)
- a37138f isolated-declarations: Improve the inference template literal (#3797) (Dunqing)
- b0d7355 isolated-declarations: Transform const expression correctly (#3793) (Dunqing)
- b38c34d isolated-declarations: Support inferring ParenthesizedExpression (#3769) (Dunqing)
- 4134de8 isolated-declarations: Add ts error code to the error message (#3755) (Dunqing)
- 94202de isolated-declarations: Add `export {}` when needed (#3754) (Dunqing)
- e95d8e3 isolated-declarations: Shrink span for arrow function that needs an explicit return type (#3752) (Dunqing)
- df9971d isolated-declarations: Improve inferring the return type from function (#3750) (Dunqing)
- 4aea2b1 isolated-declarations: Improve inferring the type of accessor (#3749) (Dunqing)
- 9ea30c4 isolated-declarations: Treat AssignmentPattern as optional (#3748) (Dunqing)

### Bug Fixes

- 2766594 codegen: Print type parameters for MethodDefinition (#3922) (Dunqing)
- 27f0531 isolated-declarations: Private constructor reaching unreachable (#3921) (Dunqing)
- 59ce38b isolated-declarations: Inferring of UnrayExpression incorrectly (#3920) (Dunqing)
- 58e54f4 isolated-declarations: Report an error for parameters if they are  ObjectPattern or ArrayPattern without an explicit type (#3810) (Dunqing)
- cb8a272 isolated-declarations: Cannot infer nested `as const` (#3807) (Dunqing)
- d8ecce5 isolated-declarations: Infer BigInt number as `bigint` type (#3806) (Dunqing)
- 4e241fc isolated-declarations: Missing `const` after transformed const enum (#3805) (Dunqing)
- 683c7b0 isolated-declarations: Shouldn’t add declare in declaration with export default (#3804) (Dunqing)
- 7d47fc3 isolated-declarations: Should stripe async and generator keyword after transformed (#3790) (Dunqing)
- 8ce794d isolated-declarations: Inferring an incorrect return type when there is an arrow function inside a function (#3768) (Dunqing)
- d29316a isolated-declarations: Transform incorrectly when there are multiple functions with the same name (#3753) (Dunqing)
- bf1c250 isolated-declarations: False positives for non-exported binding elements (#3751) (Dunqing)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- 2f5d50e isolated-declarations: Remove `Modifiers` (#3847) (Boshen)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

### Features

- ee627c3 isolated-declarations: Create unique name for `_default` (#3730) (Dunqing)
- 81e9526 isolated-declarations: Inferring set accessor parameter type from get accessor return type (#3725) (Dunqing)
- 77d5533 isolated-declarations: Report errors that are consistent with typescript. (#3720) (Dunqing)
- 0b8098a napi: Isolated-declaration (#3718) (Boshen)

### Bug Fixes

- f1b793f isolated-declarations: Function overloads reaching unreachable (#3739) (Dunqing)
- 0fbecdc isolated-declarations: Should be added to references, not bindings (#3726) (Dunqing)

### Refactor

- 3c59735 isolated-declarations: Remove `TransformDtsCtx` (#3719) (Boshen)
- 815260e isolated-declarations: Decouple codegen (#3715) (Boshen)

