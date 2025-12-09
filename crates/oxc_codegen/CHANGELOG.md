# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.102.0] - 2025-12-08

### 🚀 Features

- a607cc4 codegen: Preserve comments between CatchClause's param and body (#16167) (copilot-swe-agent)

## [0.100.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 74cf572 ast: [**BREAKING**] Make `source` field of `TSImportType` a `StringLiteral` (#16114) (copilot-swe-agent)
- 43156ae ast: [**BREAKING**] Rename `TSImportType` `argument` field to `source` (#16110) (overlookmotel)

## [0.99.0] - 2025-11-24

### 💥 BREAKING CHANGES

- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

### 🐛 Bug Fixes

- e2ca770 codegen: Add support for printing type arguments in new expressions (#15963) (Ives van Hoorne)

## [0.97.0] - 2025-11-11

### 🐛 Bug Fixes

- 020aa4f codegen: Print space before `BindingRestElement` in `ObjectPattern` (#15315) (overlookmotel)

### ⚡ Performance

- ab4b12b codegen: Reduce branches printing `ObjectPattern` (#15316) (overlookmotel)

## [0.96.0] - 2025-10-30

### 🐛 Bug Fixes

- 3fbb307 codegen: Avoid invalid sourcemap tokens for positions beyond source bounds (#15069) (copilot-swe-agent)
- 4904710 codegen: Print legal comments above directives (#14993) (Boshen)


## [0.95.0] - 2025-10-15

### ⚡ Performance

- ea3f362 codegen: Reorder match arms based on usage patterns (#14496) (Boshen)


## [0.94.0] - 2025-10-06

### 🚀 Features

- 3656908 rust: Oxc-index-vec v4.0 (#14254) (Boshen)

### 🐛 Bug Fixes

- fc519c8 mangler: Mangle private class members in subsequent classes correctly (#14361) (sapphi-red)
- b83ffe5 mangler: Mangle private class members used in nested classes properly (#14218) (sapphi-red)


## [0.93.0] - 2025-09-28

### 🐛 Bug Fixes

- aa927ab codegen: Don't inject parenthesis in optional chaining within new expressions (#14170) (sapphi-red)


## [0.92.0] - 2025-09-24

### 🚀 Features

- 0fe4d95 mangler: Mangle private class members (#14027) (sapphi-red)


## [0.91.0] - 2025-09-22

### 🐛 Bug Fixes

- 8f3f460 codegen: Add missing dot when printing import.defer and import.source (#13975) (Boshen)

### ⚡ Performance

- ed8ff7d codegen: Unroll loop in `SourcemapBuilder::update_generated_line_and_column` (#13903) (翠)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)





## [0.88.0] - 2025-09-15

### 🐛 Bug Fixes

- bf50a02 codegen: Avoid backticks for object property keys in destructuring assignments (#13631) (copilot-swe-agent)

### ⚡ Performance

- d4608f1 codegen: Reduce memory usage in `SourcemapBuilder` (#13679) (overlookmotel)
- 4ded22b codegen: Reduce allocations in `SourcemapBuilder` (#13677) (overlookmotel)
- b35bf30 codegen: Optimize sourcemap builder to reduce allocations (#13670) (Boshen)
- 641b252 codegen: Reduce branches when printing `ObjectProperty` and `BindingProperty` (#13659) (overlookmotel)


## [0.87.0] - 2025-09-08

### 🐛 Bug Fixes

- e11a946 rust: Fix missing docs (#13541) (Boshen)
- 34d3cde rust: Fix clippy issues (#13540) (Boshen)


## [0.86.0] - 2025-08-31

### 💥 BREAKING CHANGES

- edeebc6 data_structures: [**BREAKING**] Rename `SliceIterExt` to `SliceIter` (#13439) (overlookmotel)

### 🚀 Features

- 5b139aa data_structures: Add `ptr` and `end_ptr` methods to `SliceIterExt` (#13435) (overlookmotel)
- d8b027f data_structures: Add `SliceIterExt::peek` method (#13434) (overlookmotel)

### 🚜 Refactor

- 9c3b060 codegen: Clarify choice of quote when printing strings (#13440) (overlookmotel)

### ⚡ Performance

- 39fc0d6 codegen: Use `isize` for quote counters (#13441) (overlookmotel)




## [0.83.0] - 2025-08-29

### 🐛 Bug Fixes

- b53a294 codegen: Add end sourcemaps for arguments (#13355) (sapphi-red)
- 1044566 codegen: Add end sourcemaps for array literals and object literals (#13354) (sapphi-red)

### 🧪 Testing

- f547c94 codegen: Add test that verifies stack traces are correct (#13351) (sapphi-red)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- 8f533aa codegen: Correct `CRLF` handling in comment processing (#13169) (copilot-swe-agent)
- f10ac33 codegen: Remove end sourcemaps for `}`, `]`, `)` (#13180) (Boshen)

### 🚜 Refactor

- 51ca0ad codegen: Reduce repeated code (#13191) (overlookmotel)
- 3548cf4 sourcemap: Improve sourcemap visualization code (#13177) (Boshen)

### ⚡ Performance

- e3bfff1 codegen: Faster splitting comments into lines (#13190) (overlookmotel)


## [0.82.2] - 2025-08-17

### 🚀 Features

- df3829c oxc_codegen: Support configure initial indent when using `oxc_codegen` (#13091) (IWANABETHATGUY)

### 🚜 Refactor

- 5223562 codegen: Adjust some source mappings (#13084) (Boshen)

### ⚡ Performance

- 1385c71 codegen: Further reduce memory allocations in `generate_line_offset_tables` (#13056) (overlookmotel)
- ab685bd codegen: Reduce memory allocations in `generate_line_offset_tables` (#13054) (Boshen)


## [0.82.1] - 2025-08-13

### 🚀 Features

- 2c5fa58 codegen: Allow attaching comment to the top of the file. (#13048) (Boshen)


## [0.82.0] - 2025-08-12

### 💥 BREAKING CHANGES

- 128b527 data_structures: [**BREAKING**] Remove `PointerExt` trait (#12903) (overlookmotel)

### 🚜 Refactor

- f6475e1 codegen: Remove the redundant `base_len` variable (#12882) (Dunqing)

### ⚡ Performance

- 017e200 codegen: Comprehensive optimization of `print_minified_number` method (#12847) (Copilot)

### 🎨 Styling

- af065ff codegen: Re-order imports (#12918) (overlookmotel)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)

### 🐛 Bug Fixes

- 3eed87a codegen: Wrap parens for `TSUnionType` (#12830) (Boshen)

### ⚡ Performance

- 2c4369a syntax,codegen: Replace `ryu_js` with `dragonbox_ecma` for floating point formatting (#12821) (Copilot)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)

### 🚀 Features

- af4d558 codegen: Add options to control indentation (#12691) (Copilot)

### 📚 Documentation

- 514322c rust: Add minimal documentation to example files in crates directory (#12731) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)

### 🎨 Styling

- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)




## [0.78.0] - 2025-07-24

### 🚀 Features

- c135beb codegen: Keep function expression PIFEs (#12470) (sapphi-red)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)

### 🚜 Refactor

- 8917a5f codegen: Remove clone of source text in sourcemap builder (#12393) (Boshen)

### ⚡ Performance

- 8bae417 codegen: Remove the useless tokens generated by some expressions (#12394) (Boshen)
- 7d64eb9 codegen: Replace loop + push with `std::iter::repeat` (#12398) (Boshen)



## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- bb047a9 codegen: Use `SliceIterExt` in string printer (#12296) (overlookmotel)



## [0.76.0] - 2025-07-08

### 💥 BREAKING CHANGES

- 8b30a5b codegen: [**BREAKING**] Introduce `CommentOptions` (#12114) (Boshen)

### 🐛 Bug Fixes

- 92af759 codegen: Wrap comment and object literal with parens to avoid invalid syntax (#12111) (Boshen)


## [0.75.1] - 2025-07-03

### 🚀 Features

- 28fca3c codegen: Preserve comments for lingui /*i18n*/ (#12047) (Boshen)

### 🐛 Bug Fixes

- 22799c3 codegen: Escape `</script` (#11782) (Cheng Xu)
- b8ede17 codegen: Fix sourcemap for template literals (#12028) (Boshen)

### 🚜 Refactor

- e0d70ef codegen: Clarify comments (#12036) (overlookmotel)

### ⚡ Performance

- 5b99cad codegen: Optimize printing strings (#12040) (overlookmotel)
- 41ec54e codegen: Reduce branches printing `TemplateLiteral`s (#12041) (overlookmotel)
- 3982963 codegen: Remove branch (#12037) (overlookmotel)


## [0.75.0] - 2025-06-25

### 🐛 Bug Fixes

- f41705f codegen: Do not generate sourcemap for empty length spans (#11892) (Boshen)





## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🐛 Bug Fixes

- 7266200 parser: Parse `@x() @y() export default abstract class {}` (#11630) (Boshen)
- e4804ba parser: Parse decorator on `abstract class` (#11625) (Boshen)

### 🚜 Refactor

- e9a8832 parser: Rewrite decorator parsing (#11604) (Boshen)

### 📚 Documentation

- 630a17a codegen: Clarify `Codegen::source_map_path` option (#11573) (Boshen)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Features

- 7e88451 parser: Syntax errors for decorators appearing in conflicting places (#11482) (Boshen)

### Bug Fixes

- 392752f parser: Handle `import {type as as}` correctly (#11488) (camchenry)
- f729734 parser: Fix decorator placed incorrectly in initializers (#11461) (Boshen)

### Performance

- 25167f2 parser: Parse ts type signature without rewind (#11443) (Boshen)

## [0.72.2] - 2025-05-31

### Bug Fixes

- daaa8f5 parser: Correctly parse decorators of property declaration (#11370) (magic-akari)

## [0.71.0] - 2025-05-20

- 1a4fec0 codegen: [**BREAKING**] A legal comment can also be a jsdoc comment (#11158) (Boshen)

### Features

- fa06d7f codegen: Print override modifier of FormalParameter (#11134) (Ulrich Stark)
- c29b1b8 codegen: Deduplicate repeated legal comments (#11069) (Boshen)
- c79a7d0 data_structures: Introduce `PointerExt` trait (#11095) (overlookmotel)

### Bug Fixes

- ef72143 parser: Parse index signature with multiple parameter (#11068) (Boshen)

### Performance

- b9e51e2 ast: Reduce size of `Comment` to 16 bytes (#11062) (camchenry)

## [0.70.0] - 2025-05-15

### Features

- 4a72b58 codegen: Print comments inside `JSXEmptyExpression` (#11022) (Boshen)
- 1673ffb codegen: Rework printing normal / legal / annotation comments (#10997) (Boshen)

### Refactor

- 9225bde codegen: Make `Statement::Gen` code more compact (#10937) (Boshen)
- 751876b parser: Rewrite parse class element (#11035) (Boshen)

## [0.69.0] - 2025-05-09

- ad4fbf4 ast: [**BREAKING**] Simplify `RegExpPattern` (#10834) (overlookmotel)

### Bug Fixes

- 2c05fa1 parser: Fix rhs precedence while parsing `PrivateInExpression` (#10866) (Boshen)
- 087af52 parser: Set the correct context for class property definition (#10859) (Boshen)

### Refactor


## [0.68.1] - 2025-05-04

### Bug Fixes

- 368d05f codegen: Make `source_text` an option, avoid panic (#10790) (Cameron)

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

- 315143a codegen: [**BREAKING**] Remove useless `CodeGenerator` type alias (#10702) (Boshen)

### Features

- 2d13b49 codegen: Expose `with_source_text` function (#10768) (camc314)
- b01cb45 codegen: A way to keep legal comments after minification (#10689) (Boshen)

### Bug Fixes

- 4825eb5 codegen: Add missing `in` `out` from ts type parameter (#10696) (Boshen)
- 06f796f codegen: Add missing return type from object methods (#10694) (Boshen)
- 8c499c6 linter: Fix panic when doing code gen on regexp (#10769) (camc314)

### Refactor


## [0.66.0] - 2025-04-23

- 10e1018 codegen: [**BREAKING**] Print `StringLiteral` `raw` if `minify` option disabled (#10553) (overlookmotel)

### Features


### Bug Fixes

- 3ebf220 codegen: Generate missing `type` in `export type {} from 'mod'` (#10539) (Boshen)

### Performance

- 6a045c8 codegen: Speed up printing `Directive`s (#10551) (overlookmotel)

### Testing

- 14bb2be codegen: Add more tests for strings (#10552) (overlookmotel)

## [0.65.0] - 2025-04-21

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

### Features

- 5ba02b0 parser: Set `pure` on typescript wrapped AST nodes (#10520) (Boshen)

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

- 82ba30b codegen: Fix spaces before `AssignmentTargetRest` (#10443) (overlookmotel)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)
- 5850a0d parse: `type x = typeof import('')` -> ` TSTypeQuery(TSImportType)` (#10317) (Boshen)
- 41d8e9d parser: `ExportNamedDeclaration.exportKind` should be `type` for `declare` declaration (#10389) (Yuji Sugiura)

### Performance

- 426f8cb codegen: Reduce checks printing strings (#10341) (overlookmotel)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)

### Performance

- 6c9b094 codegen: Optimize `Codegen::print_list` method (#10242) (Dunqing)
- 775abac codegen: Speed up printing `StringLiteral`s (#10046) (overlookmotel)

### Refactor

- f7ff816 codegen: Improve code with `split_first` (#10285) (Boshen)
- ca8f174 codegen: Do not print useless comma for TSEnumMember (#10213) (Yuji Sugiura)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

### Testing

- 7672620 parser: Tests for lone surrogates and lossy escape characters (#10175) (overlookmotel)

## [0.62.0] - 2025-04-01

### Bug Fixes

- 9b6e344 codegen: Do not escape `$` in strings unless using backtick as quote (#10103) (overlookmotel)
- c903e42 codegen: Prevent arithmetic overflow calculating quote for `StringLiteral`s (#10102) (overlookmotel)
- 418cfaf parser: Handle asi for `declare module "foo";` (#10010) (Boshen)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Performance

- d190466 codegen: Faster printing quotes (#10080) (overlookmotel)

### Refactor

- 719742b codegen: Print string literals containing lone surrogates without reference to `raw` (#10044) (overlookmotel)

## [0.61.2] - 2025-03-23

### Bug Fixes

- eaea5fd parser: Handle invalid surrogate pair as lossy (#9964) (hi-ogawa)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.59.0] - 2025-03-18

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Bug Fixes

- a113f7e parser: Error when `}` and `>` appear in `JSXText` (#9777) (Boshen)
- 8abb4f6 parser: Correctly set `export_kind` for `ExportNamedDeclaration` (#9827) (camc314)

## [0.58.0] - 2025-03-13

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 7331656 semantic: [**BREAKING**] Rename `SymbolTable` and `ScopeTree` methods (#9613) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

### Refactor


## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.55.0] - 2025-03-05

### Features

- 2326cef parser: Apply `pure` to argument of unary expression (#9530) (Dunqing)

### Performance

- 6b4a8c6 ast, codegen, transformer: Avoid allocations when converting `RegExpFlags` to string (#9550) (overlookmotel)

## [0.54.0] - 2025-03-04

- 098f652 codegen: [**BREAKING**] Add `CommentAnnotation` to avoid parsing comments again (#9506) (Boshen)

- a8d1d48 parser,codegen: [**BREAKING**] Parse and print`#__NO_SIDE_EFFECTS__` (#9496) (Boshen)

### Features

- 7d7f16c parser: Apply pure to rhs of binary expression (#9492) (Boshen)
- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)
- 9b7017c parser,codegen: Pure annotations (#9351) (Boshen)

### Bug Fixes

- 75f06ad codegen: Do not print comments when only `annotation_comments` is enabled (#9518) (Dunqing)
- 9c6ae9f parser: `@__NO_SIDE_EFFECTS` only affects const variable decl (#9517) (Boshen)
- b7d5513 parser: Parse `@__NO_SIDE_EFFECTS__` between `export default` and `async function` (#9514) (Boshen)
- 58defe3 parser: Mark expression as pure in chain expression (#9479) (sapphi-red)
- 2a03689 parser: Mark expressions on the left side of logical and conditional expressions as pure (#9414) (sapphi-red)

### Performance


### Refactor

- 19c4835 codegen: Simplify printing comments between arguments in call-like expressions (#9501) (Dunqing)

## [0.53.0] - 2025-02-26

### Performance

- 35ee399 codegen: Use `iter::repeat_n` in `CodeBuffer` (#9325) (overlookmotel)

### Refactor

- 9d98444 codegen, data_structures: Move `CodeBuffer` into `oxc_data_structures` crate (#9326) (overlookmotel)

## [0.52.0] - 2025-02-21

### Bug Fixes

- 1cc1669 codegen: Fix `clippy::unused_peekable` warning (#9236) (Boshen)

### Refactor

- 97cc1c8 ast: Remove `TSLiteral::NullLiteral` (replaced by `TSNullKeyword`) (#9147) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

### Testing

- 51c4491 codegen: Clean up esbuild tests (Boshen)

## [0.51.0] - 2025-02-15

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

- 9091387 ast: [**BREAKING**] Remove `TSType::TSQualifiedName` (#9051) (Dunqing)

### Features


### Bug Fixes

- d9684af codegen: Fix missing StringLiteral sourcemap (#9064) (hi-ogawa)

## [0.49.0] - 2025-02-10

### Features

- b4ee617 codegen: Prefer backquotes over double / single quotes (#8839) (sapphi-red)

### Bug Fixes

- be71b03 codegen: Parenthesis is lacking when a pure comment is placed before a function expression inside a call expression (#8968) (Dunqing)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.48.2] - 2025-02-02

### Bug Fixes

- 0928a19 codegen: Emit this parameters of class methods (#8834) (michaelm)

### Performance

- d8fac6d codegen: Avoid a heap allocation when printing floats (#8807) (Boshen)

### Refactor

- 6aa2dde codegen: Accept SymbolTable instead of Mangler (#8829) (Daniel Bulant)

## [0.48.1] - 2025-01-26

### Bug Fixes

- 0944758 codegen: Remove parens from `new (import(''), function() {})` (#8707) (Boshen)

## [0.48.0] - 2025-01-24

### Features

- 99607d3 codegen: Print comments in `TSTypeLiteral` (#8679) (Boshen)

### Performance

- d966e0a codegen: Do not check for comments if turned off (#8598) (Boshen)

### Refactor

- db863a3 codegen: Use `Stack` for `binary_expr_stack` (#8663) (Boshen)
- 8cce69a codegen: Remove `match_member_expression` (#8597) (Boshen)
- a3dc4c3 crates: Clean up snapshot files (#8680) (Boshen)
- 23b49a6 linter: Use `cow_to_ascii_lowercase` instead `cow_to_lowercase` (#8678) (Boshen)

### Testing

- 39dbd2d codegen: Fix snapshot file (#8685) (Boshen)

## [0.47.0] - 2025-01-18

### Bug Fixes

- 855c839 codegen: Shorthand assignment target identifier consider mangled names (#8536) (Boshen)

## [0.46.0] - 2025-01-14

### Refactor

- 6e64eef codegen: Remove `match_expression!` (#8450) (Boshen)

## [0.45.0] - 2025-01-11

### Features

- ad146bb codegen: Print real newline when `\n` is inside template literals (#8178) (Boshen)
- a542013 minifier: Minimize `do{}while(true)` -> `do;while(true)` (#8311) (Boshen)

### Bug Fixes

- a1752a0 codegen: Fix incorrect minified `return 1n` output (#8374) (Boshen)
- 5a648bc codegen: Fix white space issue with do statements (#8348) (Boshen)
- b6d16f4 codegen: Print parenthesis on negative bigint lit when neccessary (#8258) (camc314)
- 8ed9766 codegen: Source map builder panicked because it attempted to subtract with overflow in `search_original_line_and_column` (#8185) (Dunqing)
- ad61e70 codegen: Print if else without block with proper indentation (#8135) (Boshen)

## [0.44.0] - 2024-12-25

- ad2a620 ast: [**BREAKING**] Add missing `AssignmentTargetProperty::computed` (#8097) (Boshen)

### Features

- 618b6aa codege: Minify whitespace in object getter / setter (#8080) (Boshen)
- 4727667 codegen: Minify arrow expr `(x) => y` -> `x => y` (#8078) (Boshen)
- 0562830 codegen: Minify string with backtick when needed (#8095) (Boshen)
- 6237c05 codegen: Minify more whitespace (#8089) (Boshen)
- 6355b7c codegen: Minify `export { 's' as 's' }` -> `export { 's' }` (#8093) (Boshen)
- fccfda9 codegen: Minify `class{static[computed]}` (#8088) (Boshen)
- f873139 codegen: Minify `for (_ of [])` -> `for(_ of[])` (#8086) (Boshen)
- 8b8cbcd codegen: Minify `case "foo"` -> `case"foo"` (#8085) (Boshen)
- 414c118 codegen: Minify `yield "s"` -> `yield"s"` (#8084) (Boshen)
- f8f067b codegen: Minify class method `async*fn(){}` (#8083) (Boshen)
- 1d5ae81 codegen: Minify `const [foo] = bar` -> `const[foo]=bar` (#8079) (Boshen)
- e3f78fb codegen: `new Foo()` -> `new Foo` when minify (#8077) (Boshen)
- d84d60a codegen: Minify numbers with large exponents (#8074) (Boshen)
- 373279b codegen: Balance string quotes when minify whitespace (#8072) (Boshen)

### Bug Fixes

- bdc241d codegen: Disallow template literals in object property key (#8108) (Boshen)
- 728ed20 codegen: Print `yield * ident` correctly (Boshen)

### Performance

- 78d2e83 sourcemap: Improve perf of `search_original_line_and_column` (#7926) (Cameron)

### Refactor

- 7110c7b codegen: Add `print_quoted_utf16` and `print_unquoted_utf16` methods (#8107) (Boshen)

## [0.43.0] - 2024-12-21

### Performance

- 862838f codegen: Remove useless to_owned (#8014) (Dunqing)

## [0.42.0] - 2024-12-18

### Bug Fixes

- 850dd43 codegen: Missing `,` when generating type parameters with jsx (#7929) (Dunqing)

### Performance

- 4b24335 codegen: Improve printing of statement comments (#7857) (Boshen)
- 71a40a2 codegen: Guard comment printing comments when there are no comments (#7856) (Boshen)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.41.0] - 2024-12-13

### Performance

- 4448b63 codegen: Faster writing indentation (#7820) (overlookmotel)
- afaaffa codegen: Fast path for `options.print_comments()` (#7806) (Boshen)

## [0.40.0] - 2024-12-10

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- d0b78f7 codegen: Minify whitespace for some expressions (#7671) (Boshen)
- c523ccb codegen: Better whitespace minification for import / export statements (#7650) (Boshen)

### Bug Fixes

- a222f2b codegen: Print `delete 2e308` as `delete (0, Infinity)` (#7761) (Boshen)
- b701232 codegen: Print quote correctly for directive (#7735) (Boshen)
- 8c3a954 codegen: Missing parens for `in` in `for in` loop init (#7705) (Dunqing)
- 4afbe55 codegen: Missing parens for `in` in for loop init when it includes two binary expression (#7703) (Dunqing)

### Refactor


## [0.39.0] - 2024-12-04

### Bug Fixes

- e787e9d codegen: Print parentheses correctly for ClassHeritage (#7637) (Boshen)

## [0.38.0] - 2024-11-26

### Bug Fixes

- d5df615 oxc_codegen: Don't emit empty span mapping (#7448) (Hiroshi Ogawa)

## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- 82773cb codegen: Remove underscore from bigint (#7367) (Boshen)

### Bug Fixes

- c587dd3 codegen: Do not print parenthesis for `in` expression in ArrowFunctionExpression (#7360) (Dunqing)
- a0766e6 codegen: Fix arithmetic overflow printing unspanned nodes (#7292) (overlookmotel)
- 33ec4e6 codegen: Fix arithmetic overflow printing unspanned `NewExpression` (#7289) (overlookmotel)
- 1282221 codegen: Print comments when block is empty (#7236) (Boshen)

### Refactor

- 58db9ef codegen: Do not print unnecessary parentheses if both sides use the same logical operator (#7325) (Dunqing)

## [0.36.0] - 2024-11-09

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- d1d1874 ast: [**BREAKING**] Change `comment.span` to real position that contain `//` and `/*` (#7154) (Boshen)

### Features


## [0.35.0] - 2024-11-04

### Features

- caa4b1f codegen: Improve printing of comments (#7108) (Boshen)
- 001058a codegen: Always print legal comments on its own line (#7089) (Boshen)
- 413973d codegen: Print linked and external legal comment (#7059) (Boshen)
- ee27b92 codegen: Print eof legal comments (#7058) (Boshen)
- 6516f9e codegen: Print inline legal comments (#7054) (Boshen)

### Refactor

- dd79c1b codegen: Replace `daachorse` with string match for annotation comment (#7064) (Boshen)
- 0bb1aa4 codegen: Move options to its own file (#7053) (Boshen)

## [0.34.0] - 2024-10-26

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

### Bug Fixes

- 05ef03d codegen: Correct print `__proto__` shorthand (#6802) (Boshen)
- 1b7897c codegen: Print `#field in {} << 0;` correctly (#6799) (Boshen)
- 2f6ad42 codegen: Print negative bigint `1n- -1n` correctly after constant folding (#6798) (Boshen)
- 8f17953 coverage: Remove some broken cases (#6797) (Boshen)

### Documentation

- 374b972 codegen: Add `#![warn(missing_docs)]` to `oxc_codegen` (#6711) (DonIsaac)

## [0.32.0] - 2024-10-19

- c0e9d7e codegen: [**BREAKING**] `Codegen::into_source_text` consume `Codegen` (#6539) (overlookmotel)

- 782f0a7 codegen: [**BREAKING**] Rename `print_char` method to `print_ascii_byte` (#6512) (overlookmotel)

- 91c87dd codegen: [**BREAKING**] Remove `Codegen::enableSourceMap` API (#6452) (Boshen)

- 7645e5c codegen: [**BREAKING**] Remove CommentOptions API (#6451) (Boshen)

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

### Features

- e5ed6a5 codegen: Print negative numbers (#6624) (Boshen)

### Bug Fixes

- ba385fc codegen: Panic occurred when printing the comment of the right parenthesis (#6593) (Dunqing)
- 02bfbfe codegen: Preserve parenthesis for `ChainExpression` (#6430) (Dunqing)
- 2ade16e codegen: Invalid codegen when `in` inside bin expr in or loop (#6431) (camc314)
- 6896efc codegen: Fix `in` in sequence expr in for loops (#6428) (camc314)

### Performance

- 77f3a1a codegen: Check last char with byte methods (#6509) (overlookmotel)
- 18b68ff codegen: Optimize `CodeBuffer::print_ascii_byte` (#6516) (overlookmotel)

### Documentation

- 7e909a7 codegen: Fix example for `CodeBuffer::print_ascii_bytes` (#6535) (overlookmotel)
- 235d357 codegen: Improve doc comments for `CodeBuffer` (#6511) (overlookmotel)
- c8fa2eb codegen: Correct and reformat doc comments for `CodeBuffer` (#6504) (overlookmotel)
- 40d1ee4 codegen: Fix and reformat `CodeBuffer` examples (#6499) (overlookmotel)

### Refactor

- 51fc63d codegen: Rename `CodeBuffer::print_bytes_unchecked` method (#6517) (overlookmotel)
- 05a2ebd codegen: Reorder dependencies in `Cargo.toml` (#6514) (overlookmotel)
- e7f3e28 codegen: Rename var in `CodeBuffer` (#6510) (overlookmotel)
- 1bbd383 codegen: Rename `CodeBuffer::print_ascii_bytes` method (#6507) (overlookmotel)
- cd9fe9e codegen: Rename vars in `CodeBuffer` methods (#6506) (overlookmotel)
- fc536a5 codegen: Inline `CodeBuffer` methods (#6501) (overlookmotel)
- 7420620 codegen: Add `CodeBuffer::as_bytes` method (#6498) (overlookmotel)
- 8ae174b codegen: Rename `CodeBuffer::print_byte_unchecked` method (#6496) (overlookmotel)
- 5843e01 codegen: Shorten `CodeBuffer::take_source_text` (#6495) (overlookmotel)
- 951def6 codegen: Clarify safety comments in `CodeBuffer` (#6494) (overlookmotel)
- 84a51ee codegen: Rename vars in `CodeBuffer` (#6493) (overlookmotel)
- 05bd616 codegen: Add line break (#6492) (overlookmotel)
- 204bf55 codegen: Add `CodeBuffer` to fix soundness hole (#6148) (DonIsaac)
- 702b574 codegen: Only print necessary parentheses in TSAsExpression (#6429) (Dunqing)
- f4cdc56 minifier: Use constant folding unary expression from `oxc_ecmascript` (#6647) (Boshen)
- 1a90ec4 rust: Backport v1.82.0 changes to main branch first (#6690) (Boshen)

### Testing

- e7c89a5 codegen: Uncomment passed tests that are related to trailing comments (#6589) (Dunqing)
- d816b0b codegen: Add test for `CodeBuffer::print_byte_unchecked` (#6497) (overlookmotel)

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

### Bug Fixes

- 84b2d07 codegen: Converts line comment to block comment if it is a `PURE` comment (#6356) (Dunqing)

### Refactor


## [0.30.5] - 2024-09-29

### Refactor

- ab187d1 codegen: Restrict visibility of internal methods (#6145) (DonIsaac)

## [0.30.4] - 2024-09-28

### Bug Fixes

- 8582ae3 codegen: Missing parentheses if there is a pure comment before a NewExpression as a ComputedMemberExpression's callee (#6105) (Dunqing)

### Performance

- 05852a0 codegen: Do not check whether there are annotation comments or not if we don't preserve annotation comments (#6107) (Dunqing)

## [0.30.2] - 2024-09-27

### Features

- cca433f codegen: Print `vite` / `webpack` special comments (#6021) (Dunqing)

### Bug Fixes

- c8682e9 semantic,codegen,transformer: Handle definite `!` operator in variable declarator (#6019) (Boshen)

### Refactor

- fe696f0 codegen: Simplify printing annotation comments (#6027) (Dunqing)

## [0.30.1] - 2024-09-24

### Bug Fixes

- 9ca202a codegen: Preserve newlines between comments (#6014) (Boshen)
- 4a99372 codegen: Print jsdoc comments for `TSEnumMember`s (#6007) (camc314)

## [0.30.0] - 2024-09-23

### Features

- d901772 codegen: Implement minify number from terser (#5929) (Boshen)
- 9f6696a codegen: Add new lines to `TSTypeParameterDeclaration` (#5853) (Boshen)
- bcdbba3 codegen: Print jsdoc comments that are attached to statements and class elements (#5845) (Boshen)
- 26386da codegen: Have `with_source_text` reserve memory for code buffer (#5823) (DonIsaac)
- dfbde2c isolated_declarations: Print jsdoc comments (#5858) (Boshen)

### Bug Fixes

- f4aefb5 codegen: Print `let[0]` as `(let)[0]` (#5947) (Boshen)
- cee9d0b codegen: Fix spacing of `for await (x of y)` (#5890) (Boshen)
- 5901d2a codegen: Various spacing issues (#5820) (Boshen)
- 362c427 mangler,codegen: Do not mangle top level symbols (#5965) (Boshen)
- 42dcadf parser: Hashbang comment should not keep the end newline char (#5844) (Boshen)

### Refactor

- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)
- bb95306 codegen: Change annotation comment tests to snapshot (#5800) (Boshen)
- e613a3d codegen: Prepare to add leading comments by adding a template method pattern (#5784) (Boshen)
- 7caae5b codegen: Add `GetSpan` requirement to `Gen` trait (#5772) (Boshen)

## [0.29.0] - 2024-09-13

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

### Features


### Bug Fixes

- d62defb codegen: Do not print trailing commas for `ArrayExpression` (#5551) (Boshen)

### Performance


## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

- 87c5df2 ast: [**BREAKING**] Rename `Expression::without_parentheses` (#5448) (overlookmotel)

### Features

- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)
- c782916 codegen: Print `type_parameters` in `TaggedTemplateExpression` (#5438) (Dunqing)

### Bug Fixes

- 0df1d9d ast, codegen, linter: Panics in fixers. (#5431) (rzvxa)

### Refactor

- d9d7e7c ast: Remove `IdentifierName` from `TSThisParameter` (#5327) (overlookmotel)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

### Features

- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 292d162 codegen: Print missing fields for `AccessorProperty` (#5291) (Dunqing)

### Bug Fixes

- 5c4c001 codegen: Print `export @decorator declare abstract class Foo` correctly (#5303) (Boshen)
- 7b1546b codegen: Do not print comments when `--minify` (Boshen)

### Performance

- 12a7607 codegen: Inline `Codegen::print_list` (#5221) (overlookmotel)
- fb847bd codegen: Slightly faster `print_list` (#5192) (Boshen)

### Refactor

- d4c3778 codegen: Rename vars (#5222) (overlookmotel)
- 543cad6 codegen: Remove some pub APIs (Boshen)

## [0.25.0] - 2024-08-23

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features


### Bug Fixes
- b7db235 Comments gen regression (#5003) (IWANABETHATGUY)

### Refactor

- cd9cf5e oxc: Remove `remove_whitespace` (Boshen)

## [0.24.3] - 2024-08-18

### Features

- d49fb16 oxc_codegen: Support generate range leading comments (#4898) (IWANABETHATGUY)

### Bug Fixes

- bbf9ec0 codegen: Add missing `declare` to `PropertyDefinition` (#4937) (Boshen)
- f210cf7 codegen: Print `TSSatisfiesExpression` and `TSInstantiationExpression` (#4936) (Boshen)
- 21f5762 codegen: Minify large numbers (#4889) (Boshen)
- e8de4bd codegen: Fix whitespace issue when minifying `x in new Error()` (#4886) (Boshen)
- a226962 codegen: Print `TSNonNullExpression` (#4869) (Boshen)
- 3da33d3 codegen: Missing parenthesis for `PrivateInExpression` (#4865) (Boshen)
- 1808529 codegen: Dedupe pure annotation comments (#4862) (IWANABETHATGUY)
- 508644a linter/tree-shaking: Correct the calculation of `>>`, `<<` and `>>>` (#4932) (mysteryven)

## [0.24.0] - 2024-08-08

### Bug Fixes

- 4a56954 codegen: Print raw if value is number is Infinity (#4676) (Boshen)
- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Performance

- 8dd76e4 codegen: Reduce size of `LineOffsetTable` (#4643) (overlookmotel)
- b8e6753 codegen: `u32` indexes in `LineOffsetTable` for source maps (#4641) (overlookmotel)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.1] - 2024-08-06

### Bug Fixes

- 4a56954 codegen: Print raw if value is number is Infinity (#4676) (Boshen)
- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Performance

- 8dd76e4 codegen: Reduce size of `LineOffsetTable` (#4643) (overlookmotel)
- b8e6753 codegen: `u32` indexes in `LineOffsetTable` for source maps (#4641) (overlookmotel)

### Refactor

- e78cba6 minifier: Ast passes infrastructure (#4625) (Boshen)

## [0.23.0] - 2024-08-01

- 27fd062 sourcemap: [**BREAKING**] Avoid passing `Result`s (#4541) (overlookmotel)

### Features

- a558492 codegen: Implement `BinaryExpressionVisitor` (#4548) (Boshen)
- 7446e98 codegen: Align more esbuild implementations (#4510) (Boshen)
- 35654e6 codegen: Align operator precedence with esbuild (#4509) (Boshen)

### Bug Fixes

- b58ed80 codegen: Enable more test cases (#4585) (Boshen)
- 6a94e3f codegen: Fixes for esbuild test cases (#4503) (Boshen)
- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Performance

- 7585e16 linter: Remove allocations for string comparisons (#4570) (DonIsaac)

### Refactor


## [0.22.0] - 2024-07-23

### Bug Fixes

- 44a10c4 codegen: Object shorthand with parens `({x: (x)})` -> `({ x })` (#4391) (Boshen)
- 3d88f20 codegen: Print shorthand for all `{ x }` variants (#4374) (Boshen)
- e624dff codegen,mangler: Do not print shorthand for `ObjectProperty` (#4350) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 83c2c62 codegen: Add option for choosing quotes; remove slow `choose_quot` method (#4219) (Boshen)
- e3e663b mangler: Initialize crate and integrate into minifier (#4197) (Boshen)

### Bug Fixes

- bf3d8d3 codegen: Print annotation comment inside parens for new and call expressions (#4290) (Boshen)
- 084ab76 codegen: Use `ryu-js` for f64 to string (Boshen)
- e167ef7 codegen: Print parenthesis properly (#4245) (Boshen)
- c65198f codegen: Choose the right quote for jsx attribute string (#4236) (Boshen)
- be82c28 codegen: Print `JSXAttributeValue::StringLiteral` directly (#4231) (Boshen)
- 3df9e69 mangler: No shorthand `BindingProperty`; handle var hoisting and export variables (#4319) (Boshen)
- 66b455a oxc_codegen: Avoid print same pure comments multiple time (#4230) (IWANABETHATGUY)- 1c117eb Avoid print extra semicolon after accessor property (#4199) (IWANABETHATGUY)

### Refactor

- d1c4be0 codegen: Clean up annotation_comment (Boshen)
- 06197b8 codegen: Separate tests (Boshen)
- aa22073 codegen: Improve print API (#4196) (Boshen)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Refactor


## [0.18.0] - 2024-07-09

### Features

- 365d9ba oxc_codegen: Generate annotation comments before `CallExpression` and `NewExpression` (#4119) (IWANABETHATGUY)

## [0.17.2] - 2024-07-08

### Bug Fixes

- 5472b7c codegen: 256 indentations level is not enough for codegen (Boshen)

## [0.17.1] - 2024-07-06

### Bug Fixes

- 564a75a codegen: Missing TypeParameters in TSConstructSignature (#4063) (michaelm)

## [0.17.0] - 2024-07-05

### Features

- 7768d23 isolated-declarations: Support optional class methods (#4035) (Egor Blinov)

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)
- 5e5b1b1 codegen: Correct accessibility emit for class formal-parameters/methods/properties (#4042) (Egor Blinov)
- 7844734 codegen: Missing const keyword in TSTypeParamter (#4022) (Dunqing)
- 6254a41 codegen: Missing TypeParamters in TSCallSignature (#4021) (Dunqing)

## [0.16.3] - 2024-07-02

### Bug Fixes

- 23038ad codegen: Print `TSFunctionType` inside `TSTypeAssertion` (#3999) (Boshen)

## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)

### Bug Fixes

- dac617d codegen: Print some missing typescript attributes (#3980) (Boshen)

## [0.16.1] - 2024-06-29

### Bug Fixes

- 51e54f9 codegen: Should print `TSModuleDeclarationKind` instead of just `module` (#3957) (Dunqing)

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 5847e16 ast,parser: Add `intrinsic` keyword (#3767) (Boshen)
- 01da2f7 codegen: Print TSThisParameter for TSCallSignatureDeclaration and TSMethodSignature (#3792) (Dunqing)
- 2821e0e codegen: Print readonly keyword for TSIndexSignature (#3791) (Dunqing)
- 97575d8 codegen: Print TSClassImplements and TSThisParameter (#3786) (Dunqing)

### Bug Fixes

- 2766594 codegen: Print type parameters for MethodDefinition (#3922) (Dunqing)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 4cf3c76 parser: Improve parsing of TypeScript types (#3903) (Boshen)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 534242a codegen: [**BREAKING**] Remove `CodegenOptions::enable_typescript` (#3674) (Boshen)

- 0578ece ast: [**BREAKING**] Remove `ExportDefaultDeclarationKind::TSEnumDeclaration` (#3666) (Dunqing)

### Features

- 5a99d30 codegen: Improve codegen formatting (#3735) (Boshen)
- bf9b38a codegen: Improve codegen formatting (#3731) (Boshen)
- 4a004e2 codegen: Print TSImport remaining fields (#3695) (Dunqing)
- a56cb1b codegen: Print accessibility for MethodDefinition (#3690) (Dunqing)
- 38a75e5 coverage: Improve codegen (#3729) (Boshen)
- 4f2db46 transformer-dts: `--isolatedDeclarations` dts transform (#3664) (Dunqing)

### Bug Fixes

- da1e2d0 codegen: Improve typescript codegen (#3708) (Boshen)

### Refactor

- fa7a6ba codegen: Add `gen` method to ast nodes (#3687) (Boshen)
- 09b92b6 codegen: Move `gen_ts` into `gen` to make searching things easier (#3680) (Boshen)
- 815260e isolated-declarations: Decouple codegen (#3715) (Boshen)
- 4f16664 transformer_dts: Create a `Program` for codegen (#3679) (Boshen)

## [0.14.0] - 2024-06-12

### Refactor

- f98f777 linter: Add rule fixer (#3589) (Don Isaac)

## [0.13.4] - 2024-06-07

### Features

- 5c8e16c coverage: Second transformer build does not print typescript (#3561) (Dunqing)

### Bug Fixes

- affb2c8 codegen: Print indentation before directive (#3512) (Dunqing)

## [0.13.3] - 2024-06-04

### Bug Fixes

- 98c9029 codegen: Should be double quote for jsx attribute value (#3516) (Dunqing)
- d8063b6 codegen: Wrong escape string (#3514) (Dunqing)

### Refactor

- ddac2a0 codegen: Reduce allocation for print_unquoted_str (#3525) (Dunqing)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

### Bug Fixes

- a12ed0f codegen: Using declaration in for statement (#3285) (Don Isaac)

### Refactor

- 9ced605 parser: Start porting arrow function parsing from tsc (#3340) (Boshen)
- e879685 sourcemap: Using binary search to search original position (#3360) (underfin)

## [0.13.0] - 2024-05-14

### Features

- eefb66f ast: Add type to AccessorProperty to support TSAbractAccessorProperty (#3256) (Dunqing)

### Bug Fixes

- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 0185eb2 ast: Remove duplicate `TSNamedTupleMember` representation (#3101) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)

## [0.12.5] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)

## [0.12.4] - 2024-04-19

### Features

- fd5002b codegen: Correctly print type-only imports/exports (#2993) (Dunqing)

## [0.12.1] - 2024-04-03

### Bug Fixes

- 28fae2e sourcemap: Using serde_json::to_string to quote sourcemap string (#2889) (underfin)

### Refactor

- 114f68e codegen: Make codegen sourcemap builder clearer (#2894) (underfin)

## [0.11.0] - 2024-03-30

### Features

- 243131d transformer: Numeric separator plugin. (#2795) (Ali Rezvani)- b199cb8 Add oxc sourcemap crate (#2825) (underfin)- a2cfc86 SourcemapVisualizer (#2773) (underfin)

### Bug Fixes

- 6177c2f codegen: Sourcemap token name should be original name (#2843) (underfin)
- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)

### Performance

- 2be5f9d codegen: Avoid unnecessary copy (#2727) (underfin)
- d7004da sourcemap: Remove unnecessary binary search (#2728) (underfin)

### Refactor

- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.10.0] - 2024-03-14

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

### Bug Fixes

- 9609c34 codegen: `CallExpression` sourcemap (#2717) (underfin)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

## [0.9.0] - 2024-03-05

- f66059e ast: [**BREAKING**] Align TSImportType with ESTree (#2578) (Arnaud Barré)

### Features

- 20c7bf7 ast: Add `AssignmentTargetRest` (#2601) (Boshen)
- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- 8bb1084 codegen: Add sourcemap (#2565) (Boshen)

### Bug Fixes

- ea30fd5 codegen: Fix adding mapping to sourcemaps (#2590) (overlookmotel)
- fe29fa4 codegen: Correct sourcemaps when Windows line breaks + unicode (#2584) (overlookmotel)
- 517026b codegen: Correct sourcemaps when unicode chars (#2583) (overlookmotel)

### Performance

- b7f5c63 codegen: Speed up generating sourcemap mappings (#2597) (overlookmotel)
- 42fa8eb codegen: Speed up building sourcemap line tables (#2591) (overlookmotel)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)

### Bug Fixes

- 4327916 codegen: Remove redundant semicolon in PropertyDefinition (#2511) (Dunqing)
- b5deb9a codegen: When `async` is on the left-hand side of a for-of, wrap it in parentheses (#2407) (Dunqing)
- 384d5ac codegen: Lower the level of precedence in TaggedTemplateExpression (#2391) (Wenzhe Wang)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.7.0] - 2024-02-09

### Features

- 55011e2 codegen: Avoid printing comma in ArrayAssignmentTarget if the elements is empty (#2331) (Dunqing)

### Bug Fixes

- 2eb489e codegen: Format new expession + import expression with the correct parentheses (#2346) (Dunqing)
- 721f6cb codegen: Format new expression + call expression with the correct parentheses (#2330) (Boshen)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 8ac0202 codegen: Keep shorthand in ObjectPattern and ObjectProperty (#2265) (Dunqing)
- fa555ce codegen: Change back to read raw (#2222) (Wenzhe Wang)
- 9333264 codegen: Print TemplateLiteral with `print_str` (#2207) (Wenzhe Wang)
- 1ee6d8c codegen: Move string test to codegen (#2150) (Wenzhe Wang)

### Bug Fixes

- 0c225a4 codegen: Print space before with clause in import (#2278) (Wenzhe Wang)
- d34650a codegen: Print necessary spaces for `ExportAllDeclaration` (#2190) (Yunfei He)
- 989ab88 codegen: Print `Directive` original string (#2157) (underfin)
- 29dc5e6 codegen: Add parenthesis in binary expression by precedence (#2067) (Wenzhe Wang)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)

## [0.5.0] - 2024-01-12

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)

## [0.4.0] - 2023-12-08

### Features

- 9ff0ffc ast: Implement new proposal-import-attributes (#1476) (magic-akari)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- cef78ac codegen: Indent inner class (#1085) (Wenzhe Wang)
- 854b55a codegen: Json strings proposal (#1039) (Boshen)
- 6c18b3e codegen: Beauty class print (#995) (Wenzhe Wang)
- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 809f050 codegen: Move minifying printer to codegen crate (#985) (Boshen)
- f28d96c codegen: Initialize the codegen crate and struct (#983) (Boshen)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- e8a4e81 transformer: Implement some of jsx decode entities (#1086) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)- 094dfa5 Support filter exec snap (#1084) (Wenzhe Wang)- 0e91044 Adjust the order of print semicolon (#1003) (Wenzhe Wang)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- f32bf27 codegen: Fix some typescript codegen problems (#989) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)

### Refactor

- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 110059f rust: Change `RefCell.clone().into_inner()` to `RefCell.get()` (Boshen)

