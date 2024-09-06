# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

