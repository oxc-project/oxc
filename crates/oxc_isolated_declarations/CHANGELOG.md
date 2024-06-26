# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

