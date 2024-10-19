# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.32.0] - 2024-10-19

- c0e9d7e codegen: [**BREAKING**] `Codegen::into_source_text` consume `Codegen` (#6539) (overlookmotel)

- 782f0a7 codegen: [**BREAKING**] Rename `print_char` method to `print_ascii_byte` (#6512) (overlookmotel)

- 91c87dd codegen: [**BREAKING**] Remove `Codegen::enableSourceMap` API (#6452) (Boshen)

- 7645e5c codegen: [**BREAKING**] Remove CommentOptions API (#6451) (Boshen)

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

### Features

- e5ed6a5 codegen: Print negative numbers (#6624) (Boshen)

### Bug Fixes

- ba385fc codegen: Panic occurred when printing the comment of the right parenthesis (#6593) (Dunqing)
- 02bfbfe codegen: Preserve parenthesis for `ChainExpression` (#6430) (Dunqing)
- 2ade16e codegen: Invalid codegen when `in` inside bin expr in or loop (#6431) (camc314)
- 6896efc codegen: Fix `in` in sequence expr in for loops (#6428) (camc314)

### Performance

- 77f3a1a codegen: Check last char with byte methods (#6509) (overlookmotel)
- 18b68ff codegen: Optimize `CodeBuffer::print_ascii_byte` (#6516) (overlookmotel)

### Documentation

- 7e909a7 codegen: Fix example for `CodeBuffer::print_ascii_bytes` (#6535) (overlookmotel)
- 235d357 codegen: Improve doc comments for `CodeBuffer` (#6511) (overlookmotel)
- c8fa2eb codegen: Correct and reformat doc comments for `CodeBuffer` (#6504) (overlookmotel)
- 40d1ee4 codegen: Fix and reformat `CodeBuffer` examples (#6499) (overlookmotel)

### Refactor

- 51fc63d codegen: Rename `CodeBuffer::print_bytes_unchecked` method (#6517) (overlookmotel)
- 05a2ebd codegen: Reorder dependencies in `Cargo.toml` (#6514) (overlookmotel)
- e7f3e28 codegen: Rename var in `CodeBuffer` (#6510) (overlookmotel)
- 1bbd383 codegen: Rename `CodeBuffer::print_ascii_bytes` method (#6507) (overlookmotel)
- cd9fe9e codegen: Rename vars in `CodeBuffer` methods (#6506) (overlookmotel)
- fc536a5 codegen: Inline `CodeBuffer` methods (#6501) (overlookmotel)
- 7420620 codegen: Add `CodeBuffer::as_bytes` method (#6498) (overlookmotel)
- 8ae174b codegen: Rename `CodeBuffer::print_byte_unchecked` method (#6496) (overlookmotel)
- 5843e01 codegen: Shorten `CodeBuffer::take_source_text` (#6495) (overlookmotel)
- 951def6 codegen: Clarify safety comments in `CodeBuffer` (#6494) (overlookmotel)
- 84a51ee codegen: Rename vars in `CodeBuffer` (#6493) (overlookmotel)
- 05bd616 codegen: Add line break (#6492) (overlookmotel)
- 204bf55 codegen: Add `CodeBuffer` to fix soundness hole (#6148) (DonIsaac)
- 702b574 codegen: Only print necessary parentheses in TSAsExpression (#6429) (Dunqing)
- f4cdc56 minifier: Use constant folding unary expression from `oxc_ecmascript` (#6647) (Boshen)
- 1a90ec4 rust: Backport v1.82.0 changes to main branch first (#6690) (Boshen)

### Testing

- e7c89a5 codegen: Uncomment passed tests that are related to trailing comments (#6589) (Dunqing)
- d816b0b codegen: Add test for `CodeBuffer::print_byte_unchecked` (#6497) (overlookmotel)

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

### Bug Fixes

- 84b2d07 codegen: Converts line comment to block comment if it is a `PURE` comment (#6356) (Dunqing)

### Refactor


## [0.30.5] - 2024-09-29

### Refactor

- ab187d1 codegen: Restrict visibility of internal methods (#6145) (DonIsaac)

## [0.30.4] - 2024-09-28

### Bug Fixes

- 8582ae3 codegen: Missing parentheses if there is a pure comment before a NewExpression as a ComputedMemberExpression's callee (#6105) (Dunqing)

### Performance

- 05852a0 codegen: Do not check whether there are annotation comments or not if we don't preserve annotation comments (#6107) (Dunqing)

## [0.30.2] - 2024-09-27

### Features

- cca433f codegen: Print `vite` / `webpack` special comments (#6021) (Dunqing)

### Bug Fixes

- c8682e9 semantic,codegen,transformer: Handle definite `!` operator in variable declarator (#6019) (Boshen)

### Refactor

- fe696f0 codegen: Simplify printing annotation comments (#6027) (Dunqing)

## [0.30.1] - 2024-09-24

### Bug Fixes

- 9ca202a codegen: Preserve newlines between comments (#6014) (Boshen)
- 4a99372 codegen: Print jsdoc comments for `TSEnumMember`s (#6007) (camc314)

## [0.30.0] - 2024-09-23

### Features

- d901772 codegen: Implement minify number from terser (#5929) (Boshen)
- 9f6696a codegen: Add new lines to `TSTypeParameterDeclaration` (#5853) (Boshen)
- bcdbba3 codegen: Print jsdoc comments that are attached to statements and class elements (#5845) (Boshen)
- 26386da codegen: Have `with_source_text` reserve memory for code buffer (#5823) (DonIsaac)
- dfbde2c isolated_declarations: Print jsdoc comments (#5858) (Boshen)

### Bug Fixes

- f4aefb5 codegen: Print `let[0]` as `(let)[0]` (#5947) (Boshen)
- cee9d0b codegen: Fix spacing of `for await (x of y)` (#5890) (Boshen)
- 5901d2a codegen: Various spacing issues (#5820) (Boshen)
- 362c427 mangler,codegen: Do not mangle top level symbols (#5965) (Boshen)
- 42dcadf parser: Hashbang comment should not keep the end newline char (#5844) (Boshen)

### Refactor

- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)
- bb95306 codegen: Change annotation comment tests to snapshot (#5800) (Boshen)
- e613a3d codegen: Prepare to add leading comments by adding a template method pattern (#5784) (Boshen)
- 7caae5b codegen: Add `GetSpan` requirement to `Gen` trait (#5772) (Boshen)

## [0.29.0] - 2024-09-13

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

### Features


### Bug Fixes

- d62defb codegen: Do not print trailing commas for `ArrayExpression` (#5551) (Boshen)

### Performance


## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

- 87c5df2 ast: [**BREAKING**] Rename `Expression::without_parentheses` (#5448) (overlookmotel)

### Features

- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)
- c782916 codegen: Print `type_parameters` in `TaggedTemplateExpression` (#5438) (Dunqing)

### Bug Fixes

- 0df1d9d ast, codegen, linter: Panics in fixers. (#5431) (rzvxa)

### Refactor

- d9d7e7c ast: Remove `IdentifierName` from `TSThisParameter` (#5327) (overlookmotel)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

### Features

- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 292d162 codegen: Print missing fields for `AccessorProperty` (#5291) (Dunqing)

### Bug Fixes

- 5c4c001 codegen: Print `export @decorator declare abstract class Foo` correctly (#5303) (Boshen)
- 7b1546b codegen: Do not print comments when `--minify` (Boshen)

### Performance

- 12a7607 codegen: Inline `Codegen::print_list` (#5221) (overlookmotel)
- fb847bd codegen: Slightly faster `print_list` (#5192) (Boshen)

### Refactor

- d4c3778 codegen: Rename vars (#5222) (overlookmotel)
- 543cad6 codegen: Remove some pub APIs (Boshen)

## [0.25.0] - 2024-08-23

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features


### Bug Fixes
- b7db235 Comments gen regression (#5003) (IWANABETHATGUY)

### Refactor

- cd9cf5e oxc: Remove `remove_whitespace` (Boshen)

## [0.24.3] - 2024-08-18

### Features

- d49fb16 oxc_codegen: Support generate range leading comments (#4898) (IWANABETHATGUY)

### Bug Fixes

- bbf9ec0 codegen: Add missing `declare` to `PropertyDefinition` (#4937) (Boshen)
- f210cf7 codegen: Print `TSSatisfiesExpression` and `TSInstantiationExpression` (#4936) (Boshen)
- 21f5762 codegen: Minify large numbers (#4889) (Boshen)
- e8de4bd codegen: Fix whitespace issue when minifying `x in new Error()` (#4886) (Boshen)
- a226962 codegen: Print `TSNonNullExpression` (#4869) (Boshen)
- 3da33d3 codegen: Missing parenthesis for `PrivateInExpression` (#4865) (Boshen)
- 1808529 codegen: Dedupe pure annotation comments (#4862) (IWANABETHATGUY)
- 508644a linter/tree-shaking: Correct the calculation of `>>`, `<<` and `>>>` (#4932) (mysteryven)

## [0.24.0] - 2024-08-08

### Bug Fixes

- 4a56954 codegen: Print raw if value is number is Infinity (#4676) (Boshen)
- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Performance

- 8dd76e4 codegen: Reduce size of `LineOffsetTable` (#4643) (overlookmotel)
- b8e6753 codegen: `u32` indexes in `LineOffsetTable` for source maps (#4641) (overlookmotel)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.1] - 2024-08-06

### Bug Fixes

- 4a56954 codegen: Print raw if value is number is Infinity (#4676) (Boshen)
- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Performance

- 8dd76e4 codegen: Reduce size of `LineOffsetTable` (#4643) (overlookmotel)
- b8e6753 codegen: `u32` indexes in `LineOffsetTable` for source maps (#4641) (overlookmotel)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.0] - 2024-08-01

- 27fd062 sourcemap: [**BREAKING**] Avoid passing `Result`s (#4541) (overlookmotel)

### Features

- a558492 codegen: Implement `BinaryExpressionVisitor` (#4548) (Boshen)
- 7446e98 codegen: Align more esbuild implementations (#4510) (Boshen)
- 35654e6 codegen: Align operator precedence with esbuild (#4509) (Boshen)

### Bug Fixes

- b58ed80 codegen: Enable more test cases (#4585) (Boshen)
- 6a94e3f codegen: Fixes for esbuild test cases (#4503) (Boshen)
- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Performance

- 7585e16 linter: Remove allocations for string comparisons (#4570) (DonIsaac)

### Refactor


## [0.22.0] - 2024-07-23

### Bug Fixes

- 44a10c4 codegen: Object shorthand with parens `({x: (x)})` -> `({ x })` (#4391) (Boshen)
- 3d88f20 codegen: Print shorthand for all `{ x }` variants (#4374) (Boshen)
- e624dff codegen,mangler: Do not print shorthand for `ObjectProperty` (#4350) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- e3e663b mangler: Initialize crate and integrate into minifier (#4197) (Boshen)

### Bug Fixes

- bf3d8d3 codegen: Print annotation comment inside parens for new and call expressions (#4290) (Boshen)
- 084ab76 codegen: Use `ryu-js` for f64 to string (Boshen)
- e167ef7 codegen: Print parenthesis properly (#4245) (Boshen)
- c65198f codegen: Choose the right quote for jsx attribute string (#4236) (Boshen)
- be82c28 codegen: Print `JSXAttributeValue::StringLiteral` directly (#4231) (Boshen)
- 3df9e69 mangler: No shorthand `BindingProperty`; handle var hoisting and export variables (#4319) (Boshen)
- 66b455a oxc_codegen: Avoid print same pure comments multiple time (#4230) (IWANABETHATGUY)- 1c117eb Avoid print extra semicolon after accessor property (#4199) (IWANABETHATGUY)

### Refactor

- d1c4be0 codegen: Clean up annotation_comment (Boshen)
- 06197b8 codegen: Separate tests (Boshen)
- aa22073 codegen: Improve print API (#4196) (Boshen)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Refactor


## [0.18.0] - 2024-07-09

### Features

- 365d9ba oxc_codegen: Generate annotation comments before `CallExpression` and `NewExpression` (#4119) (IWANABETHATGUY)

## [0.17.2] - 2024-07-08

### Bug Fixes

- 5472b7c codegen: 256 indentations level is not enough for codegen (Boshen)

## [0.17.1] - 2024-07-06

### Bug Fixes

- 564a75a codegen: Missing TypeParameters in TSConstructSignature (#4063) (michaelm)

## [0.17.0] - 2024-07-05

### Features

- 7768d23 isolated-declarations: Support optional class methods (#4035) (Egor Blinov)

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)
- 5e5b1b1 codegen: Correct accessibility emit for class formal-parameters/methods/properties (#4042) (Egor Blinov)
- 7844734 codegen: Missing const keyword in TSTypeParamter (#4022) (Dunqing)
- 6254a41 codegen: Missing TypeParamters in TSCallSignature (#4021) (Dunqing)

## [0.16.3] - 2024-07-02

### Bug Fixes

- 23038ad codegen: Print `TSFunctionType` inside `TSTypeAssertion` (#3999) (Boshen)

## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)

### Bug Fixes

- dac617d codegen: Print some missing typescript attributes (#3980) (Boshen)

## [0.16.1] - 2024-06-29

### Bug Fixes

- 51e54f9 codegen: Should print `TSModuleDeclarationKind` instead of just `module` (#3957) (Dunqing)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 5847e16 ast,parser: Add `intrinsic` keyword (#3767) (Boshen)
- 01da2f7 codegen: Print TSThisParameter for TSCallSignatureDeclaration and TSMethodSignature (#3792) (Dunqing)
- 2821e0e codegen: Print readonly keyword for TSIndexSignature (#3791) (Dunqing)
- 97575d8 codegen: Print TSClassImplements and TSThisParameter (#3786) (Dunqing)

### Bug Fixes

- 2766594 codegen: Print type parameters for MethodDefinition (#3922) (Dunqing)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 4cf3c76 parser: Improve parsing of TypeScript types (#3903) (Boshen)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 534242a codegen: [**BREAKING**] Remove `CodegenOptions::enable_typescript` (#3674) (Boshen)

- 0578ece ast: [**BREAKING**] Remove `ExportDefaultDeclarationKind::TSEnumDeclaration` (#3666) (Dunqing)

### Features

- 5a99d30 codegen: Improve codegen formatting (#3735) (Boshen)
- bf9b38a codegen: Improve codegen formatting (#3731) (Boshen)
- 4a004e2 codegen: Print TSImport remaining fields (#3695) (Dunqing)
- a56cb1b codegen: Print accessibility for MethodDefinition (#3690) (Dunqing)
- 38a75e5 coverage: Improve codegen (#3729) (Boshen)
- 4f2db46 transformer-dts: `--isolatedDeclarations` dts transform (#3664) (Dunqing)

### Bug Fixes

- da1e2d0 codegen: Improve typescript codegen (#3708) (Boshen)

### Refactor

- fa7a6ba codegen: Add `gen` method to ast nodes (#3687) (Boshen)
- 09b92b6 codegen: Move `gen_ts` into `gen` to make searching things easier (#3680) (Boshen)
- 815260e isolated-declarations: Decouple codegen (#3715) (Boshen)
- 4f16664 transformer_dts: Create a `Program` for codegen (#3679) (Boshen)

## [0.14.0] - 2024-06-12

### Refactor

- f98f777 linter: Add rule fixer (#3589) (Don Isaac)

## [0.13.4] - 2024-06-07

### Features

- 5c8e16c coverage: Second transformer build does not print typescript (#3561) (Dunqing)

### Bug Fixes

- affb2c8 codegen: Print indentation before directive (#3512) (Dunqing)

## [0.13.3] - 2024-06-04

### Bug Fixes

- 98c9029 codegen: Should be double quote for jsx attribute value (#3516) (Dunqing)
- d8063b6 codegen: Wrong escape string (#3514) (Dunqing)

### Refactor

- ddac2a0 codegen: Reduce allocation for print_unquoted_str (#3525) (Dunqing)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

### Bug Fixes

- a12ed0f codegen: Using declaration in for statement (#3285) (Don Isaac)

### Refactor

- 9ced605 parser: Start porting arrow function parsing from tsc (#3340) (Boshen)
- e879685 sourcemap: Using binary search to search original position (#3360) (underfin)

## [0.13.0] - 2024-05-14

### Features

- eefb66f ast: Add type to AccessorProperty to support TSAbractAccessorProperty (#3256) (Dunqing)

### Bug Fixes

- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 0185eb2 ast: Remove duplicate `TSNamedTupleMember` representation (#3101) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)

## [0.12.5] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)

## [0.12.4] - 2024-04-19

### Features

- fd5002b codegen: Correctly print type-only imports/exports (#2993) (Dunqing)

## [0.12.1] - 2024-04-03

### Bug Fixes

- 28fae2e sourcemap: Using serde_json::to_string to quote sourcemap string (#2889) (underfin)

### Refactor

- 114f68e codegen: Make codegen sourcemap builder clearer (#2894) (underfin)

## [0.11.0] - 2024-03-30

### Features

- 243131d transformer: Numeric separator plugin. (#2795) (Ali Rezvani)- b199cb8 Add oxc sourcemap crate (#2825) (underfin)- a2cfc86 SourcemapVisualizer (#2773) (underfin)

### Bug Fixes

- 6177c2f codegen: Sourcemap token name should be original name (#2843) (underfin)
- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)

### Performance

- 2be5f9d codegen: Avoid unnecessary copy (#2727) (underfin)
- d7004da sourcemap: Remove unnecessary binary search (#2728) (underfin)

### Refactor

- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.10.0] - 2024-03-14

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

### Bug Fixes

- 9609c34 codegen: `CallExpression` sourcemap (#2717) (underfin)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

## [0.9.0] - 2024-03-05

- f66059e ast: [**BREAKING**] Align TSImportType with ESTree (#2578) (Arnaud Barré)

### Features

- 20c7bf7 ast: Add `AssignmentTargetRest` (#2601) (Boshen)
- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- 8bb1084 codegen: Add sourcemap (#2565) (Boshen)

### Bug Fixes

- ea30fd5 codegen: Fix adding mapping to sourcemaps (#2590) (overlookmotel)
- fe29fa4 codegen: Correct sourcemaps when Windows line breaks + unicode (#2584) (overlookmotel)
- 517026b codegen: Correct sourcemaps when unicode chars (#2583) (overlookmotel)

### Performance

- b7f5c63 codegen: Speed up generating sourcemap mappings (#2597) (overlookmotel)
- 42fa8eb codegen: Speed up building sourcemap line tables (#2591) (overlookmotel)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)

### Bug Fixes

- 4327916 codegen: Remove redundant semicolon in PropertyDefinition (#2511) (Dunqing)
- b5deb9a codegen: When `async` is on the left-hand side of a for-of, wrap it in parentheses (#2407) (Dunqing)
- 384d5ac codegen: Lower the level of precedence in TaggedTemplateExpression (#2391) (Wenzhe Wang)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.7.0] - 2024-02-09

### Features

- 55011e2 codegen: Avoid printing comma in ArrayAssignmentTarget if the elements is empty (#2331) (Dunqing)

### Bug Fixes

- 2eb489e codegen: Format new expession + import expression with the correct parentheses (#2346) (Dunqing)
- 721f6cb codegen: Format new expression + call expression with the correct parentheses (#2330) (Boshen)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 8ac0202 codegen: Keep shorthand in ObjectPattern and ObjectProperty (#2265) (Dunqing)
- fa555ce codegen: Change back to read raw (#2222) (Wenzhe Wang)
- 9333264 codegen: Print TemplateLiteral with `print_str` (#2207) (Wenzhe Wang)
- 1ee6d8c codegen: Move string test to codegen (#2150) (Wenzhe Wang)

### Bug Fixes

- 0c225a4 codegen: Print space before with clause in import (#2278) (Wenzhe Wang)
- d34650a codegen: Print necessary spaces for `ExportAllDeclaration` (#2190) (Yunfei He)
- 989ab88 codegen: Print `Directive` original string (#2157) (underfin)
- 29dc5e6 codegen: Add parenthesis in binary expression by precedence (#2067) (Wenzhe Wang)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)

## [0.5.0] - 2024-01-12

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)

## [0.4.0] - 2023-12-08

### Features

- 9ff0ffc ast: Implement new proposal-import-attributes (#1476) (magic-akari)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- cef78ac codegen: Indent inner class (#1085) (Wenzhe Wang)
- 854b55a codegen: Json strings proposal (#1039) (Boshen)
- 6c18b3e codegen: Beauty class print (#995) (Wenzhe Wang)
- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 809f050 codegen: Move minifying printer to codegen crate (#985) (Boshen)
- f28d96c codegen: Initialize the codegen crate and struct (#983) (Boshen)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- e8a4e81 transformer: Implement some of jsx decode entities (#1086) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)- 094dfa5 Support filter exec snap (#1084) (Wenzhe Wang)- 0e91044 Adjust the order of print semicolon (#1003) (Wenzhe Wang)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- f32bf27 codegen: Fix some typescript codegen problems (#989) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)

### Refactor

- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 110059f rust: Change `RefCell.clone().into_inner()` to `RefCell.get()` (Boshen)

