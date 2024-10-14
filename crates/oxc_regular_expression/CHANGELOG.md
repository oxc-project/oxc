# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

### Refactor

- acab777 regular_expression: Misc fixes (#6234) (leaysgur)

## [0.30.2] - 2024-09-27

### Features

- 8d026e1 regular_expression: Implement `GetSpan` for RegExp AST nodes (#6056) (camchenry)
- 7764793 regular_expression: Implement visitor pattern trait for regex AST (#6055) (camchenry)

## [0.28.0] - 2024-09-11

### Bug Fixes

- 304ce25 regular_expression: Keep LegacyOctalEscape raw digits for `to_string` (#5692) (leaysgur)
- 0511d55 regular_expression: Report more MayContainStrings error in (nested)class (#5661) (leaysgur)
- 41582ea regular_expression: Improve RegExp `to_string()` results (#5635) (leaysgur)
- 28aad28 regular_expression: Handle `-` in `/[\-]/u` as escaped character (#5631) (leaysgur)

### Refactor

- 0ac420d linter: Use meaningful names for diagnostic parameters (#5564) (Don Isaac)
- 2da42ef regular_expression: Improve AST docs with refactoring may_contain_strings (#5665) (leaysgur)
- dec1395 regular_expression: Align diagnostics (#5543) (leaysgur)

## [0.27.0] - 2024-09-06

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)
- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)

### Bug Fixes

- 9b984b3 regex: Panic on displaying surrogated `UnicodeEscape` characters. (#5469) (rzvxa)
- 88b7ddb regular_expression: Handle unterminated character class (#5523) (leaysgur)

### Refactor

- ccc8a27 ast, ast_tools: Use full method path for generated derives trait calls. (#5462) (rzvxa)
- e7bd49d regular_expression: Correct typo (#5429) (overlookmotel)

## [0.26.0] - 2024-09-03

### Features

- 46b641b regular_expression: Validate max quantifier value (#5218) (leaysgur)

### Bug Fixes

- cffce11 regular_expression: Prevent panic on too large number (#5282) (leaysgur)

