# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.15.0] - 2024-06-18

### Features

- d65c652 parser: Display jsx mismatch error, e.g. `<Foo></Bar>` (#3696) (Boshen)

### Bug Fixes

- da1e2d0 codegen: Improve typescript codegen (#3708) (Boshen)

## [0.13.2] - 2024-06-03

### Bug Fixes

- 350cd91 parser: Should parser error when function declaration has no name (#3461) (Dunqing)
- cf41513 parser: Parse const extends in arrow functions correctly (#3450) (Dunqing)
- 6078a6d parser: Fix lexer error while parsing parenthesized arrow expressions (#3400) (Boshen)

## [0.13.1] - 2024-05-22

### Performance

- 27030b9 lexer: Use bitshifting when parsing known integers (#3296) (Don Isaac)
- 508dae6 lexer: Dedupe numeric separator check (#3283) (Don Isaac)
- fdb31c3 parser: More efficient number parsing (#3342) (overlookmotel)
- 46cb5f9 parser: Use `FxHashSet` for `not_parenthesized_arrow` (#3344) (Boshen)

### Refactor

- 6b3d019 paresr: Move some structs to js module (#3341) (Boshen)
- 89a1f97 parser: Improve expression parsing (#3352) (Boshen)
- e818fba parser: Improve `parse_simple_arrow_function_expression` (#3349) (Boshen)
- 1e802c7 parser: Clean up `ParserState` (#3345) (Boshen)
- 0742081 parser: Improve is_parenthesized_arrow_function_expression (#3343) (Boshen)
- 9ced605 parser: Start porting arrow function parsing from tsc (#3340) (Boshen)

## [0.13.0] - 2024-05-14

### Features

- eefb66f ast: Add type to AccessorProperty to support TSAbractAccessorProperty (#3256) (Dunqing)

### Bug Fixes

- c4ccf9f parser: Parse `DecoratorCallExpression` when `Arguments` contains `MemberExpression` (#3265) (Boshen)
- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)

### Performance

- 7338364 lexer: Improve comment building performance by using a vec instead of btreemap (#3186) (Boshen)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 0185eb2 ast: Remove duplicate `TSNamedTupleMember` representation (#3101) (overlookmotel)
- 942b2ba ast: Add array element `Elision` type (#3074) (overlookmotel)
- 312f74b diagnostics: S/OxcDiagnostic::new/OxcDiagnostic::error (Boshen)
- b27a905 parser: Simplify `Context` passing (#3266) (Boshen)
- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)- 1b4ebb3 Run fmt (Boshen)

## [0.12.5] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)

### Bug Fixes

- d44301c parser: Fix comment typos (#3036) (overlookmotel)

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)
- 383b449 ast: Box `ImportDeclarationSpecifier` enum variants (#3061) (overlookmotel)
- 2804e7d ast: Reduce indirection in AST types (#3051) (overlookmotel)

## [0.12.3] - 2024-04-11

### Refactor

- 5974819 ast: Clean up the ts type visit methods (Boshen)

## [0.11.0] - 2024-03-30

### Bug Fixes

- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)
- 798a1fd parser: Fix failed to parse `JSXChild` after `JSXEmptyExpression` (#2726) (Boshen)

### Performance

- e793063 parser: Faster lexing JSX identifiers (#2557) (overlookmotel)

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- 198eea0 ast: Add walk functions to Visit trait. (#2791) (Ali Rezvani)

## [0.10.0] - 2024-03-14

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

- 7768123 parser: [**BREAKING**] Drop TSImportEqualsDeclaration.is_export (#2654) (Arnaud Barré)

### Features
- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)- 265b2fb Miette v7 (#2465) (Boshen)

### Bug Fixes

- 6c6adb4 ast: Parse rest parameter with the correct optional and type annotation syntax (#2686) (Boshen)
- 2a235d3 ast: Parse `with_clause` in re-export declaration (#2634) (magic-akari)
- 86ee074 parser: Remove all duplicated comments in trivia builder (#2689) (Boshen)
- cda9c93 parser: Improve lexing of jsx identifier to fix duplicated comments after jsx name (#2687) (Boshen)
- b378e7e parser: Fix span for JSXEmptyExpression with comment (#2673) (Arnaud Barré)
- 8226031 parser: Fix span start for return type in function type (#2660) (Arnaud Barré)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)
- 240ff19 parser: Improve parsing of `BindingPattern` in TypeScript (#2624) (Boshen)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

- f66059e ast: [**BREAKING**] Align TSImportType with ESTree (#2578) (Arnaud Barré)

### Features

- 20c7bf7 ast: Add `AssignmentTargetRest` (#2601) (Boshen)
- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- 9479865 napi/parser: Expose `preserveParans` option (#2582) (Boshen)
- e2d2ce3 parser: Parse decorators properly (#2603) (Boshen)

### Bug Fixes

- 637cd1d ast: Support TSIndexSignature.readonly (#2579) (Arnaud Barré)
- 258b9b1 ast: Support FormalParameter.override (#2577) (Arnaud Barré)
- 78f30bc ast: Change TSMappedType.type_annotation from TSTypeAnnotation to TSType (#2571) (Arnaud Barré)
- 97aa9cf parser: Fix span end for TSEmptyBodyFunctionExpression (#2606) (Arnaud Barré)
- 9cc960e parser: Fix duplicated comments during parser rewind (#2600) (Boshen)
- 24d46bc parser: Fix span start for TSModuleDeclaration (#2593) (Arnaud Barré)
- ac520d0 parser: Fix span start for TSExportAssignment (#2594) (Arnaud Barré)
- d9cc429 parser: Parse empty method declaration as TSEmptyBodyFunctionExpression (#2574) (Arnaud Barré)
- 32028eb parser: TSConditionalType span start (#2570) (Arnaud Barré)
- 6700810 parser: Set span end for TSEnumDeclaration (#2573) (Arnaud Barré)
- 8a81851 parser: Don't parse null as a literal type (#2572) (Arnaud Barré)

### Performance

- bf42158 parser: Inline `end_span` and `parse_identifier_kind` which are on the hot path (#2612) (Boshen)
- 78f8c2c parser: Lex JSXText with memchr (#2558) (overlookmotel)
- 5a13714 parser: Faster lexing template strings (#2541) (overlookmotel)
- 24ded3c parser: Lex JSX strings with `memchr` (#2528) (overlookmotel)

### Refactor

- dd31c64 parser: `byte_search` macro evaluate to matched byte (#2555) (overlookmotel)
- c579620 parser: Small efficiencies in `byte_search` macro usage (#2554) (overlookmotel)
- 18cff6a parser: Remove start params for `byte_search` macro arms (#2553) (overlookmotel)
- 34ecdd5 parser: Simplify `byte_search` macro (#2552) (overlookmotel)
- ddccaa1 parser: Remove unsafe code in lexer (#2549) (overlookmotel)
- 9d7ea6b parser: Single function for all string slicing (#2540) (overlookmotel)
- 0ddfc85 parser: Remove unsafe code (#2527) (overlookmotel)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- 70295a5 ast: Update arrow_expression to arrow_function_expression (#2496) (Dunqing)
- 7a796c4 ast: Add `TSModuleDeclaration.kind` (#2487) (Boshen)
- 60db720 parser: Parse import attributes in TSImportType (#2436) (Dunqing)
- ef336cb parser: Recover from `async x [newline] => x` (#2375) (Boshen)
- 197fa16 semantic: Add check for duplicate class elements in checker (#2455) (Dunqing)

### Bug Fixes

- 5212f7b parser: Fix missing end span from `TSTypeAliasDeclaration` (#2485) (Boshen)
- 73e116e parser: Incorrect parsing of class accessor property name (#2386) (Dunqing)

### Performance

- 996a9d2 parser: `byte_search` macro always unroll main loop (#2439) (overlookmotel)
- 383f5b3 parser: Consume multi-line comments faster (#2377) (overlookmotel)
- c4fa738 parser: Consume single-line comments faster (#2374) (overlookmotel)
- 0be8397 parser: Optimize lexing strings (#2366) (overlookmotel)

### Refactor

- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- 3cbe786 ast: Update TSImportType parameter to argument (#2429) (Dunqing)
- a78303d parser: `continue_if` in `byte_search` macro not unsafe (#2440) (overlookmotel)
- a5a3c69 parser: Correct comment (#2441) (overlookmotel)
- cc2ddbe parser: Catch all illegal UTF-8 bytes (#2415) (overlookmotel)
- b29719d parser: Add methods to `Source` + `SourcePosition` (#2373) (overlookmotel)
- 79ae9a9 parser: Extend `byte_search` macro (#2372) (overlookmotel)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.7.0] - 2024-02-09

### Features

- a3570d4 semantic: Report parameter related errors for setter/getter (#2316) (Dunqing)

### Bug Fixes

- 2f6cf73 parser: Remove erroneous debug assertion (#2356) (overlookmotel)

### Performance

- c0d1d6b parser: Lex strings as bytes (#2357) (overlookmotel)
- 8376f15 parser: Eat whitespace after line break (#2353) (overlookmotel)
- d3a59f2 parser: Lex identifiers as bytes not chars (#2352) (overlookmotel)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)
- 6910e4f parser: Macro for ASCII identifier byte handlers (#2351) (overlookmotel)
- 6f597b1 parser: All pointer manipulation through `SourcePosition` (#2350) (overlookmotel)
- 185b3db parser: Fix outdated comment (#2344) (overlookmotel)
- f347016 parser: Make `Source::set_position` safe (#2341) (overlookmotel)
- 0bdecb5 parser: Wrapper type for parser (#2339) (overlookmotel)
- cdef41d parser: Lexer replace `Chars` with `Source` (#2288) (overlookmotel)
- 9811c3a parser: Name byte handler functions (#2301) (overlookmotel)

## [0.6.0] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- 165f948 ast: Remove expression property from Function (#2247) (Dunqing)
- 36c718e tasks: Benchmarks for lexer (#2101) (overlookmotel)

### Bug Fixes

- ea8cc98 ast: AcessorProperty is missing decorators (#2176) (Dunqing)
- 2beacd3 lexer: Correct the span for irregular whitespaces (#2245) (Boshen)
- e123be0 parser: Correct MAX_LEN for 32-bit systems (#2204) (overlookmotel)
- 2f5afff parser: Fix crash on TSTemplateLiteralType in function return position (#2089) (Boshen)
- 712e99c parser: Restore regex flag parsing (#2007) (overlookmotel)

### Performance

- 81e33a3 parser: Faster offset calculation (#2215) (overlookmotel)
- 20679d1 parser: Pad `Token` to 16 bytes (#2211) (overlookmotel)
- 66a7a68 parser: Lexer byte handlers consume ASCII chars faster (#2046) (overlookmotel)
- 60a927d parser: Lexer match byte not char (#2025) (overlookmotel)
- 1886a5b parser: Reduce `Token` size from 16 to 12 bytes (#2010) (Boshen)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)
- 622a2c3 lexer: Don't use `lexer.current.chars` directly (#2237) (overlookmotel)
- d0d7082 parser: Consume chars when parsing surrogate pair escape (#2243) (overlookmotel)
- 5279e89 parser: Byte handler for illegal bytes (#2229) (overlookmotel)
- 3d79d77 parser: Split lexer into multiple files (#2228) (overlookmotel)
- 51ac392 parser: Mark `ByteHandler`s unsafe (#2212) (overlookmotel)
- 872d751 parser: Re-order match branches (#2209) (overlookmotel)
- 71898ff parser: Move source length check into lexer (#2206) (overlookmotel)
- bc7ea0b parser: Make `is_identifier` methods consistent (overlookmotel)
- 3f2b48f parser: Remove useless string builder from jsx text lexer (#2096) (Boshen)
- 0e32618 parser: Combine token kinds for skipped tokens (#2072) (overlookmotel)
- 8d5f5b8 parser: Macro for ASCII byte handlers (#2066) (overlookmotel)
- 408acb9 parser: Lexer handle unicode without branch (#2039) (overlookmotel)
- b4d76f0 parser: Remove noop code (#2028) (overlookmotel)
- 6996948 parser: Remove extraneous code from regex parsing (#2008) (overlookmotel)

## [0.5.0] - 2024-01-12

### Features

- c1cfd17 linter: No-irregular-whitespace rule (#1835) (Deivid Almeida)

### Bug Fixes

- b50c5ec parser: Unexpected ts type annotation in get/set (#1942) (Dunqing)
- eb2966c parser: Fix incorrectly identified directives (#1885) (overlookmotel)
- c3090c2 parser: Terminate parsing if an EmptyParenthesizedExpression error occurs (#1874) (Dunqing)
- 62bc8c5 parser: Error on source larger than 4 GiB (#1860) (overlookmotel)
- 2b4d1bf parser: Await in jsx expression (Boshen)
- 19e77b0 parser: False postive for "Missing initializer in const declaration" in declare + namespace (#1724) (Boshen)

### Refactor

- a2858ed ast: Introduce `ThisParameter` (#1728) (magic-akari)
- aa91fde parser: Only allocate for escaped template strings (#2005) (Boshen)
- 38f86b0 parser: Remove string builder from number parsing (#2002) (Boshen)
- c731685 parser: Reduce work parsing regexps (#1999) (overlookmotel)
- 4706765 parser: Reduce `Token` size from 32 to 16 bytes (#1962) (Boshen)
- 6e0bd52 parser: Remove TokenValue::Number from Token (#1945) (Boshen)
- 08438e0 parser: Remove TokenValue::RegExp from `Token` (#1926) (Boshen)
- 7eb2573 parser: Parse BigInt lazily (#1924) (Boshen)
- 5b2696b parser: Report `this` parameter error (#1788) (magic-akari)

## [0.4.0] - 2023-12-08

### Features

- 9ff0ffc ast: Implement new proposal-import-attributes (#1476) (magic-akari)
- 07b0109 parser: Add `preserve_parens` option (default: true) (#1474) (Boshen)
- 1554f7c parsr: Parse `let.a = 1` with error recovery (#1587) (Boshen)
- 567c6ed prettier: Print directives (#1497) (Boshen)
- 0218ae8 prettier: Print leading comments with newlines (#1434) (Boshen)

### Bug Fixes

- a7e0706 parser: Correct `import_kind` of `TSImportEqualsDeclaration` (#1449) (magic-akari)
- 4453529 parser: Fix type import (#1291) (magic-akari)
- 9c0aafc parser: Disallow ReservedWord in NamedExports (#1230) (magic-akari)
- 8afb81a parser: ASI of async class member (#1214) (magic-akari)

### Refactor

- 9842be4 parser: Remove duplicated code (Boshen)
- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 854b55a codegen: Json strings proposal (#1039) (Boshen)
- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- 5b1e1e5 parser: TypeScript 5.2 (#811) (Cameron)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)- 266253c Ts parsing error (#940) (IWANABETHATGUY)

### Refactor

- 4787220 ast: Clean up some methods (Boshen)
- 70189f9 ast: Change the arguments order for some `new` functions (Boshen)
- db5417f clippy: Allow clippy::too_many_lines (Boshen)

## [0.2.0] - 2023-09-14

### Features

- e7c2313 ast: Add `SymbolId` and `ReferenceId` (#755) (Yunfei He)

### Bug Fixes

- 7c8e6ab parser: Parse [+In] in object binding initializer (#874) (Boshen)
- 2f48bdf parser,semantic: Make semantic own `Trivias` (#711) (Boshen)

### Performance

- f447cf3 lexer: Only check the first lower case for `match_keyword` (#913) (Boshen)
- 7962e81 lexer: Remove an extra branch from `identifier_name_handler` (#912) (Boshen)
- d25355c lexer: Reduce an extra branch from peek (#841) (Boshen)
- a272c1f lexer: Reduce checks on ident -> keyword (#783) (Boshen)
- c8a215e lexer: Jump table (#779) (Boshen)
- babbc47 parser: Lazily build trivia map instead of build in-place (#903) (Boshen)
- 1793397 parser: Remove an extra branch from `parse_member_expression_rhs` hot path (#896) (Boshen)

### Refactor

- 3516759 ast: Use `atom` for `Directive` and `Hashbang` (#701) (Yunfei He)
- 56aaf31 benchmark: Use codspeed for all benchmarks (#839) (Boshen)- a2dbfee Clean up fuzzer, move it to repo root (#872) (Boshen)- 12798e0 Improve code coverage a little bit (Boshen)

