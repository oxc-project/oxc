# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).


## [0.88.0] - 2025-09-15

### 🚀 Features

- db33196 parser: Adds typescript rule for empty argument list (#13730) (Karan Kiri)

### 🐛 Bug Fixes

- f795d69 parser: Improve diagnostics around modifier checks (#13526) (Ulrich Stark)


## [0.87.0] - 2025-09-08

### 🐛 Bug Fixes

- 34d3cde rust: Fix clippy issues (#13540) (Boshen)
- 65aca9e parser: Reset popped lexer errors when rewinding (#13494) (Ulrich Stark)

### ⚡ Performance

- 3ead0dd parser: Store Option<diagnostic> for lexer errors (#13520) (camc314)





## [0.83.0] - 2025-08-29

### 🚀 Features

- 903a150 parser: Report more invalid modifier locations (#13368) (Ulrich Stark)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- ade2ccb parser: Produce syntax error for `export enum` and similar ts syntaxes (#13208) (Boshen)

### 🚜 Refactor

- b2d59a2 parser: Improve safety of char to bytes conversions (#13193) (overlookmotel)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- fdfec21 lexer: Simplify byte handler macros (#13057) (overlookmotel)

### 📚 Documentation

- 56ae824 lexer: Update comment to match code (#13103) (overlookmotel)



## [0.82.0] - 2025-08-12

### 💥 BREAKING CHANGES

- 128b527 data_structures: [**BREAKING**] Remove `PointerExt` trait (#12903) (overlookmotel)

### 🚜 Refactor

- 51aaafd rust: Enable `unnecessary_unwrap` lint (#12908) (camc314)

### ⚡ Performance

- 47a565f lexer: Only check for hashbang at start of file (#12521) (overlookmotel)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)

### 🚜 Refactor

- febb4fa parser: Add `StatementContext::TopLevelStatementList` (#12806) (overlookmotel)

### ⚡ Performance

- 373b5b7 lexer: Add `#[cold]` to unicode path (#12768) (copilot-swe-agent)
- ae0137c lexer: Improve byte_handlers for `!` and `?` (#12831) (Boshen)
- 5d96425 parser: Register `import` / `export` statements in module record directly (#12807) (overlookmotel)
- 00bdfc0 parser: Remove a bound check in `match_keyword` (#12778) (Boshen)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)
- 7332ae4 ast: [**BREAKING**] Box `rest` fields of `ArrayAssignmentTarget` and `ObjectAssignmentTarget` (#12698) (Copilot)

### 🐛 Bug Fixes

- e836e55 parser: Prevent panic when parsing invalid extends clause (#12551) (Cameron)
- ce5876d parser: Validate inner expression of type assertions in assignment targets (#12614) (camc314)

### 🚜 Refactor

- 4fc0868 parser: Reduce unnecessary backtracking in hot paths (#12708) (Copilot)
- 8a27974 parser: Shorten `AstBuilder` calls (#12716) (overlookmotel)

### 📚 Documentation

- 514322c rust: Add minimal documentation to example files in crates directory (#12731) (Copilot)
- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)



## [0.79.0] - 2025-07-30

### 🎨 Styling

- 977d3ba lexer: Reformat `Kind` matchers (#12520) (overlookmotel)


## [0.78.0] - 2025-07-24

### 🚀 Features

- c135beb codegen: Keep function expression PIFEs (#12470) (sapphi-red)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)



## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- 4d88252 parser: Remove unnecessary `unbox` (#12302) (overlookmotel)
- 1058e8a parser: Shorten code (#12301) (overlookmotel)


## [0.77.0] - 2025-07-12

### 🐛 Bug Fixes

- a46708f parser: Handle `%` token as a v8_intrinsic only if option is enabled (#12128) (leaysgur)


## [0.76.0] - 2025-07-08

### ⚡ Performance

- 349d395 parser: Speed up simple lookaheads by introducing `Lexer::peek_token` (#11358) (Ulrich Stark)
- 494c29d parser: Optimize around `parse_return_type` (#12095) (Ulrich Stark)


## [0.75.1] - 2025-07-03

### 🚀 Features

- b446a66 parser: Report duplicate `private` / `protected` / `public` modifier (#11996) (Boshen)

### 🐛 Bug Fixes

- 6c9c580 parser: Panic when parsing interface with missing implements (#11898) (camc314)


## [0.75.0] - 2025-06-25

### 💥 BREAKING CHANGES

- 9a2548a napi/parser: [**BREAKING**] Add `range` option (#11728) (Bacary Bruno Bodian)

### 🐛 Bug Fixes

- 066c4c4 parser: Do not produce AST for incorrect rest parameter position (#11894) (Boshen)


## [0.74.0] - 2025-06-23

### 🚜 Refactor

- edb47e5 parser: Simplify `parse_import_or_export_specifier` (#11847) (Ulrich Stark)
- 7d31600 parser: Avoid unnecessary referencing (#11846) (Ulrich Stark)

### ⚡ Performance

- a8e4f01 parser: Avoid redundant Kind checks when parsing for loops (#11799) (Ulrich Stark)


## [0.73.2] - 2025-06-18

### 🚀 Features

- 8c341a2 sema/check: Ts setters cannot have initializers (#11695) (Don Isaac)


## [0.73.1] - 2025-06-17

### 🚀 Features

- e05d9bb parser: Introduce `ParserImpl::token_source` method (#11737) (overlookmotel)
- 563684a parser: Emit diagnostic for modifiers on static block (#11727) (Ulrich Stark)
- 8fb53b6 parser: Forbid `declare` on class getter and setter (#11717) (Boshen)

### 🐛 Bug Fixes

- 854b0f1 parser: Allocate all strings in arena (#11738) (overlookmotel)

### 🚜 Refactor

- b0a1561 parser: Move empty ts type parameter checks to parser (#11696) (Don Isaac)
- cf35cfd parser: Shorten Span construction (#11685) (Ulrich Stark)

### ⚡ Performance

- 2f25ca6 parser: Optimize code around parsing delimited list and object (#11755) (Ulrich Stark)


## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- e0ae6b2 parser: Produce syntax error for decorator in class methods in js (#11624) (Boshen)
- cdf7cdc parser: Produce syntax error for decorators in non-class methods (#11614) (Boshen)
- 3b03fd3 parser: Produce correct syntax error for `interface I extends (typeof T)` (#11610) (Boshen)
- ab3284a parser: Produce syntax error for `interface A implements B {}` (#11608) (Boshen)
- 844a8a8 parser: Produce syntax error for `declare function foo() {}` (#11606) (Boshen)
- 387c7f6 parser: Add better debug impl for `Token` (#11541) (camc314)

### 🐛 Bug Fixes

- 4e40089 parser: Parse `TSTypePredicate` correctly (#11666) (Boshen)
- eb55d83 parser: Parse `using()` correctly (#11664) (Boshen)
- 7266200 parser: Parse `@x() @y() export default abstract class {}` (#11630) (Boshen)
- 40ca1ef parser: Don't parse a single "webpack" word as a webpack magic comment (#11626) (Boshen)
- e4804ba parser: Parse decorator on `abstract class` (#11625) (Boshen)
- cb17dae parser: Report error on malformed template expressions (#11540) (camc314)
- 069c2b4 parser: Correct `TemplateTail::to_str` from `$}` to `}` (#11539) (camc314)
- 551cd2a parser: Fix parsing of `import source` and `import defer` (#11537) (camchenry)

### 🚜 Refactor

- e519176 parser: Remove rewind in hot path for parsing `?.something` and `?.[` (#11643) (camchenry)
- 4140bb8 parser: Remove rewind in hot path for parsing `for (let` (#11623) (camchenry)
- 40b3a0e parser: Reduce rewind in checking if start of function type or constructor type (#11622) (camchenry)
- 850543b parser: Remove lookahead in parsing intrinsic keyword (#11621) (camchenry)
- b7b0dc3 parser: Improve `TSModuleDeclaration` parsing (#11605) (Boshen)
- e9a8832 parser: Rewrite decorator parsing (#11604) (Boshen)
- b2bd741 parser: Speed up and migrate ts errors for parsing ts tuple elements (#11580) (Ulrich Stark)
- 4130b41 parser: Store export entries in arena (#11567) (camchenry)

### ⚡ Performance

- b34c6f6 parser,semantic: Improve handling of diagnostics (#11641) (Boshen)
- 78f1336 parser: Remove lookahead for checking for-let-of and for-async-of (#11655) (camchenry)
- e389748 parser: Add early returns when eating modifiers before decorators (#11653) (camchenry)
- f224585 parser: Improve perf of parse_template_lit (#11542) (camc314)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Features

- d2da854 parser: Produce syntax error for `satisfies` and `as` in js files (#11502) (Boshen)
- cd063d1 parser: Produce syntax error for decorators on overload (#11496) (Boshen)
- 458c372 parser: Produce syntax error for decorators in incorrect places (#11491) (Boshen)
- 7e88451 parser: Syntax errors for decorators appearing in conflicting places (#11482) (Boshen)

### Bug Fixes

- 392752f parser: Handle `import {type as as}` correctly (#11488) (camchenry)
- be3bd8c parser: Fix panic while parsing `async await => {}` in module (#11493) (Boshen)
- e291191 parser: Fix panic when parsing `export import` (#11473) (Boshen)
- f729734 parser: Fix decorator placed incorrectly in initializers (#11461) (Boshen)

### Performance

- 7a5295d parser: Skip `try_parse` when current token is not an identifier (#11475) (leaysgur)
- 25167f2 parser: Parse ts type signature without rewind (#11443) (Boshen)
- 2e5a243 parser: Rewrite parse object literal element to avoid rewind (#11431) (Boshen)
- b776847 parser: Parse `async function` without rewind (#11427) (Boshen)
- dc110f5 parser: Parse binding list without token peek (#11423) (Boshen)
- 2953a07 parser: Parse `<` without token peek (#11422) (Boshen)
- 6203181 parser: Parse jsx open fragment without token peek (#11421) (Boshen)
- 767e759 parser: Import `import` statement without token peak (#11420) (Boshen)
- e41f85c parser: Optimize around `eat_decorators` (#11416) (Ulrich Stark)
- d79cac1 parser: Parse `const` declaration without token peek (#11419) (Boshen)
- b1d8d98 parser: Parse `let` declaration without token peek (#11413) (Boshen)
- eaf19ed parser: Optimize around `parse_type_arguments_in_expression` (#11417) (Ulrich Stark)

### Refactor

- bf974da parser: Remove lookahead for parsing import declarations and specifiers (#11381) (camchenry)
- 333b801 parser: Reduce backtracking during postfix type parsing (#11432) (therewillbecode)
- a3e1585 parser: Reduce backtracking for literal type node parsing (#11426) (therewillbecode)
- ed57fa3 parser: Reduce backtracking for assertion signature parsing (#11424) (therewillbecode)

## [0.72.2] - 2025-05-31

### Bug Fixes

- daaa8f5 parser: Correctly parse decorators of property declaration (#11370) (magic-akari)

### Performance

- 24aba18 parser: Avoid checkpoint when parsing left curly in jsx (#11377) (Ulrich Stark)
- 1bdeed2 parser: Remove lexer lookahead (#11349) (Boshen)

### Refactor

- 996194a parser: Remove unnecessary Tristate and checks (#11404) (Ulrich Stark)
- cd3ed4d parser: Replace `at` and `bump` combinations with `eat` (#11390) (Ulrich Stark)
- 4c49274 parser: Rewrite import/export specifier parsing (#11356) (camchenry)
- bfaa443 parser: Consolidate export type `lookahead()` calls (#11341) (leaysgur)

## [0.72.1] - 2025-05-28

### Performance

- 14cb3c7 parser: Simplify getting span of identifiers and literals (#11323) (overlookmotel)
- 2372f00 parser: `check_identifier` match on `Kind` not `&str` (#11322) (overlookmotel)
- 552a977 parser: Avoid work in `parse_function_id` (#11321) (overlookmotel)
- 6eda38a parser: Remove branch parsing class elements (#11319) (overlookmotel)

### Refactor

- 069b843 parser: Avoid peek in parse_delimited_list (#11343) (leaysgur)
- 99e6490 parser: Remove lexer lookahead in module parsing (#11330) (camchenry)
- 08eb1eb parser: Align jsx parsing to tsc (#11314) (leaysgur)
- 54dfbd3 parser: Remove Lexer lookahead from JS function parsing (#11307) (therewillbecode)
- 44bb9fb parser: Remove lexer lookahead in JS let declaration parsing (#11308) (therewillbecode)
- 2e43b6f parser: Remove Lexer peeking for js/expression (#11298) (leaysgur)
- 7f2d660 parser: Remove lexer lookahead in object parsing (#11274) (camchenry)
- 8a062b5 parser: Remove lexer lookahead in JS statement parsing (#11273) (camchenry)

## [0.72.0] - 2025-05-24

### Features

- 03390ad allocator: `TakeIn` trait with `AllocatorAccessor` (#11201) (Boshen)
- 2398906 parser: Check mixed coalesce and new exponential with `preserveParens:false` (#11264) (Boshen)

### Bug Fixes

- aa510cf parser: Produce syntax error for `({}) = x` when `preserveParens:false` (#11263) (Boshen)
- dcdcf12 parser: Parse `new (import("x"))` with `preserveParens: false` (#11251) (Boshen)
- 8e8dea5 parser: Fix incorrect token start in `re_lex_right_angle` (#11204) (Boshen)

### Performance

- 254048d lexer: Remove string allocation (#11255) (overlookmotel)
- 14fcf89 parser: Remove redundant checks (#11207) (Ulrich Stark)

### Refactor

- 02d3bb7 parser: Use `StringBuilder` instead of `String` (#11259) (overlookmotel)
- b99749c parser: Remove lexer lookahead in parsing TS statements (#11253) (camchenry)
- 4e12796 parser: Remove Lexer peeking for js/class (#11243) (leaysgur)
- 6ddf7a8 parser: Remove token lookahead in type parsing (#11241) (camchenry)
- 86e753d parser: Remove Lexer peeking for jsx (#11232) (leaysgur)
- 07e6ae0 parser: Remove Lexer peeking for modifiers (#11228) (leaysgur)
- 62f7184 parser: Replace peek in `parse_rest_binding` with checkpoint (#11225) (camchenry)
- def05bc parser: Remove lookahead usage in parsing arrow function expressions (#11220) (camchenry)
- a9dbf0a parser: Use checkpoints instead of `peek_at` in `is_un_parenthesized_async_arrow_function_worker` (#11218) (camchenry)
- a4e2eb1 parser: Make lexer code slightly more readable (#11212) (Ulrich Stark)

## [0.71.0] - 2025-05-20

- 1a4fec0 codegen: [**BREAKING**] A legal comment can also be a jsdoc comment (#11158) (Boshen)

### Bug Fixes

- 83e4f9b parser: Fix reading `Token` flags on big-endian systems (#11153) (overlookmotel)
- ef72143 parser: Parse index signature with multiple parameter (#11068) (Boshen)

### Performance

- 6571b9b ast: Use bitflags for storing comment newline state (#11096) (camchenry)
- b9e51e2 ast: Reduce size of `Comment` to 16 bytes (#11062) (camchenry)
- 0f9b43e lexer: Tighten search loops (#11118) (overlookmotel)
- 261e78b lexer: Use `offset_from` and `offset_from_unsigned` for pointer comparisons (#11116) (overlookmotel)

### Refactor

- 58c7de6 ast: Rename `CommentNewlines` fields (#11151) (overlookmotel)
- 7b9ab22 parser: Use bump instead of eat if ignoring return value (#11137) (Ulrich Stark)
- bb8bde3 various: Update macros to use `expr` fragment specifier (#11113) (overlookmotel)

## [0.70.0] - 2025-05-15

### Bug Fixes

- 635aa96 napi: Computed final source type from `lang` then `sourceType` (#11060) (Boshen)
- 4c9a9b3 parser: Guard against re-lex tokens when fatal error (#11023) (Boshen)
- 2b02d84 parser: Allow `for(using using` stmts (#10985) (camc314)

### Performance

- 80c2a5b parser: Use 8 bits for each `Token` flag (#11046) (overlookmotel)
- a711ff4 parser: Make `Kind::Eof` (default) 0 (#11014) (overlookmotel)
- cfd1ed3 parser: Explore packed tokens (#10933) (Tom Gasson)

### Refactor

- 47c624b lexer: Re-order `Token` methods (#11040) (overlookmotel)
- c0b68eb lexer: Harden safety of transmute (#11013) (overlookmotel)
- 54bfb4b lexer: Tidy tests for `Token` (#11011) (overlookmotel)
- de3035a parser: `Token::set_has_separator` take `bool` (#11041) (overlookmotel)
- 919cc59 parser: Make `ParserImpl::asi` implementation more compact (#11037) (Boshen)
- 751876b parser: Rewrite parse class element (#11035) (Boshen)
- b526da9 parser: Make `Token` fields private (#10936) (Boshen)
- c993edd parser/lexer: Shorten code (#10999) (overlookmotel)

### Styling

- c049765 lexer: Reformat comments (#11012) (overlookmotel)

## [0.69.0] - 2025-05-09

- 2b5d826 ast: [**BREAKING**] Fix field order for `TSTypeAssertion` (#10906) (overlookmotel)

- 1f35910 ast: [**BREAKING**] Fix field order for `TSNamedTupleMember` (#10905) (overlookmotel)

- 8a3bba8 ast: [**BREAKING**] Fix field order for `PropertyDefinition` (#10902) (overlookmotel)

- 5746d36 ast: [**BREAKING**] Fix field order for `NewExpression` (#10893) (overlookmotel)

- 0139793 ast: [**BREAKING**] Re-order fields of `TaggedTemplateExpression` (#10889) (overlookmotel)

- 6646b6b ast: [**BREAKING**] Fix field order for `JSXOpeningElement` (#10882) (overlookmotel)

- cc2ed21 ast: [**BREAKING**] Fix field order for `JSXElement` and `JSXFragment` (#10881) (overlookmotel)

- ad4fbf4 ast: [**BREAKING**] Simplify `RegExpPattern` (#10834) (overlookmotel)

### Features

- 3cf867c napi/parser: Expose module record data for `export default interface` (#10894) (Boshen)
- 539eb9d parser: `accessor` modifier cannot be used with `readonly` and `declare` modifier. (#10870) (Boshen)

### Bug Fixes

- 2c09243 ast: Fix field order for `AccessorProperty` (#10878) (overlookmotel)
- 2c05fa1 parser: Fix rhs precedence while parsing `PrivateInExpression` (#10866) (Boshen)
- 087af52 parser: Set the correct context for class property definition (#10859) (Boshen)

### Refactor

- d5cd29d parser: Refactor parse member expression (#10880) (Boshen)

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

### Bug Fixes

- 7234ba4 estree: Adjust span for `TSTypePredicate`.`typeAnnotation` (#10711) (Yuji Sugiura)
- 2718f29 parser: Fix panic when the parser tries to re-lex `>>` (#10756) (Boshen)
- d1d05d3 parser: Check comma in JSX expr lazily (#10739) (Yuji Sugiura)
- f803807 parser: Fix crash when parsing `for(in` (#10640) (Boshen)

### Performance

- 4861a62 parser: Faster parsing `TemplateElement`s (#10678) (overlookmotel)

### Documentation

- 24ada6f lexer: Correct comment (#10700) (overlookmotel)

### Refactor


## [0.67.0] - 2025-04-27

### Features

- e228840 parser: Fast forward lexer to EOF if errors are encountered (#10579) (Boshen)

### Bug Fixes

- a9785e3 parser,linter: Consider typescript declarations for named exports (#10532) (Ulrich Stark)

### Performance

- e6d5a44 lexer: Use `get_unchecked` for byte access in comment parsing (#10635) (camc314)
- f89aec6 parser: Improve perf of checking for licence/legal comments (#10616) (Cameron)
- 7059ffa parser: Mark error paths as cold (#10614) (Don Isaac)
- 3fafc0d parser: Fast path for parsing ts declarations (#10596) (Boshen)
- 4f56b2c parser: Remove `-> Result<T>` from all parsing methods (#10588) (Boshen)

### Refactor

- 76ea6a9 parser: Remove return `Result` from `read_regex` (#10598) (Boshen)

## [0.66.0] - 2025-04-23

### Features

- 7d5ad7d parser: Report error when `import type { type }` is used (#10528) (camc314)
- 6e40fac parser: Report error when `export type { type }` is used (#10524) (camc314)

## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

### Features

- 5ba02b0 parser: Set `pure` on typescript wrapped AST nodes (#10520) (Boshen)
- 588da69 parser: A rest parameter cannot have an initializer (#10467) (Boshen)
- c8336dd parser: Error for `const { ...a: b } = {}` (#10466) (Boshen)

### Bug Fixes

- 4f1343b parser: Fix missing type export in module information (#10516) (Ulrich Stark)
- 7664bd0 parser: Fix `using` asi (#10504) (Boshen)
- b7e0536 parser: Correct AST for `a<b>?.()` (#10461) (Boshen)

### Documentation

- ac23773 parser: Update parser example (#10468) (overlookmotel)

### Refactor


## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)
- 2c66ac2 codegen: Preserve code coverage ignore comments (e.g. `v8 ignore`) (#10338) (Boshen)

### Bug Fixes

- 9734152 ast: Handle `TSThisType` in `TSTypePredicate` (#10328) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)
- 5850a0d parse: `type x = typeof import('')` -> ` TSTypeQuery(TSImportType)` (#10317) (Boshen)
- 58ab8ff parser: Adjust class start position when decorators are involved (#10438) (Boshen)
- 3d7bcac parser: Fix span position for `+ ++x` (#10429) (Boshen)
- f9fd666 parser: Report errors for duplicate extends/implements clauses(TS1172/1173/1175) (#10420) (Yuji Sugiura)
- 385d009 parser: Correctly handle `?` postfixed element type in `TupleType` (#10390) (Yuji Sugiura)
- 41d8e9d parser: `ExportNamedDeclaration.exportKind` should be `type` for `declare` declaration (#10389) (Yuji Sugiura)
- 4fe9151 parser: Handle `JSDocUnknownType` correctly (#10363) (Yuji Sugiura)

### Performance

- 93b8e86 parser: Use `ArenaVec` to store decorators (#10437) (Dunqing)
- 0a42695 parser: Pass span starts (u32) around instead of Span (2x u32) (#10433) (Boshen)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)
- a6b2232 parser: Shorten code (#10445) (overlookmotel)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)
- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)

### Performance

- fa0e455 cfg, diagnostics, lexer, syntax, tasks: Remove `write!` macro where unnecessary (#10236) (overlookmotel)

### Documentation

- d8bbe2a lexer: Fix doc comment (#10181) (overlookmotel)

### Refactor

- ec10d94 parser: Use `AstBuilder::string_literal_with_lone_surrogates` (#10178) (overlookmotel)
- bcdbd38 transformer, minifier: Replace `AstBuilder::move_xxxx` methods with `TakeIn` trait (#10170) (Dunqing)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

## [0.62.0] - 2025-04-01

### Bug Fixes

- 418cfaf parser: Handle asi for `declare module "foo";` (#10010) (Boshen)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Performance

- 59b855f lexer: Faster decoding unicode escape sequences (#10073) (overlookmotel)

### Refactor

- 630e189 lexer: Shorten code for parsing hex digit (#10072) (overlookmotel)
- a24cceb lexer: Remove unnecessary line (#10042) (overlookmotel)
- c7079b5 lexer: Clarify and reformat comments (#10040) (overlookmotel)
- 326b4df lexer: Simplify macros for string parsing + correct comment (#10039) (overlookmotel)
- b93c394 parser: Rename var (#10012) (overlookmotel)

## [0.61.2] - 2025-03-23

### Bug Fixes

- eaea5fd parser: Handle invalid surrogate pair as lossy (#9964) (hi-ogawa)
- e696fda parser: Fix broken `regular_expression` feature (#9963) (Boshen)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 38ad787 data_structures: Add `assert_unchecked!` macro (#9885) (overlookmotel)
- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)
- d4a83ba parser: Report duplicate modifier `Accessibility modifier already seen.` (#9890) (Boshen)
- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

### Bug Fixes

- a113f7e parser: Error when `}` and `>` appear in `JSXText` (#9777) (Boshen)
- 8abb4f6 parser: Correctly set `export_kind` for `ExportNamedDeclaration` (#9827) (camc314)
- f707d1f parser: Set kind of var_declarator correctly for using decl (#9753) (camc314)

### Performance

- 6fc26db lexer: Mark error case as cold branch when parsing `JSXText` (#9831) (overlookmotel)

### Refactor

- 3945385 parser: Simplify parsing extends clause (#9773) (Dunqing)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features


### Performance

- a83cebd parser: Do not call `ParserImpl::end_span` twice for `StringLiteral`s (#9737) (overlookmotel)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 638007b parser: Apply `preserveParens` to `TSParenthesizedType` (#9653) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)
- cfdcfdb parser: Fix end span for optional binding pattern without type annotation (#9652) (Boshen)
- 26da65d parser: Parse asi after class accessor property (#9623) (Boshen)
- 87462fd parser: Fix end span for `using` declaration (#9622) (Boshen)

## [0.56.4] - 2025-03-07

### Refactor

- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.55.0] - 2025-03-05

### Features

- 2326cef parser: Apply `pure` to argument of unary expression (#9530) (Dunqing)

### Bug Fixes

- a88eb56 parser: Parsing errors occur when type parameters are followed by `as` or `satisfies` (#9553) (Dunqing)
- 2c6e3f1 parser: Fix false positive parsing optional member expr (#9534) (camc314)

## [0.54.0] - 2025-03-04

- 098f652 codegen: [**BREAKING**] Add `CommentAnnotation` to avoid parsing comments again (#9506) (Boshen)

- a8d1d48 parser,codegen: [**BREAKING**] Parse and print`#__NO_SIDE_EFFECTS__` (#9496) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 7d7f16c parser: Apply pure to rhs of binary expression (#9492) (Boshen)
- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)
- 9b7017c parser,codegen: Pure annotations (#9351) (Boshen)

### Bug Fixes

- 9c6ae9f parser: `@__NO_SIDE_EFFECTS` only affects const variable decl (#9517) (Boshen)
- b7d5513 parser: Parse `@__NO_SIDE_EFFECTS__` between `export default` and `async function` (#9514) (Boshen)
- 01de74c parser: Correctly fail parsing when parsing `foo.bar?.` (#9499) (camc314)
- 58defe3 parser: Mark expression as pure in chain expression (#9479) (sapphi-red)
- 2a03689 parser: Mark expressions on the left side of logical and conditional expressions as pure (#9414) (sapphi-red)

### Performance


## [0.53.0] - 2025-02-26

### Performance

- 61939ca ast/estree: Faster UTF-8 to UTF-16 span conversion (#9349) (overlookmotel)

### Refactor

- 7427900 ast: Re-order `ExportDefaultDeclaration` fields (#9348) (overlookmotel)
- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)
- 4e9e8cf lexer: Reduce scope of `unsafe` blocks (#9320) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features

- bde4126 parser: Parse `a ? b ? (c = 0) : d => 1 : (e = 2) : f => 3` (#9229) (Boshen)

### Documentation

- 3414824 oxc: Enable `clippy::too_long_first_doc_paragraph` (#9237) (Boshen)

### Refactor

- ef856f5 oxc: Apply `clippy::needless_pass_by_ref_mut` (#9253) (Boshen)
- d615b34 parser: Add `ArrowFunctionHead` struct (#9222) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.51.0] - 2025-02-15

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

### Features


### Bug Fixes

- bc64c9d lexer: Fix decoding lone `\r` in template literals (#9066) (overlookmotel)
- b8278d8 parser: Parse `let _: null` as `TSNullKeyword` (#9133) (Boshen)

## [0.50.0] - 2025-02-12

- d9189f1 ast: [**BREAKING**] Remove `PrivateInExpression::operator` field (#9041) (overlookmotel)

### Bug Fixes

- 662ab90 parser: Correct AST for `#field in x << y` (#9039) (Boshen)
- 567bc2c parser: Fix `SequenceExpression` span (#9035) (hi-ogawa)

### Refactor


## [0.49.0] - 2025-02-10

- bbb075d ast: [**BREAKING**] Name `AstBuilder` enum builders after variant name not type name (#8890) (overlookmotel)

### Refactor


### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- cebb350 minfier: Clean up some esbuild tests (Boshen)

## [0.48.1] - 2025-01-26

### Features

- b7f13e6 ast: Implement utf8 to utf16 span converter (#8687) (Boshen)

## [0.48.0] - 2025-01-24

### Bug Fixes

- 178c232 parser: Parse `intrinsic` TS keyword (#8627) (Kevin Deng 三咲智子)
- 48717ab parser: Parse `true` as `TSLiteralType` (#8626) (Kevin Deng 三咲智子)

### Performance

- 3fa87ff lexer: Peak 2 bytes after `!` (#8662) (Boshen)

### Documentation

- 3be0392 lexer: Fix doc comment (#8664) (overlookmotel)

### Refactor

- 864b8ef parser: Shorten code (#8640) (overlookmotel)
- b8d9a51 span: Deal only in owned `Atom`s (#8641) (overlookmotel)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.47.0] - 2025-01-18

### Features

- c479a58 napi/parser: Expose dynamic import expressions (#8540) (Boshen)

### Refactor

- 2857ae1 parser: Refactor visitor in regexp example (#8524) (overlookmotel)
- b5ed58e span: All methods take owned `Span` (#8297) (overlookmotel)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 2da4365 parser: Missing initializer in destructuring declaration inside for loop head (#8222) (Boshen)

### Bug Fixes

- e1f8ea4 lexer: `Source` is not `Clone` (#8294) (overlookmotel)
- f88acb3 parser: Allow line breaks between `const` and `enum` (#8193) (branchseer)

### Refactor

- 64bfdfe lexer: Tighten safety of lexer by always including lifetime on `SourcePosition` (#8293) (overlookmotel)
- 0344e98 lexer: Make `handle_byte` a method of `Lexer` (#8291) (overlookmotel)
- fabf116 lexer: Replace `#[allow]` with `#[expect]` (#8289) (overlookmotel)
- 0462edb lexer: Rename function param (#8288) (overlookmotel)

### Styling

- 4d2888d lexer: Reorder imports (#8290) (overlookmotel)

### Testing

- 16dcdaf lexer: Assert size of `Token` in 32-bit WASM (#8292) (overlookmotel)

## [0.44.0] - 2024-12-25

- ad2a620 ast: [**BREAKING**] Add missing `AssignmentTargetProperty::computed` (#8097) (Boshen)

### Bug Fixes

- 55d6eb9 parser: Disallow type parameters on class constructors (#8071) (injuly)
- be2c60d parser: Parse `import source from from 'mod'` (#8056) (Boshen)

## [0.42.0] - 2024-12-18

### Features

- 81eedb1 parser: 'readonly' type modifier is only permitted on array and tuple literal types. (#7880) (Boshen)

### Bug Fixes

- 111dc52 parser: Include export token in spans of TSNamespaceExportDeclaration (#7963) (branchseer)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.41.0] - 2024-12-13

- fb325dc ast: [**BREAKING**] `span` field must be the first element (#7821) (Boshen)

### Bug Fixes

- 7610dc1 parser: Parse `import source from 'mod'` (#7833) (Boshen)

### Refactor


## [0.40.0] - 2024-12-10

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- 00fea92 napi/parser: Expose span positions of `import.meta` (#7677) (Boshen)
- b8dc333 syntax: Add `ExportEntry::is_type` (#7676) (Boshen)

### Bug Fixes

- 8c0b0ee parser: Better diagnostic for invalid `for await` syntax (#7649) (Boshen)

### Performance

- d503a84 parser: Reorder `parse_statement` match conditions (#7645) (Boshen)
- e923e4e parser: Inline all token kind checks (#7644) (Boshen)

### Refactor

- 36d1493 parser: Use `ModuleRecord::has_module_syntax` for setting sourceType (#7646) (Boshen)

## [0.39.0] - 2024-12-04

- c2ced15 parser,linter: [**BREAKING**] Use a different `ModuleRecord` for linter (#7554) (Boshen)

- 8a788b8 parser: [**BREAKING**] Build `ModuleRecord` directly in parser (#7546) (Boshen)

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features

- 33e5a49 syntax: Add statement span to `ImportEntry` and `ExportEntry` (#7583) (Boshen)

### Refactor

- b24beeb parser: Use `PropName` trait from `oxc_ecmascript` (#7543) (Boshen)
- f0e7acc syntax: Change `ModuleRecord::not_esm` to `has_module_syntax` (#7579) (Boshen)
- 18519de syntax: Remove `ModuleRecord::export_default` (#7578) (Boshen)
- d476660 syntax: Remove `ModuleRecord::exported_bindings_duplicated` because it is a syntax error (#7577) (Boshen)
- 17663f5 syntax: Remove `ModuleRecord::export_default_duplicated` because it is a syntax error (#7576) (Boshen)

## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 878189c parser,linter: [**BREAKING**] Add `ParserReturn::is_flow_language`; linter ignore flow error (#7373) (Boshen)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- e6922df parser: Fix incorrect AST for `x?.f<T>()` (#7387) (Boshen)

### Bug Fixes

- 666b6c1 parser: Add missing `ChainExpression` in optional `TSInstantiationExpression` (#7371) (Boshen)

### Refactor


## [0.36.0] - 2024-11-09

- b11ed2c ast: [**BREAKING**] Remove useless `ObjectProperty::init` field (#7220) (Boshen)

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- d1d1874 ast: [**BREAKING**] Change `comment.span` to real position that contain `//` and `/*` (#7154) (Boshen)

### Features

- 9d6cc9d estree: ESTree compatibility for all literals (#7152) (ottomated)

### Refactor


## [0.35.0] - 2024-11-04

### Bug Fixes

- caaf00e parser: Fix incorrect parsed `TSIndexSignature` (#7016) (Boshen)

### Performance

- fa9a4ec parser: Check `.` before `[` in `parse_member_expression_rest` (#6979) (Boshen)

### Refactor

- 953b051 parser: Remove `oxc_ecmascript` crate (#7109) (Boshen)
- fdd480d parser: Do not use `AstBuilder::*_from_*` methods (#7068) (overlookmotel)
- 9e85b10 parser: Add `ParserImpl::alloc` method (#7063) (overlookmotel)
- 17a938e parser: Use function `parse_type_member_semicolon` (#7018) (Boshen)
- aa1b29c parser: Remove `parse_ts_index_signature_member` function (#7017) (Boshen)

## [0.34.0] - 2024-10-26

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

- a1ca964 ast, parser: [**BREAKING**] Remove `NumericLiteral::new` method (#6787) (overlookmotel)

- aeaa27a ast, parser, transformer, traverse: [**BREAKING**] Remove `BindingIdentifier::new` methods (#6786) (overlookmotel)

- ecc9151 ast, parser, transformer, traverse: [**BREAKING**] Remove `IdentifierReference::new` methods (#6785) (overlookmotel)

- 8032813 regular_expression: [**BREAKING**] Migrate to new regexp parser API (#6741) (leaysgur)

### Bug Fixes


### Refactor


## [0.32.0] - 2024-10-19

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

- 2808973 ast: [**BREAKING**] Add `Program::comments` (#6445) (Boshen)

### Features

- 58467a5 parser: Better handling of invalid modifiers (#6482) (DonIsaac)
- 8ea6b72 parser: Better errors for reserved words used as identifier names (#6478) (DonIsaac)

### Bug Fixes

- 721cf0f parser: Should be treated comments where after `(` as leading comments of next token (#6588) (Dunqing)
- b1bf12c parser: Do not parse `as` and `satisfies` expression in javascript (#6442) (Boshen)

### Performance

- 4d8bc8c parser: Precompute `is_typescript` (#6443) (Boshen)

### Refactor

- 073b02a ast: Type params field before params in TS function declaration types (#6391) (overlookmotel)
- c45723b parser: Fix typo in var name (#6500) (overlookmotel)

## [0.31.0] - 2024-10-08

- 01b878e parser: [**BREAKING**] Use `BindingIdentifier` for `namespace` declaration names (#6003) (DonIsaac)

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

- 32d972e parser: [**BREAKING**] Treat unambiguous files containing TS export assignments as modules (#6253) (overlookmotel)

### Features

- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

### Bug Fixes

- 6159560 parser: String `ImportSpecifier`s for type imports (#6352) (DonIsaac)
- 1380d8b parser: Should regard comments where after `=` as leading comments of next token (#6355) (Dunqing)

### Refactor

- 3b53dd4 parser: Provide better error messages for `const` modifiers on class elements (#6353) (DonIsaac)

## [0.30.4] - 2024-09-28

### Bug Fixes

- fd6798f parser: Remove unintended `pub Kind` (#6109) (Boshen)

## [0.30.2] - 2024-09-27

### Bug Fixes

- 0658576 paresr: Do not report missing initializer error in ambient context (#6020) (Boshen)

## [0.30.0] - 2024-09-23

### Features

- e8bf30a ast: Add `Comment::real_span` (#5764) (Boshen)
- bcdbba3 codegen: Print jsdoc comments that are attached to statements and class elements (#5845) (Boshen)
- 8e7556f parser: Calculate leading and trailing position for comments (#5785) (Boshen)

### Bug Fixes

- 42dcadf parser: Hashbang comment should not keep the end newline char (#5844) (Boshen)

### Documentation

- 3120c6c parser: Add module and struct level documentation (#5831) (DonIsaac)

### Refactor

- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)
- 31e9db4 parser: Shorten `UniquePromise` code (#5805) (overlookmotel)
- 2322b8b parser: Remove dead code warning when running tests (#5804) (overlookmotel)
- 4abfa76 parser: Add `--ast` and `--comments` to example (Boshen)
- a4b55bf parser: Use AstBuilder (#5743) (Boshen)

## [0.29.0] - 2024-09-13

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

- 603817b oxc: [**BREAKING**] Add `SourceType::Unambiguous`; parse `.js` as unambiguous (#5557) (Boshen)

### Features


### Performance


### Refactor

- 0ac420d linter: Use meaningful names for diagnostic parameters (#5564) (Don Isaac)

## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

### Features

- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)
- 10279f5 parser: Add syntax error for hyphen in `JSXMemberExpression` `<Foo.bar-baz />` (#5440) (Boshen)

### Refactor

- d9d7e7c ast: Remove `IdentifierName` from `TSThisParameter` (#5327) (overlookmotel)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

### Features

- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 49cd5db ast,parser: Add `definite` flag to `AccessorProperty` node (#5182) (DonIsaac)
- c2fa725 ast,parser: Parse `TSTypeAnnotations` on `AccessorProperty` (#5179) (DonIsaac)
- 7dfd51a parser: Report class properties that are both definite and optional (#5181) (DonIsaac)
- a563968 parser: Report errors on optional accessor properties (#5180) (DonIsaac)

### Bug Fixes

- d4c06ef parser: Revert "check for `@flow` with recoverable errors as well" (#5297) (overlookmotel)
- e1d8b92 parser: Check for `@flow` with recoverable errors as well (Boshen)
- e6fd52e parser: Change unterminated regex error to be non-recoverable (#5285) (Boshen)
- 1686920 parser: Span for invalid regex flags (#5225) (leaysgur)

### Refactor

- d236554 parser: Move `JSXIdentifier` conversion code into parser (#5345) (overlookmotel)
- bc59dd2 parser: Improve example for `byte_search!` macro usage (#5234) (overlookmotel)
- a3ddfdd parser: Improve lexer pointer maths (#5233) (overlookmotel)

### Testing

- 7009177 parser: Fix incorrect flow error test (Boshen)

## [0.25.0] - 2024-08-23

- b2ff2df parser: [**BREAKING**] Remove builder pattern from `Parser` struct (#5000) (Boshen)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 6800e69 oxc: Add `Compiler` and `CompilerInterface` (#4954) (Boshen)
- afe728a parser: Parse regular expression with regex parser (#4998) (Boshen)

### Bug Fixes

- efbdced parser: Only show flow error if it's a flow file (#5069) (Boshen)

### Refactor

- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)

## [0.24.3] - 2024-08-18

### Bug Fixes

- 21f5762 codegen: Minify large numbers (#4889) (Boshen)
- 1bdde2c parser: Detect @flow in `/** @flow */ comment (#4861) (Boshen)

## [0.24.2] - 2024-08-12

### Documentation

- 559baa5 parser: Clean up doc regarding performance; remove conformance (Boshen)

## [0.24.0] - 2024-08-08

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Refactor

- d25dea7 parser: Use `ast_builder` in more places. (#4612) (rzvxa)

## [0.23.1] - 2024-08-06

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Refactor

- d25dea7 parser: Use `ast_builder` in more places. (#4612) (rzvxa)

## [0.23.0] - 2024-08-01

### Features

- 7446e98 codegen: Align more esbuild implementations (#4510) (Boshen)
- 35654e6 codegen: Align operator precedence with esbuild (#4509) (Boshen)

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Performance

- 55a8763 parser: Faster decoding unicode escapes in identifiers (#4579) (overlookmotel)
- ae1d38f parser: Fast path for ASCII when checking char after numeric literal (#4577) (overlookmotel)
- 56ae615 parser: Make not at EOF the hot path in `Source` methods (#4576) (overlookmotel)
- 25679e6 parser: Optimize `Lexer::hex_digit` (#4572) (overlookmotel)
- bb33bcc parser: Speed up lexing non-decimal numbers (#4571) (overlookmotel)
- ab8509e parser: Use `-` not `saturating_sub` (#4561) (overlookmotel)
- c9c38a1 parser: Support peeking over bytes (#4304) (lucab)
- 0870ee1 parser: Get and check lookahead token (#4534) (lucab)

### Refactor

- e68ed62 parser: Convert lexer byte handler for `|` to a single match (#4575) (overlookmotel)
- bba824b parser: Convert `Lexer::read_minus` to a single match (#4574) (overlookmotel)
- ef5418a parser: Convert `Lexer::read_left_angle` to a single match (#4573) (overlookmotel)
- 9e5be78 parser: Add `Lexer::consume_2_chars` (#4569) (overlookmotel)
- 649913e parser: Extract `u8` not `&u8` when iterating over bytes (#4568) (overlookmotel)
- 59f00c0 parser: Rename function (#4566) (overlookmotel)
- 8e3e910 parser: Rename vars (#4565) (overlookmotel)
- 0c0601f parser: Rename function (#4564) (overlookmotel)
- 0acc4a7 parser: Fetch 2 bytes in `?` byte handler (#4563) (overlookmotel)
- 565eccf parser: Shorten lexer code (#4562) (overlookmotel)
- 148bdb5 parser: Adjust function inlining (#4530) (overlookmotel)

## [0.22.1] - 2024-07-27

### Performance

- 868fc87 parser: Optimize conditional advance on ASCII values (#4298) (lucab)

## [0.22.0] - 2024-07-23

- f68b659 ast: [**BREAKING**] Reorder fields of `ArrowFunctionExpression` (#4364) (Dunqing)

### Bug Fixes

- aece1df ast: Visit `Program`s `hashbang` field first (#4368) (overlookmotel)

### Refactor

- a2eabe1 parser: Use error codes for ts diagnostics (#4335) (DonIsaac)

## [0.21.0] - 2024-07-18

### Features

- 20cdb1f semantic: Align class scope with typescript (#4195) (Dunqing)

### Bug Fixes

- 9a87e41 parser: Avoid crashing on invalid const modifier (#4267) (lucab)
- 641a78b parser: Fix tests for number parsing (#4254) (overlookmotel)

### Performance

- a8dc4f3 parser: Speed up parsing numbers with `_` separators (#4259) (overlookmotel)
- b94540d parser: Speed up parsing octal literals (#4258) (overlookmotel)
- a7b328c parser: Faster parsing decimal numbers (#4257) (overlookmotel)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)
- ace4f1f semantic: Update the order of `visit_function` and `Visit` fields in the builder to be consistent (#4248) (Dunqing)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)
- 4a656c3 lexer: Incorrect lexing of large hex/octal/binary literals (#4072) (DonIsaac)
- 28eeee0 parser: Fix asi error diagnostic pointing at invalid text causing crash (#4163) (Boshen)

### Refactor


## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features

- 3a0f2aa parser: Check for illegal modifiers in modules and namespaces (#4126) (DonIsaac)

## [0.17.1] - 2024-07-06

### Performance

- 7fe2a2f parser: Do not copy comments (#4067) (overlookmotel)

## [0.17.0] - 2024-07-05

- e32b4bc ast: [**BREAKING**] Store trivia comments in a sorted slice (#4045) (Luca Bruno)

### Refactor

- 243c9f3 parser: Use function instead of trait to parse list with rest element (#4028) (Boshen)
- 1dacb1f parser: Use function instead of trait to parse delimited lists (#4014) (Boshen)

## [0.16.3] - 2024-07-02

### Refactor

- d0eac46 parser: Use function instead of trait to parse normal lists (#4003) (Boshen)

## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)
- 63f36da parser: Parse modifiers with `parse_modifiers` (take 2) (#3977) (DonIsaac)

## [0.16.1] - 2024-06-29

### Features

- 7b38bde parser: Parse modifiers with `parse_modifiers` (#3948) (DonIsaac)

### Refactor

- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- 1f85f1a ast: [**BREAKING**] Revert adding `span` field to the `BindingPattern` type. (#3899) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 5847e16 ast,parser: Add `intrinsic` keyword (#3767) (Boshen)
- dd540c8 minifier: Add skeleton for ReplaceGlobalDefines ast pass (#3803) (Boshen)

### Bug Fixes

- 275349a parser: Parse function type parameter name `accessor` (#3926) (Boshen)
- ef82c78 parser: Trailing comma is not allowed in ParenthesizedExpression (#3885) (Dunqing)
- 13754cb parser: Change diagnostic to "modifier cannot be used here" (#3853) (Boshen)

### Performance

- 4bf405d parser: Add a few more inline hints to cursor functions (#3894) (Boshen)- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- a471e62 parser: Clean up `try_parse` (#3925) (Boshen)
- 3db2553 parser: Improve parsing of TypeScript type arguments (#3923) (Boshen)
- 4cf3c76 parser: Improve parsing of TypeScript types (#3903) (Boshen)
- 187f078 parser: Improve parsing of `parse_function_or_constructor_type` (#3892) (Boshen)
- 97d59fc parser: Move code around for parsing `Modifiers` (#3849) (Boshen)- d6437fe Clean up some usages of `with_labels` (#3854) (Boshen)

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

