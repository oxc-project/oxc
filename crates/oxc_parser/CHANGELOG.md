# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.2] - 2024-06-03

### Bug Fixes

* parser: should parser error when function declaration has no name (#3461)
* parser: parse const extends in arrow functions correctly (#3450)
* parser: fix lexer error while parsing parenthesized arrow expressions (#3400)

## [0.13.1] - 2024-05-22

### Performance

* lexer: use bitshifting when parsing known integers (#3296)
* lexer: dedupe numeric separator check (#3283)
* parser: more efficient number parsing (#3342)
* parser: use `FxHashSet` for `not_parenthesized_arrow` (#3344)

### Refactor

* paresr: move some structs to js module (#3341)
* parser: improve expression parsing (#3352)
* parser: improve `parse_simple_arrow_function_expression` (#3349)
* parser: clean up `ParserState` (#3345)
* parser: improve is_parenthesized_arrow_function_expression (#3343)
* parser: start porting arrow function parsing from tsc (#3340)

## [0.13.0] - 2024-05-14

### Refactor

* ast: squash nested enums (#3115)
* ast: remove duplicate `TSNamedTupleMember` representation (#3101)
* ast: add array element `Elision` type (#3074)
* diagnostics: s/OxcDiagnostic::new/OxcDiagnostic::error
* parser: simplify `Context` passing (#3266)
* parser,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3214)
* syntax: move number related functions to number module (#3130)- run fmt |

### Bug Fixes

* parser: parse `DecoratorCallExpression` when `Arguments` contains `MemberExpression` (#3265)
* parser: correctly parse cls.fn<C> = x (#3208)

### Features

* ast: add type to AccessorProperty to support TSAbractAccessorProperty (#3256)

### Performance

* lexer: improve comment building performance by using a vec instead of btreemap (#3186)

## [0.12.5] - 2024-04-22

### Performance

* ast: box typescript enum variants. (#3065)
* ast: box enum variants (#3058)
* ast: box `ImportDeclarationSpecifier` enum variants (#3061)
* ast: reduce indirection in AST types (#3051)

### Features

* ast: add `CatchParameter` node (#3049)

### Bug Fixes

* parser: fix comment typos (#3036)

## [0.12.3] - 2024-04-11

### Refactor

* ast: clean up the ts type visit methods

## [0.11.0] - 2024-03-30

### Bug Fixes

* parser: add support for empty module declaration (#2834)
* parser: fix failed to parse `JSXChild` after `JSXEmptyExpression` (#2726)

### Refactor

* ast: add walk_mut functions (#2776)
* ast: add walk functions to Visit trait. (#2791)

### Performance

* parser: faster lexing JSX identifiers (#2557)

## [0.10.0] - 2024-03-14

- **BREAKING** ast: rename BigintLiteral to BigIntLiteral (#2659)

- **BREAKING** parser: drop TSImportEqualsDeclaration.is_export (#2654)

### Features
- merge features `serde` and `wasm` to `serialize` (#2716) |- miette v7 (#2465) |

### Refactor

* ast: refactor `Trivias` API - have less noise around it (#2692)
* parser: improve parsing of `BindingPattern` in TypeScript (#2624)- rename `CompactString` to `CompactStr` (#2619) |

### Bug Fixes

* ast: parse rest parameter with the correct optional and type annotation syntax (#2686)
* ast: parse `with_clause` in re-export declaration (#2634)
* parser: remove all duplicated comments in trivia builder (#2689)
* parser: improve lexing of jsx identifier to fix duplicated comments after jsx name (#2687)
* parser: fix span for JSXEmptyExpression with comment (#2673)
* parser: fix span start for return type in function type (#2660)
* parser: parse named rest element in type tuple (#2655)

## [0.9.0] - 2024-03-05

- **BREAKING** ast: align TSImportType with ESTree (#2578)

### Performance

* parser: inline `end_span` and `parse_identifier_kind` which are on the hot path (#2612)
* parser: lex JSXText with memchr (#2558)
* parser: faster lexing template strings (#2541)
* parser: lex JSX strings with `memchr` (#2528)

### Bug Fixes

* ast: support TSIndexSignature.readonly (#2579)
* ast: support FormalParameter.override (#2577)
* ast: change TSMappedType.type_annotation from TSTypeAnnotation to TSType (#2571)
* parser: fix span end for TSEmptyBodyFunctionExpression (#2606)
* parser: fix duplicated comments during parser rewind (#2600)
* parser: fix span start for TSModuleDeclaration (#2593)
* parser: fix span start for TSExportAssignment (#2594)
* parser: parse empty method declaration as TSEmptyBodyFunctionExpression (#2574)
* parser: TSConditionalType span start (#2570)
* parser: set span end for TSEnumDeclaration (#2573)
* parser: don't parse null as a literal type (#2572)

### Features

* ast: add `AssignmentTargetRest` (#2601)
* ast: add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)
* napi/parser: expose `preserveParans` option (#2582)
* parser: parse decorators properly (#2603)

### Refactor

* parser: `byte_search` macro evaluate to matched byte (#2555)
* parser: small efficiencies in `byte_search` macro usage (#2554)
* parser: remove start params for `byte_search` macro arms (#2553)
* parser: simplify `byte_search` macro (#2552)
* parser: remove unsafe code in lexer (#2549)
* parser: single function for all string slicing (#2540)
* parser: remove unsafe code (#2527)

## [0.8.0] - 2024-02-26

### Features

* Codegen: Improve codegen (#2460)
* ast: update arrow_expression to arrow_function_expression (#2496)
* ast: add `TSModuleDeclaration.kind` (#2487)
* parser: parse import attributes in TSImportType (#2436)
* parser: recover from `async x [newline] => x` (#2375)
* semantic: add check for duplicate class elements in checker (#2455)

### Bug Fixes

* parser: fix missing end span from `TSTypeAliasDeclaration` (#2485)
* parser: incorrect parsing of class accessor property name (#2386)

### Refactor

* ast: s/TSThisKeyword/TSThisType to align with estree
* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: update TSImportType parameter to argument (#2429)
* parser: `continue_if` in `byte_search` macro not unsafe (#2440)
* parser: correct comment (#2441)
* parser: catch all illegal UTF-8 bytes (#2415)
* parser: add methods to `Source` + `SourcePosition` (#2373)
* parser: extend `byte_search` macro (#2372)- remove `panic!` from examples (#2454) |

### Performance

* parser: `byte_search` macro always unroll main loop (#2439)
* parser: consume multi-line comments faster (#2377)
* parser: consume single-line comments faster (#2374)
* parser: optimize lexing strings (#2366)

## [0.7.0] - 2024-02-09

### Performance

* parser: lex strings as bytes (#2357)
* parser: eat whitespace after line break (#2353)
* parser: lex identifiers as bytes not chars (#2352)

### Bug Fixes

* parser: remove erroneous debug assertion (#2356)

### Refactor

* ast: fix BigInt memory leak by removing it (#2293)
* parser: macro for ASCII identifier byte handlers (#2351)
* parser: all pointer manipulation through `SourcePosition` (#2350)
* parser: fix outdated comment (#2344)
* parser: make `Source::set_position` safe (#2341)
* parser: wrapper type for parser (#2339)
* parser: lexer replace `Chars` with `Source` (#2288)
* parser: name byte handler functions (#2301)

### Features

* semantic: report parameter related errors for setter/getter (#2316)

## [0.6.0] - 2024-02-03

### Features

* ast: remove generator property from ArrowFunction (#2260)
* ast: remove expression property from Function (#2247)
* tasks: benchmarks for lexer (#2101)

### Bug Fixes

* ast: AcessorProperty is missing decorators (#2176)
* lexer: correct the span for irregular whitespaces (#2245)
* parser: correct MAX_LEN for 32-bit systems (#2204)
* parser: fix crash on TSTemplateLiteralType in function return position (#2089)
* parser: restore regex flag parsing (#2007)

### Refactor

* ast: rename RestElement to BindingRestElement (#2116)
* lexer: don't use `lexer.current.chars` directly (#2237)
* parser: consume chars when parsing surrogate pair escape (#2243)
* parser: byte handler for illegal bytes (#2229)
* parser: split lexer into multiple files (#2228)
* parser: mark `ByteHandler`s unsafe (#2212)
* parser: re-order match branches (#2209)
* parser: move source length check into lexer (#2206)
* parser: make `is_identifier` methods consistent
* parser: remove useless string builder from jsx text lexer (#2096)
* parser: combine token kinds for skipped tokens (#2072)
* parser: macro for ASCII byte handlers (#2066)
* parser: lexer handle unicode without branch (#2039)
* parser: remove noop code (#2028)
* parser: remove extraneous code from regex parsing (#2008)

### Performance

* parser: faster offset calculation (#2215)
* parser: pad `Token` to 16 bytes (#2211)
* parser: lexer byte handlers consume ASCII chars faster (#2046)
* parser: lexer match byte not char (#2025)
* parser: reduce `Token` size from 16 to 12 bytes (#2010)

## [0.5.0] - 2024-01-12

### Refactor

* ast: introduce `ThisParameter` (#1728)
* parser: only allocate for escaped template strings (#2005)
* parser: remove string builder from number parsing (#2002)
* parser: reduce work parsing regexps (#1999)
* parser: reduce `Token` size from 32 to 16 bytes (#1962)
* parser: remove TokenValue::Number from Token (#1945)
* parser: remove TokenValue::RegExp from `Token` (#1926)
* parser: parse BigInt lazily (#1924)
* parser: report `this` parameter error (#1788)

### Bug Fixes

* parser: unexpected ts type annotation in get/set (#1942)
* parser: fix incorrectly identified directives (#1885)
* parser: terminate parsing if an EmptyParenthesizedExpression error occurs (#1874)
* parser: error on source larger than 4 GiB (#1860)
* parser: await in jsx expression
* parser: false postive for "Missing initializer in const declaration" in declare + namespace (#1724)

### Features

* linter: no-irregular-whitespace rule (#1835)

## [0.4.0] - 2023-12-08

### Features

* ast: implement new proposal-import-attributes (#1476)
* parser: add `preserve_parens` option (default: true) (#1474)
* parsr: parse `let.a = 1` with error recovery (#1587)
* prettier: print directives (#1497)
* prettier: print leading comments with newlines (#1434)

### Refactor

* parser: remove duplicated code
* rust: move to workspace lint table (#1444)

### Bug Fixes

* parser: correct `import_kind` of `TSImportEqualsDeclaration` (#1449)
* parser: Fix type import (#1291)
* parser: Disallow ReservedWord in NamedExports (#1230)
* parser: ASI of async class member (#1214)

## [0.3.0] - 2023-11-06

### Bug Fixes

* ast: jsx attribute value and text child should be jsx string (#1089)
* linter: revert changes to JSX attribute strings (#1101)- ts parsing error (#940) |

### Features

* codegen: json strings proposal (#1039)
* minifier: partially re-enable minifier (#963)
* parser: TypeScript 5.2 (#811)
* transformer: implement some of needs_explicit_esm for typescript (#1047)

### Refactor

* ast: clean up some methods
* ast: change the arguments order for some `new` functions
* clippy: allow clippy::too_many_lines

## [0.2.0] - 2023-09-14

### Performance

* lexer: only check the first lower case for `match_keyword` (#913)
* lexer: remove an extra branch from `identifier_name_handler` (#912)
* lexer: reduce an extra branch from peek (#841)
* lexer: reduce checks on ident -> keyword (#783)
* lexer: jump table (#779)
* parser: lazily build trivia map instead of build in-place (#903)
* parser: remove an extra branch from `parse_member_expression_rhs` hot path (#896)

### Bug Fixes

* parser: parse [+In] in object binding initializer (#874)
* parser,semantic: make semantic own `Trivias` (#711)

### Refactor

* ast: use `atom` for `Directive` and `Hashbang` (#701)
* benchmark: use codspeed for all benchmarks (#839)- clean up fuzzer, move it to repo root (#872) |- improve code coverage a little bit |

### Features

* ast: add `SymbolId` and `ReferenceId` (#755)

