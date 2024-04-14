# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.2] - 2024-04-03

### Bug Fixes

- avoid unsafe code search original name (#2895)

## [0.11.1] - 2024-04-03

### Bug Fixes

- Using serde_json::to_string to quote sourcemap string (#2889)

### Refactor

- Make codegen sourcemap builder clearer (#2894)

## [0.11.0] - 2024-03-30

### Features

- Add oxc sourcemap crate (#2825)
- Numeric separator plugin. (#2795)
- SourcemapVisualizer (#2773)

### Bug Fixes

- Sourcemap token name should be original name (#2843)
- Add support for empty module declaration (#2834)

### Performance

- Remove unnecessary binary search (#2728)
- Avoid unnecessary copy (#2727)

### Refactor

- Change sourcemap name to take a reference (#2779)

## [0.10.0] - 2024-03-14

### Bug Fixes

- `CallExpression` sourcemap (#2717)
- Rename BigintLiteral to BigIntLiteral (#2659)
- Parse named rest element in type tuple (#2655)

## [0.9.0] - 2024-03-05

### Features

- Add `AssignmentTargetRest` (#2601)
- Add sourcemap (#2565)
- Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)

### Bug Fixes

- Fix adding mapping to sourcemaps (#2590)
- Align TSImportType with ESTree (#2578)
- Correct sourcemaps when Windows line breaks + unicode (#2584)
- Correct sourcemaps when unicode chars (#2583)

### Performance

- Speed up generating sourcemap mappings (#2597)
- Speed up building sourcemap line tables (#2591)

### Refactor

- Clean up API around building sourcemaps (#2602)

## [0.8.0] - 2024-02-26

### Features

- Improve codegen (#2460)
- Configurable typescript codegen (#2443)

### Bug Fixes

- Remove redundant semicolon in PropertyDefinition (#2511)
- When `async` is on the left-hand side of a for-of, wrap it in parentheses (#2407)
- Lower the level of precedence in TaggedTemplateExpression (#2391)

### Refactor

- Remove `TSEnumBody` (#2509)
- S/TSThisKeyword/TSThisType to align with estree
- S/NumberLiteral/NumericLiteral to align with estree
- S/ArrowExpression/ArrowFunctionExpression to align estree
- Remove `panic!` from examples (#2454)

## [0.7.0] - 2024-02-09

### Features

- Avoid printing comma in ArrayAssignmentTarget if the elements is empty (#2331)

### Bug Fixes

- Format new expession + import expression with the correct parentheses (#2346)
- Format new expression + call expression with the correct parentheses (#2330)

### Refactor

- Fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Features

- Keep shorthand in ObjectPattern and ObjectProperty (#2265)
- Change back to read raw (#2222)
- Print TemplateLiteral with `print_str` (#2207)
- Move string test to codegen (#2150)

### Bug Fixes

- Print space before with clause in import (#2278)
- Print necessary spaces for `ExportAllDeclaration` (#2190)
- Print `Directive` original string (#2157)
- Add parenthesis in binary expression by precedence (#2067)

### Refactor

- Rename RestElement to BindingRestElement (#2116)

## [0.5.0] - 2024-01-12

### Refactor

- Remove oxc_formatter (#1968)

## [0.4.0] - 2023-12-08

### Features

- Implement new proposal-import-attributes (#1476)

### Refactor

- Move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

- Indent inner class (#1085)
- Implement some of jsx decode entities (#1086)
- Support filter exec snap (#1084)
- Implement some of needs_explicit_esm for typescript (#1047)
- Json strings proposal (#1039)
- Add utils to make logical_assignment_operators pass (#1017)
- ES2020 Nullish Coalescing Operator (#1004)
- Adjust the order of print semicolon (#1003)
- Beauty class print (#995)
- Add transform and minify (#993)
- Implement the basics of non-minifying codegen (#987)
- Move minifying printer to codegen crate (#985)
- Initialize the codegen crate and struct (#983)

### Bug Fixes

- Revert changes to JSX attribute strings (#1101)
- Jsx attribute value and text child should be jsx string (#1089)
- Fix some typescript codegen problems (#989)

### Refactor

- Change `RefCell.clone().into_inner()` to `RefCell.get()`
- Make the minifier api only accept an ast (#990)

