# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.14.0] - 2024-06-05

### Bug Fixes

* codegen: print indentation before directive (#3512)

## [0.13.3] - 2024-06-04

### Refactor

* codegen: reduce allocation for print_unquoted_str (#3525)

### Bug Fixes

* codegen: should be double quote for jsx attribute value (#3516)
* codegen: wrong escape string (#3514)

## [0.13.2] - 2024-06-03

### Features

* oxc_codegen: preserve annotate comment (#3465)

## [0.13.1] - 2024-05-22

### Features

* syntax: export `is_reserved_keyword` and `is_global_object` method (#3384)

### Refactor

* parser: start porting arrow function parsing from tsc (#3340)
* sourcemap: using binary search to search original position (#3360)

### Bug Fixes

* codegen: using declaration in for statement (#3285)

## [0.13.0] - 2024-05-14

### Features

* ast: add type to AccessorProperty to support TSAbractAccessorProperty (#3256)

### Bug Fixes

* parser: correctly parse cls.fn<C> = x (#3208)

### Refactor

* ast: squash nested enums (#3115)
* ast: remove duplicate `TSNamedTupleMember` representation (#3101)
* syntax: move number related functions to number module (#3130)

## [0.12.5] - 2024-04-22

### Features

* ast: add `CatchParameter` node (#3049)

## [0.12.4] - 2024-04-19

### Features

* codegen: correctly print type-only imports/exports (#2993)

## [0.12.1] - 2024-04-03

### Refactor

* codegen: make codegen sourcemap builder clearer (#2894)

### Bug Fixes

* sourcemap: using serde_json::to_string to quote sourcemap string (#2889)

## [0.11.0] - 2024-03-30

### Bug Fixes

* codegen: sourcemap token name should be original name (#2843)
* parser: add support for empty module declaration (#2834)

### Features

* transformer: numeric separator plugin. (#2795)- add oxc sourcemap crate (#2825) |- SourcemapVisualizer (#2773) |

### Refactor

* sourcemap: change sourcemap name to take a reference (#2779)

### Performance

* codegen: avoid unnecessary copy (#2727)
* sourcemap: remove unnecessary binary search (#2728)

## [0.10.0] - 2024-03-14

- **BREAKING** ast: rename BigintLiteral to BigIntLiteral (#2659)

### Bug Fixes

* codegen: `CallExpression` sourcemap (#2717)
* parser: parse named rest element in type tuple (#2655)

## [0.9.0] - 2024-03-05

- **BREAKING** ast: align TSImportType with ESTree (#2578)

### Refactor

* codegen: clean up API around building sourcemaps (#2602)

### Features

* ast: add `AssignmentTargetRest` (#2601)
* ast: add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)
* codegen: add sourcemap (#2565)

### Performance

* codegen: speed up generating sourcemap mappings (#2597)
* codegen: speed up building sourcemap line tables (#2591)

### Bug Fixes

* codegen: fix adding mapping to sourcemaps (#2590)
* codegen: correct sourcemaps when Windows line breaks + unicode (#2584)
* codegen: correct sourcemaps when unicode chars (#2583)

## [0.8.0] - 2024-02-26

### Bug Fixes

* codegen: remove redundant semicolon in PropertyDefinition (#2511)
* codegen: when `async` is on the left-hand side of a for-of, wrap it in parentheses (#2407)
* codegen: lower the level of precedence in TaggedTemplateExpression (#2391)

### Refactor

* ast: remove `TSEnumBody` (#2509)
* ast: s/TSThisKeyword/TSThisType to align with estree
* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: s/ArrowExpression/ArrowFunctionExpression to align estree- remove `panic!` from examples (#2454) |

### Features

* Codegen: Improve codegen (#2460)
* codegen: configurable typescript codegen (#2443)

## [0.7.0] - 2024-02-09

### Bug Fixes

* codegen: format new expession + import expression with the correct parentheses (#2346)
* codegen: format new expression + call expression with the correct parentheses (#2330)

### Features

* codegen: avoid printing comma in ArrayAssignmentTarget if the elements is empty (#2331)

### Refactor

* ast: fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Bug Fixes

* codegen: print space before with clause in import (#2278)
* codegen: print necessary spaces for `ExportAllDeclaration` (#2190)
* codegen: print `Directive` original string (#2157)
* codegen: add parenthesis in binary expression by precedence (#2067)

### Features

* codegen: keep shorthand in ObjectPattern and ObjectProperty (#2265)
* codegen: change back to read raw (#2222)
* codegen: print TemplateLiteral with `print_str` (#2207)
* codegen: move string test to codegen (#2150)

### Refactor

* ast: rename RestElement to BindingRestElement (#2116)

## [0.5.0] - 2024-01-12

### Refactor

* formatter,linter,codegen: remove oxc_formatter (#1968)

## [0.4.0] - 2023-12-08

### Features

* ast: implement new proposal-import-attributes (#1476)

### Refactor

* rust: move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Bug Fixes

* ast: jsx attribute value and text child should be jsx string (#1089)
* codegen: fix some typescript codegen problems (#989)
* linter: revert changes to JSX attribute strings (#1101)

### Features

* codegen: indent inner class (#1085)
* codegen: json strings proposal (#1039)
* codegen: beauty class print (#995)
* codegen: implement the basics of non-minifying codegen (#987)
* codegen: move minifying printer to codegen crate (#985)
* codegen: initialize the codegen crate and struct (#983)
* playground: add transform and minify (#993)
* transformer: implement some of jsx decode entities (#1086)
* transformer: implement some of needs_explicit_esm for typescript (#1047)
* transformer: add utils to make logical_assignment_operators pass (#1017)
* transformer: ES2020 Nullish Coalescing Operator (#1004)- support filter exec snap (#1084) |- adjust the order of print semicolon (#1003) |

### Refactor

* minifier: make the minifier api only accept an ast (#990)
* rust: change `RefCell.clone().into_inner()` to `RefCell.get()`

