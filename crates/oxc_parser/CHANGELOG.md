# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.0] - 2024-03-30

### Bug Fixes

- Add support for empty module declaration (#2834)
- Fix failed to parse `JSXChild` after `JSXEmptyExpression` (#2726)

### Performance

- Faster lexing JSX identifiers (#2557)

### Refactor

- Add walk_mut functions (#2776)
- Add walk functions to Visit trait. (#2791)

## [0.10.0] - 2024-03-14

### Features

- Merge features `serde` and `wasm` to `serialize` (#2716)
- Miette v7 (#2465)

### Bug Fixes

- Remove all duplicated comments in trivia builder (#2689)
- Improve lexing of jsx identifier to fix duplicated comments after jsx name (#2687)
- Parse rest parameter with the correct optional and type annotation syntax (#2686)
- Fix span for JSXEmptyExpression with comment (#2673)
- Fix span start for return type in function type (#2660)
- Rename BigintLiteral to BigIntLiteral (#2659)
- Parse named rest element in type tuple (#2655)
- Drop TSImportEqualsDeclaration.is_export (#2654)
- Parse `with_clause` in re-export declaration (#2634)

### Refactor

- Refactor `Trivias` API - have less noise around it (#2692)
- Improve parsing of `BindingPattern` in TypeScript (#2624)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Features

- Parse decorators properly (#2603)
- Add `AssignmentTargetRest` (#2601)
- Expose `preserveParans` option (#2582)
- Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)

### Bug Fixes

- Fix span end for TSEmptyBodyFunctionExpression (#2606)
- Fix duplicated comments during parser rewind (#2600)
- Fix span start for TSModuleDeclaration (#2593)
- Align TSImportType with ESTree (#2578)
- Fix span start for TSExportAssignment (#2594)
- Parse empty method declaration as TSEmptyBodyFunctionExpression (#2574)
- Support TSIndexSignature.readonly (#2579)
- Support FormalParameter.override (#2577)
- Change TSMappedType.type_annotation from TSTypeAnnotation to TSType (#2571)
- TSConditionalType span start (#2570)
- Set span end for TSEnumDeclaration (#2573)
- Don't parse null as a literal type (#2572)

### Performance

- Inline `end_span` and `parse_identifier_kind` which are on the hot path (#2612)
- Lex JSXText with memchr (#2558)
- Faster lexing template strings (#2541)
- Lex JSX strings with `memchr` (#2528)

### Refactor

- `byte_search` macro evaluate to matched byte (#2555)
- Small efficiencies in `byte_search` macro usage (#2554)
- Remove start params for `byte_search` macro arms (#2553)
- Simplify `byte_search` macro (#2552)
- Remove unsafe code in lexer (#2549)
- Single function for all string slicing (#2540)
- Remove unsafe code (#2527)

## [0.8.0] - 2024-02-26

### Features

- Update arrow_expression to arrow_function_expression (#2496)
- Add `TSModuleDeclaration.kind` (#2487)
- Improve codegen (#2460)
- Add check for duplicate class elements in checker (#2455)
- Parse import attributes in TSImportType (#2436)
- Recover from `async x [newline] => x` (#2375)

### Bug Fixes

- Fix missing end span from `TSTypeAliasDeclaration` (#2485)
- Incorrect parsing of class accessor property name (#2386)

### Performance

- `byte_search` macro always unroll main loop (#2439)
- Consume multi-line comments faster (#2377)
- Consume single-line comments faster (#2374)
- Optimize lexing strings (#2366)

### Refactor

- S/TSThisKeyword/TSThisType to align with estree
- S/NumberLiteral/NumericLiteral to align with estree
- Remove `panic!` from examples (#2454)
- `continue_if` in `byte_search` macro not unsafe (#2440)
- Correct comment (#2441)
- Update TSImportType parameter to argument (#2429)
- Catch all illegal UTF-8 bytes (#2415)
- Add methods to `Source` + `SourcePosition` (#2373)
- Extend `byte_search` macro (#2372)

## [0.7.0] - 2024-02-09

### Features

- Report parameter related errors for setter/getter (#2316)

### Bug Fixes

- Remove erroneous debug assertion (#2356)

### Performance

- Lex strings as bytes (#2357)
- Eat whitespace after line break (#2353)
- Lex identifiers as bytes not chars (#2352)

### Refactor

- Macro for ASCII identifier byte handlers (#2351)
- All pointer manipulation through `SourcePosition` (#2350)
- Fix outdated comment (#2344)
- Make `Source::set_position` safe (#2341)
- Wrapper type for parser (#2339)
- Lexer replace `Chars` with `Source` (#2288)
- Name byte handler functions (#2301)
- Fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Features

- Remove generator property from ArrowFunction (#2260)
- Remove expression property from Function (#2247)
- Benchmarks for lexer (#2101)

### Bug Fixes

- Correct the span for irregular whitespaces (#2245)
- Correct MAX_LEN for 32-bit systems (#2204)
- AcessorProperty is missing decorators (#2176)
- Fix crash on TSTemplateLiteralType in function return position (#2089)
- Restore regex flag parsing (#2007)

### Performance

- Faster offset calculation (#2215)
- Pad `Token` to 16 bytes (#2211)
- Lexer byte handlers consume ASCII chars faster (#2046)
- Lexer match byte not char (#2025)
- Reduce `Token` size from 16 to 12 bytes (#2010)

### Refactor

- Consume chars when parsing surrogate pair escape (#2243)
- Don't use `lexer.current.chars` directly (#2237)
- Byte handler for illegal bytes (#2229)
- Split lexer into multiple files (#2228)
- Mark `ByteHandler`s unsafe (#2212)
- Re-order match branches (#2209)
- Move source length check into lexer (#2206)
- Make `is_identifier` methods consistent
- Rename RestElement to BindingRestElement (#2116)
- Remove useless string builder from jsx text lexer (#2096)
- Combine token kinds for skipped tokens (#2072)
- Macro for ASCII byte handlers (#2066)
- Lexer handle unicode without branch (#2039)
- Remove noop code (#2028)
- Remove extraneous code from regex parsing (#2008)

## [0.5.0] - 2024-01-12

### Features

- No-irregular-whitespace rule (#1835)

### Bug Fixes

- Unexpected ts type annotation in get/set (#1942)
- Fix incorrectly identified directives (#1885)
- Terminate parsing if an EmptyParenthesizedExpression error occurs (#1874)
- Error on source larger than 4 GiB (#1860)
- Await in jsx expression
- False postive for "Missing initializer in const declaration" in declare + namespace (#1724)

### Refactor

- Only allocate for escaped template strings (#2005)
- Remove string builder from number parsing (#2002)
- Reduce work parsing regexps (#1999)
- Reduce `Token` size from 32 to 16 bytes (#1962)
- Remove TokenValue::Number from Token (#1945)
- Remove TokenValue::RegExp from `Token` (#1926)
- Parse BigInt lazily (#1924)
- Report `this` parameter error (#1788)
- Introduce `ThisParameter` (#1728)

## [0.4.0] - 2023-12-08

### Features

- Parse `let.a = 1` with error recovery (#1587)
- Implement new proposal-import-attributes (#1476)
- Print directives (#1497)
- Add `preserve_parens` option (default: true) (#1474)
- Print leading comments with newlines (#1434)

### Bug Fixes

- Correct `import_kind` of `TSImportEqualsDeclaration` (#1449)
- Fix type import (#1291)
- Disallow ReservedWord in NamedExports (#1230)
- ASI of async class member (#1214)

### Refactor

- Remove duplicated code
- Move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

- Implement some of needs_explicit_esm for typescript (#1047)
- Json strings proposal (#1039)
- Partially re-enable minifier (#963)
- TypeScript 5.2 (#811)

### Bug Fixes

- Revert changes to JSX attribute strings (#1101)
- Jsx attribute value and text child should be jsx string (#1089)
- Ts parsing error (#940)

### Refactor

- Allow clippy::too_many_lines
- Clean up some methods
- Change the arguments order for some `new` functions

## [0.2.0] - 2023-09-14

### Features

- Add `SymbolId` and `ReferenceId` (#755)

### Bug Fixes

- Parse [+In] in object binding initializer (#874)
- Make semantic own `Trivias` (#711)

### Performance

- Only check the first lower case for `match_keyword` (#913)
- Remove an extra branch from `identifier_name_handler` (#912)
- Lazily build trivia map instead of build in-place (#903)
- Remove an extra branch from `parse_member_expression_rhs` hot path (#896)
- Reduce an extra branch from peek (#841)
- Reduce checks on ident -> keyword (#783)
- Jump table (#779)

### Refactor

- Clean up fuzzer, move it to repo root (#872)
- Use codspeed for all benchmarks (#839)
- Improve code coverage a little bit
- Use `atom` for `Directive` and `Hashbang` (#701)

