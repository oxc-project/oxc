# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.17.0] - 2024-07-05

- e32b4bc ast: [**BREAKING**] Store trivia comments in a sorted slice (#4045) (Luca Bruno)

- 1df6ac0 ast: [**BREAKING**] Rename `visit_enum_memeber` to `visit_ts_enum_member`. (#4000) (rzvxa)

- 4a0eaa0 ast: [**BREAKING**] Rename `visit_enum` to `visit_ts_enum_declaration`. (#3998) (rzvxa)

- c98d8aa ast: [**BREAKING**] Rename `visit_arrow_expression` to `visit_arrow_function_expression`. (#3995) (rzvxa)

### Features

- 1854a52 ast_codegen: Introduce the `#[span]` hint. (#4012) (rzvxa)
- 7538af1 ast_codegen: Add visit generator (#3954) (rzvxa)

### Bug Fixes

- 05a047c isolated-declarations: Method following an abstract method gets dropped (#4024) (Dunqing)

### Refactor

- b51f75b ast_codegen: No longer outputs discard variable for empty visitors. (#4008) (rzvxa)

## [0.16.3] - 2024-07-02

### Features

- b257d53 linter: Support report `@typescript-eslint/consistent-type-imports` (#3895) (mysteryven)

### Bug Fixes

- d995f94 semantic: Resolve reference incorrectly when a parameter references a parameter that hasn't been defined yet (#4004) (Dunqing)

### Refactor

- 0fe22a8 ast: Reorder fields to reflect their visit order. (#3994) (rzvxa)

## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)

## [0.16.1] - 2024-06-29

### Bug Fixes

- 31e4c3b isolated-declarations: `declare global {}` should be kept even if it is not exported (#3956) (Dunqing)

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

- 497769c ast: Add some visitor functions (#3785) (Dunqing)
- 4b06dc7 ast: Add TSType::TSIntrinsicKeyword to is_keyword (#3775) (Dunqing)
- 5847e16 ast,parser: Add `intrinsic` keyword (#3767) (Boshen)
- 2e026e1 ast_codegen: Generate `ast_kind.rs`. (#3888) (rzvxa)
- 09f4d3c ast_codegen: Add `ImplGetSpanGenerator`. (#3852) (rzvxa)
- d5f6aeb semantic: Check for illegal symbol modifiers (#3838) (Don Isaac)

### Bug Fixes

- 063cfde ast: Correct JSON serialization of `TSModuleBlock` (#3858) (overlookmotel)
- 66f404c ast: Fix JSON serialization of `BindingPattern` (#3856) (overlookmotel)
- 59ce38b isolated-declarations: Inferring of UnrayExpression incorrectly (#3920) (Dunqing)
- 8c9fc63 semantic: Apply strict mode scope flag for strict mode TS Modules (#3861) (overlookmotel)
- aea3e9a transformer: Correct spans for TS annotations transform (#3782) (overlookmotel)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 6f26087 ast: Add comment about alternatives to `AstBuilder::copy` (#3905) (overlookmotel)
- 442aca3 ast: Add comment not to use `AstBuilder::copy` (#3891) (overlookmotel)
- acf69fa ast: Refactor custom `Serialize` impls (#3859) (overlookmotel)
- 9e148e9 ast: Add line breaks (#3860) (overlookmotel)
- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- a648748 ast: Shorten code in AST builder (#3835) (overlookmotel)
- 1206967 ast: Reduce allocations in AST builder (#3834) (overlookmotel)
- 4cf3c76 parser: Improve parsing of TypeScript types (#3903) (Boshen)
- 97d59fc parser: Move code around for parsing `Modifiers` (#3849) (Boshen)
- 1061baa traverse: Separate `#[scope]` attr (#3901) (overlookmotel)
- fcd21a6 traverse: Indicate scope entry point with `scope(enter_before)` attr (#3882) (overlookmotel)
- 2045c92 traverse: Improve parsing attrs in traverse codegen (#3879) (overlookmotel)

## [0.15.0] - 2024-06-18

- 0578ece ast: [**BREAKING**] Remove `ExportDefaultDeclarationKind::TSEnumDeclaration` (#3666) (Dunqing)

### Features

- 81e9526 isolated-declarations: Inferring set accessor parameter type from get accessor return type (#3725) (Dunqing)
- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)
- 046ff3f linter/eslint: Add `no_unreachable` rule. (#3238) (rzvxa)
- 910193e transformer-dts: Report error for super class (#3711) (Dunqing)
- 413d7be transformer-dts: Transform enum support (#3710) (Dunqing)
- 35c382e transformer-dts: Remove type annotation from private field (#3689) (Dunqing)
- 0e6d3ce transformer-dts: Report error for async function and generator (#3688) (Dunqing)
- b22b59a transformer-dts: Transform namespace support (#3683) (Dunqing)
- 4f2db46 transformer-dts: `--isolatedDeclarations` dts transform (#3664) (Dunqing)

### Bug Fixes

- 2158268 ast: Incorrect visit order in function (#3681) (Dunqing)
- da1e2d0 codegen: Improve typescript codegen (#3708) (Boshen)
- 90743e2 traverse: Change visit order for `Function` (#3685) (overlookmotel)

### Refactor

- fa7a6ba codegen: Add `gen` method to ast nodes (#3687) (Boshen)

## [0.14.0] - 2024-06-12

### Features

- f6d9ca6 linter: Add `eslint/sort-imports` rule (#3568) (Wang Wenzhe)

### Bug Fixes

- f8f6d33 ast: Correct `visited_node` attr for strict mode of arrow fns (#3635) (overlookmotel)

### Performance

- 3a59294 transformer: React display name transform reduce Atom allocations (#3616) (overlookmotel)

### Refactor

- 0f92521 ast: Replace recursion with loop (#3626) (overlookmotel)
- 08f1010 ast: Make `AstBuilder` `Copy` (#3602) (overlookmotel)
- f98f777 linter: Add rule fixer (#3589) (Don Isaac)
- 89bcbd5 transformer: Move `BoundIdentifier` into helpers (#3610) (overlookmotel)

## [0.13.4] - 2024-06-07

### Features

- a939ddd transformer/typescript: Remove more typescript ast nodes (#3563) (Dunqing)
- e8a20f8 transformer/typescript: Remove typescript ast nodes (#3559) (Dunqing)

## [0.13.2] - 2024-06-03

### Features

- 0d2c977 linter: Add `oxc/no-const-enum` rule (#3435) (Wang Wenzhe)

### Bug Fixes

- ea53267 ast: UsingDeclaration is not a typescript syntax (#3482) (Dunqing)

### Refactor

- ff7e8c7 ast: Update scope attrs (#3494) (overlookmotel)
- 55bbde2 ast: Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488) (overlookmotel)
- 9c3d163 ast: Rename function params (#3487) (overlookmotel)
- 286b5ed ast: Remove defunct hashing of `Span` (#3486) (overlookmotel)

## [0.13.1] - 2024-05-22

### Bug Fixes

- 9594441 linter/react: `rules_of_hooks` add support for property hooks/components. (#3300) (rzvxa)- 899a52b Fix some nightly warnings (Boshen)

### Performance

- 33386ef ast: Inline all `ASTBuilder` methods (#3295) (Boshen)

### Refactor

- 938ae12 ast: Fix clippy lint on nightly (#3346) (overlookmotel)
- 723a46f ast: Store `ScopeId` in AST nodes (#3302) (overlookmotel)
- 89a1f97 parser: Improve expression parsing (#3352) (Boshen)

## [0.13.0] - 2024-05-14

### Features

- eefb66f ast: Add type to AccessorProperty to support TSAbractAccessorProperty (#3256) (Dunqing)
- ac1a40f ast: Add `callee_name` method to the `CallExpression`. (#3076) (Ali Rezvani)
- 870d11f syntax: Add `ToJsString` trait for f64 (#3131) (Boshen)
- be87ca8 transform: `oxc_traverse` crate (#3169) (overlookmotel)
- 34dd53c transformer: Report ambient module cannot be nested error (#3253) (Dunqing)
- 78875b7 transformer: Implement typescript namespace (#3025) (Boshen)
- be8fabe transformer/react: Enable jsx plugin when development is true (#3141) (Dunqing)
- 46c02ae traverse: Add scope flags to `TraverseCtx` (#3229) (overlookmotel)

### Bug Fixes

- 81f90fd ast: Do not include `trailing_comma` in JSON AST (#3073) (overlookmotel)
- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)
- 65540c0 traverse: Set `ScopeFlags::Function` bit for class methods (#3277) (overlookmotel)
- 6fd7a3c traverse: Create scopes for functions (#3273) (overlookmotel)
- 4e20b04 traverse: Create scope for function nested in class method (#3234) (overlookmotel)

### Documentation

- c6bd616 ast: Document enum inheritance (#3192) (overlookmotel)

### Refactor

- 4208733 ast: Order AST type fields in visitation order (#3228) (overlookmotel)
- c84c116 ast: Add `is_strict` methods (#3227) (overlookmotel)
- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 0185eb2 ast: Remove duplicate `TSNamedTupleMember` representation (#3101) (overlookmotel)
- 942b2ba ast: Add array element `Elision` type (#3074) (overlookmotel)
- f5dccc9 coverage: Avoid an `String::from_utf8` over head during serialization (#3145) (Boshen)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- 5329b0f transform: Fix doc comments for methods generated by `inherit_variants!` macro (#3195) (overlookmotel)

## [0.12.5] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)
- 383b449 ast: Box `ImportDeclarationSpecifier` enum variants (#3061) (overlookmotel)
- 2804e7d ast: Reduce indirection in AST types (#3051) (overlookmotel)

### Refactor

- 1249c6c ast: Implement same traits on all fieldless enums (#3031) (overlookmotel)
- 0b9470e ast: Shorten code (#3027) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- bd9fc6d transformer: React jsx transform (#2961) (Boshen)
- e673550 transformer: Start on TypeScript annotation removal (#2951) (Miles Johnson)
- f903a22 transformer: Implement react-jsx-self (#2946) (Boshen)
- 0c04bf7 transformer: Transform TypeScript namespace (#2942) (Boshen)
- e14ac17 transformer/typescript: Insert this assignment after the super call (#3018) (Dunqing)

## [0.12.3] - 2024-04-11

### Features

- 6c00908 oxc_ast: Add missing ast visits for types (#2938) (Brad Zacher)

### Refactor

- 5974819 ast: Clean up the ts type visit methods (Boshen)

## [0.12.1] - 2024-04-03

### Bug Fixes

- 5f8f7f8 ast: `FinallyClause` won't get visited as `BlockStatement` anymore. (#2881) (Ali Rezvani)

## [0.11.0] - 2024-03-30

### Bug Fixes

- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- 198eea0 ast: Add walk functions to Visit trait. (#2791) (Ali Rezvani)
- 813226b ast: Get rid of unsafe transmutation in VisitMut trait. (#2764) (Ali Rezvani)

## [0.10.0] - 2024-03-14

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

- 7768123 parser: [**BREAKING**] Drop TSImportEqualsDeclaration.is_export (#2654) (Arnaud Barré)

### Features

- 0d7bc8f ast: Fill in missing ast visits (#2705) (Boshen)
- 8e3e404 prettier: Print `with_clause` in reexport declaration (#2635) (magic-akari)
- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)
- 308b780 transformer/decorators: Handling the coexistence of class decorators and member decorators (#2636) (Dunqing)- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

### Bug Fixes

- c820a5b ast: Serialize empty array elements as null (#2707) (overlookmotel)
- acf127b ast: Correct TS type for `ArrayAssignmentTarget` (#2699) (overlookmotel)
- 3305734 ast: Add `type` field to TS types for `ObjectPattern` etc (#2670) (overlookmotel)
- f27db30 ast: Fix TS type for `AssignmentTargetRest` (#2668) (overlookmotel)
- d47f0e2 ast: Rename `TSIndexSignatureName` in JSON AST (#2664) (overlookmotel)
- cc5be63 ast: Fix serializing rest elements (#2652) (overlookmotel)
- 88f94bb ast: Add `RestElement`s in serialized AST to elements array (#2567) (overlookmotel)
- 2a235d3 ast: Parse `with_clause` in re-export declaration (#2634) (magic-akari)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)
- cba1e2f ast: Import `Tsify` to shorten code (#2665) (overlookmotel)
- a01cf9f ast: Remove `Serialize` impls for Identifier types (#2651) (overlookmotel)
- 6b5723c ast: Shorten manual TS defs (#2638) (overlookmotel)- 89e8d15 Derive `SerAttrs` on all AST types (#2698) (overlookmotel)- 3c1e0db Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) (overlookmotel)- d76ee6b "wasm" feature enable "serde" feature (#2639) (overlookmotel)

## [0.9.0] - 2024-03-05

- f66059e ast: [**BREAKING**] Align TSImportType with ESTree (#2578) (Arnaud Barré)

### Features

- 1db307a ast: Serialize `BindingPattern` to estree (#2610) (Boshen)
- f6709e4 ast: Serialize identifiers to ESTree (#2521) (Arnaud Barré)
- 20c7bf7 ast: Add `AssignmentTargetRest` (#2601) (Boshen)
- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)

### Bug Fixes

- 49778ab ast: Temporary fix tsify not generating some typings (#2611) (Boshen)
- 1d65713 ast: Expose NumericLiteral.raw (#2588) (Arnaud Barré)
- 637cd1d ast: Support TSIndexSignature.readonly (#2579) (Arnaud Barré)
- 258b9b1 ast: Support FormalParameter.override (#2577) (Arnaud Barré)
- 78f30bc ast: Change TSMappedType.type_annotation from TSTypeAnnotation to TSType (#2571) (Arnaud Barré)
- e339461 ast: Rename serialized fields to camel case (#2566) (overlookmotel)
- fd8f735 ast: Missing visit JSXElementName enum (#2547) (Dunqing)
- d181209 ast: Add Function to generated TS types and fix ModifierKind serialization (#2534) (Arnaud Barré)
- 6d5ec6d ast: Few serialization issues (#2522) (Arnaud Barré)
- f00834d linter: Fix getter return rule false positives in TypeScript (#2543) (BlackSoulHub)
- d9cc429 parser: Parse empty method declaration as TSEmptyBodyFunctionExpression (#2574) (Arnaud Barré)
- 1519b90 semantic: Incorrect scope for switch statement (#2513) (Dunqing)

## [0.8.0] - 2024-02-26

### Features

- 70295a5 ast: Update arrow_expression to arrow_function_expression (#2496) (Dunqing)
- 7a796c4 ast: Add `TSModuleDeclaration.kind` (#2487) (Boshen)
- f5aadc7 linter: Handle cjs `module.exports = {} as default export (#2493) (Boshen)
- f64c7e0 linter: Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492) (Boshen)
- 60db720 parser: Parse import attributes in TSImportType (#2436) (Dunqing)
- 642484e prettier: Print newlines between array expression elements (#2379) (Boshen)
- 3d008ab transformer/decorators: Insert instanceBrand function (#2480) (Dunqing)

### Bug Fixes

- 871a73a prettier: Semi colon after class property (#2387) (Boshen)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)
- 3cbe786 ast: Update TSImportType parameter to argument (#2429) (Dunqing)

## [0.7.0] - 2024-02-09

### Features

- d571839 ast: Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317) (Dunqing)
- a3570d4 semantic: Report parameter related errors for setter/getter (#2316) (Dunqing)

### Bug Fixes

- 2eb489e codegen: Format new expession + import expression with the correct parentheses (#2346) (Dunqing)
- b5e43fb linter: Fix no_dupe_keys false postive on similar key names (#2291) (Boshen)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- 165f948 ast: Remove expression property from Function (#2247) (Dunqing)
- f673e41 ast: Remove serde skip for symbol_id and reference_id (#2220) (Dunqing)
- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)
- 080e78c ast: Complete AccessorProperty todo in has_decorator (#2178) (Dunqing)
- ac4b3a4 ast: Visit TSTypeQuery (#2021) (Dunqing)
- d71175e semantic: Check optional parameters (#2263) (Dunqing)
- 8898377 semantic: Cfg prototype (#2019) (Boshen)
- 9e598ff transformer: Add decorators plugin (#2139) (Dunqing)
- 02c18d8 transformer/decorators: Support for static and private member decorators (#2246) (Dunqing)
- ba85b09 transformer/decorators: Support method decorator and is not static (#2238) (Dunqing)
- e5719e9 transformer/decorators: Support transform member decorators (#2171) (Dunqing)
- 7f89bfe transformer/decorators: Support version 2023-05 (#2152) (Dunqing)
- 04b401c transformer/decorators: Support transform the class decorators in export declaration (#2145) (Dunqing)
- 56ca8b6 transformer/typescript: Support transform namespace (#2075) (Dunqing)
- 3413bb3 transformer/typescript: Remove type-related exports (#2056) (Dunqing)

### Bug Fixes

- ea8cc98 ast: AcessorProperty is missing decorators (#2176) (Dunqing)
- 2f5afff parser: Fix crash on TSTemplateLiteralType in function return position (#2089) (Boshen)

### Refactor

- b261e86 ast: Improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153) (Dunqing)
- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)
- 1de3518 linter: Remove Regex and change error position (#2188) (Wenzhe Wang)
- 2924258 semantic: Adding binder for ImportSpecifier replaces the ModuleDeclaration's binder (#2230) (Dunqing)
- f59e87f semantic: Checking label in ContinueStatement based on LabelBuilder (#2202) (Dunqing)
- 8bccdab semantic: Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114) (Dunqing)
- de6d2f5 transformer/decorators: Optimizing code with ast.private_field (#2249) (Dunqing)

## [0.5.0] - 2024-01-12

### Features

- 0a08686 ast: Visit TSModuleReference (#1998) (Dunqing)
- d41e3fd ast: Enter/leave ClassBody and PrivateInExpression (#1792) (Dunqing)
- 67b7cc0 ast: Support visit more jsx ast in visit (#1662) (Dunqing)
- c1cfd17 linter: No-irregular-whitespace rule (#1835) (Deivid Almeida)
- f45a3cc linter: Support eslint/no-unused-private-class-members rule (#1820) (Dunqing)
- 0c19991 prettier: Print CallExpression arguments correctly (#1631) (Dunqing)
- ca04312 semantic: Add ClassTable (#1793) (Dunqing)

### Bug Fixes

- adfe24e ast: Implement `GetSpan` for `JSXElement` (#1861) (Qix)- 0d77e1e Default visitor should visit prop init at `visit_object_property` (#2000) (underfin)

### Refactor

- a2858ed ast: Introduce `ThisParameter` (#1728) (magic-akari)
- 08438e0 parser: Remove TokenValue::RegExp from `Token` (#1926) (Boshen)

## [0.4.0] - 2023-12-08

### Features

- 4043ca9 ast: Add enter node and scope for `VisitMut` trait (#1570) (IWANABETHATGUY)
- 9ff0ffc ast: Implement new proposal-import-attributes (#1476) (magic-akari)
- 446ba16 ast: Add to_string function to VariableDelcartionKind (#1303) (Dunqing)
- 0115314 ast/semantic: Parse jsdoc on `PropertyDefinition` (#1517) (Shannon Rothe)
- afeed17 linter: Eslint-lugin-unicorn no_useless_length_check (#1541) (Radu Baston)
- 9754ef0 pretter: Start formatting `ModuleDeclaration` and `ArrowExpression` (#1354) (Boshen)
- da87b9b prettier: Binaryish expressions with parens (#1597) (Boshen)
- 1bd1c5b prettier: Check parens for `(let)[a] = 1` (#1585) (Boshen)
- c50fcec prettier: Wrap return statements with parentheses (#1583) (Boshen)
- e55fdc6 prettier: Add parens to conditional and arrow expr (#1530) (Boshen)
- 78c6fcd prettier: Improve format of ExportDefaultDeclaration  (#1520) (Boshen)
- 064353c prettier: Turn off preserve_parens and start working on need-parens (#1487) (Boshen)
- 0bf3dbf prettier: Add infra for need_parens (#1450) (Boshen)
- 9a21d1a prettier: Print `ExportAllDeclaration` (#1381) (Boshen)
- 6d8fa7f prettier: Sort regex flags (#1372) (Boshen)
- bfdb6ea prettier: Print statements with newlines (#1367) (Boshen)
- 5f31662 prettier: Add the basics of comment printing (#1313) (Boshen)
- 92c1d9d transform: TypeScript Enum (#1173) (magic-akari)
- 6cbc5dd transformer: Start on `function_name` transform. (#1510) (Miles Johnson)- 872e8ad Eslint-plugin-unicorn (recommended) prefer-node-protocol (#1618) (IWANABETHATGUY)

### Bug Fixes

- 6ebb42d ast: Remove debug_assertions from `debug_name` (Boshen)
- 9c0aafc parser: Disallow ReservedWord in NamedExports (#1230) (magic-akari)

### Refactor

- be043c3 ast: VariableDeclarationKind::to_string -> as_str (#1321) (Boshen)
- c5b138f prettier: Clean up object::print_object_properties (#1573) (Boshen)
- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 6c1388d ast: Enter/leave scopes in Visit (Don Isaac)
- 6c18b3e codegen: Beauty class print (#995) (Wenzhe Wang)
- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 25247e3 linter: Eslint/no-fallthrough (nursery) (Sg)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- 5b1e1e5 parser: TypeScript 5.2 (#811) (Cameron)
- 1661385 semantic: Check non-simple lhs expression of assignment expression (#994) (Boshen)
- 0856111 transformer: Implement more of react transform attributes (#1081) (Boshen)
- 5fb27fb transformer: Implement key extraction for react automatic (#1077) (Boshen)
- 394ed35 transformer: Implement react get_attribute_name (#1076) (Boshen)
- d8f1a7f transformer: Start implementing react jsx transform (#1057) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)
- 9ad2634 transformer: Class Static Block (#962) (magic-akari)
- 21066a9 transformer: Shorthand Properties (#960) (magic-akari)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)
- 419d5aa transformer: Transformer prototype (#918) (Boshen)
- 203cf37 transformer/react: Read comment pragma @jsxRuntime classic / automatic (#1133) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- f32bf27 codegen: Fix some typescript codegen problems (#989) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)- 266253c Ts parsing error (#940) (IWANABETHATGUY)

### Refactor

- 94792e9 ast: Split syntax_directed_operations into separate files (Boshen)
- 4787220 ast: Clean up some methods (Boshen)
- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- 70189f9 ast: Change the arguments order for some `new` functions (Boshen)
- db5417f clippy: Allow clippy::too_many_lines (Boshen)
- eaeb630 clippy: Allow struct_excessive_bools (Boshen)
- c7a04f4 transformer: Remove returning None from transform functions (#1079) (Boshen)

## [0.2.0] - 2023-09-14

### Features

- 741aa8d ast: Add to ChainExpression and ExpressionArrayElement to ASTKind (#785) (u9g)
- e7c2313 ast: Add `SymbolId` and `ReferenceId` (#755) (Yunfei He)
- 4754133 ast: AstKind::debug_name() (#665) (Don Isaac)

### Performance

- 6628fc8 linter: Reduce mallocs (#654) (Don Isaac)
- babbc47 parser: Lazily build trivia map instead of build in-place (#903) (Boshen)

### Documentation

- 89b49bd ast: Document why Directive.directive is a raw string (Boshen)

### Refactor

- 3516759 ast: Use `atom` for `Directive` and `Hashbang` (#701) (Yunfei He)- fdf288c Improve code coverage in various places (#721) (Boshen)

