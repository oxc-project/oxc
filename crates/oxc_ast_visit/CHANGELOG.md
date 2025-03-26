# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features


## [0.59.0] - 2025-03-18

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Bug Fixes


## [0.58.0] - 2025-03-13

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

- 48a0394 ast: [**BREAKING**] Add `scope_id` to `TSFunctionType` (#9559) (camc314)

### Features


### Bug Fixes

- bbb4f98 semantic: Insert binding into correct scope for TSInferType (#9567) (camc314)

## [0.55.0] - 2025-03-05

### Features

- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

## [0.54.0] - 2025-03-04

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)

