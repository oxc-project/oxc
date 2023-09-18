# Transformer

Legend:
* [Syntax] means this is a syntax plugin, no code is required because it is supported by the parser.
* [Regex] means this is a regex transform, which is not supported.
* [Codegen] means the code generator is responsible for the feature.

## Target

From [@babel/preset-env](https://babel.dev/docs/babel-preset-env).

### ES2024

- [ ] [Regex] unicode-sets-regex

### ES2023

Does not have any features.

### ES2022

- [ ] class-properties
- [ ] class-static-block
- [ ] private-methods
- [ ] private-property-in-object
- [x] [Syntax] syntax-top-level-await

### ES2021

- [ ] logical-assignment-operators
- [x] [Syntax] numeric-separator

### ES2020

- [ ] dynamic-import
- [ ] export-namespace-from
- [ ] nullish-coalescing-operator
- [ ] optional-chaining
- [x] [Syntax] syntax-bigint
- [x] [Syntax] syntax-dynamic-import
- [x] [Syntax] syntax-import-meta

### ES2019

- [x] optional-catch-binding
- [ ] [Codegen] json-strings

### ES2018

- [ ] async-generator-functions
- [ ] object-rest-spread
- [ ] [Regex] unicode-property-regex
- [ ] [Regex] dotall-regex
- [ ] [Regex] named-capturing-groups-regex

### ES2017

- [ ] async-to-generator

### ES2016

- [ ] exponentiation-operator

### ES2015

- [ ] arrow-functions
- [ ] block-scoping
- [ ] classes
- [ ] computed-properties
- [ ] destructuring
- [ ] duplicate-keys
- [ ] for-of
- [ ] function-name
- [ ] instanceof
- [ ] literals
- [ ] new-target
- [ ] object-super
- [ ] parameters
- [ ] shorthand-properties
- [ ] spread
- [ ] sticky-regex
- [ ] template-literals
- [ ] typeof-symbol
- [ ] [Regex] unicode-escapes
- [ ] [Regex] unicode-regex
