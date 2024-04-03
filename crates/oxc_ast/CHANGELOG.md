# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.1] - 2024-04-03

### Bug Fixes

- `FinallyClause` won't get visited as `BlockStatement` anymore. (#2881)

## [0.11.0] - 2024-03-30

### Bug Fixes

- Add support for empty module declaration (#2834)

### Refactor

- Add walk_mut functions (#2776)
- Add walk functions to Visit trait. (#2791)
- Get rid of unsafe transmutation in VisitMut trait. (#2764)

## [0.10.0] - 2024-03-14

### Features

- Merge features `serde` and `wasm` to `serialize` (#2716)
- Fill in missing ast visits (#2705)
- Handling the coexistence of class decorators and member decorators (#2636)
- Print `with_clause` in reexport declaration (#2635)
- Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)

### Bug Fixes

- Serialize empty array elements as null (#2707)
- Correct TS type for `ArrayAssignmentTarget` (#2699)
- Add `type` field to TS types for `ObjectPattern` etc (#2670)
- Fix TS type for `AssignmentTargetRest` (#2668)
- Rename `TSIndexSignatureName` in JSON AST (#2664)
- Rename BigintLiteral to BigIntLiteral (#2659)
- Parse named rest element in type tuple (#2655)
- Drop TSImportEqualsDeclaration.is_export (#2654)
- Fix serializing rest elements (#2652)
- Add `RestElement`s in serialized AST to elements array (#2567)
- Parse `with_clause` in re-export declaration (#2634)

### Refactor

- Derive `SerAttrs` on all AST types (#2698)
- Refactor `Trivias` API - have less noise around it (#2692)
- Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669)
- Import `Tsify` to shorten code (#2665)
- Remove `Serialize` impls for Identifier types (#2651)
- "wasm" feature enable "serde" feature (#2639)
- Shorten manual TS defs (#2638)

## [0.9.0] - 2024-03-05

### Features

- Serialize `BindingPattern` to estree (#2610)
- Serialize identifiers to ESTree (#2521)
- Add `AssignmentTargetRest` (#2601)
- Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)

### Bug Fixes

- Temporary fix tsify not generating some typings (#2611)
- Align TSImportType with ESTree (#2578)
- Expose NumericLiteral.raw (#2588)
- Parse empty method declaration as TSEmptyBodyFunctionExpression (#2574)
- Support TSIndexSignature.readonly (#2579)
- Support FormalParameter.override (#2577)
- Change TSMappedType.type_annotation from TSTypeAnnotation to TSType (#2571)
- Rename serialized fields to camel case (#2566)
- Fix getter return rule false positives in TypeScript (#2543)
- Missing visit JSXElementName enum (#2547)
- Add Function to generated TS types and fix ModifierKind serialization (#2534)
- Few serialization issues (#2522)
- Incorrect scope for switch statement (#2513)

## [0.8.0] - 2024-02-26

### Features

- Update arrow_expression to arrow_function_expression (#2496)
- Handle cjs `module.exports = {} as default export (#2493)
- Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
- Add `TSModuleDeclaration.kind` (#2487)
- Insert instanceBrand function (#2480)
- Parse import attributes in TSImportType (#2436)
- Print newlines between array expression elements (#2379)

### Bug Fixes

- Semi colon after class property (#2387)

### Refactor

- Remove `TSEnumBody` (#2509)
- S/TSThisKeyword/TSThisType to align with estree
- S/NumberLiteral/NumericLiteral to align with estree
- S/ArrowExpression/ArrowFunctionExpression to align estree
- Update TSImportType parameter to argument (#2429)

## [0.7.0] - 2024-02-09

### Features

- Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317)
- Report parameter related errors for setter/getter (#2316)

### Bug Fixes

- Format new expession + import expression with the correct parentheses (#2346)
- Fix no_dupe_keys false postive on similar key names (#2291)

### Refactor

- Fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Features

- Check optional parameters (#2263)
- Remove generator property from ArrowFunction (#2260)
- Remove expression property from Function (#2247)
- Support for static and private member decorators (#2246)
- Support method decorator and is not static (#2238)
- Remove serde skip for symbol_id and reference_id (#2220)
- TypeScript definition for wasm target (#2158)
- Complete AccessorProperty todo in has_decorator (#2178)
- Support transform member decorators (#2171)
- Cfg prototype (#2019)
- Support version 2023-05 (#2152)
- Support transform the class decorators in export declaration (#2145)
- Add decorators plugin (#2139)
- Support transform namespace (#2075)
- Remove type-related exports (#2056)
- Visit TSTypeQuery (#2021)

### Bug Fixes

- AcessorProperty is missing decorators (#2176)
- Fix crash on TSTemplateLiteralType in function return position (#2089)

### Refactor

- Optimizing code with ast.private_field (#2249)
- Adding binder for ImportSpecifier replaces the ModuleDeclaration's binder (#2230)
- Checking label in ContinueStatement based on LabelBuilder (#2202)
- Remove Regex and change error position (#2188)
- Improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153)
- Rename RestElement to BindingRestElement (#2116)
- Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114)

## [0.5.0] - 2024-01-12

### Features

- Visit TSModuleReference (#1998)
- No-irregular-whitespace rule (#1835)
- Support eslint/no-unused-private-class-members rule (#1820)
- Add ClassTable (#1793)
- Enter/leave ClassBody and PrivateInExpression (#1792)
- Support visit more jsx ast in visit (#1662)
- Print CallExpression arguments correctly (#1631)

### Bug Fixes

- Default visitor should visit prop init at `visit_object_property` (#2000)
- Implement `GetSpan` for `JSXElement` (#1861)

### Refactor

- Remove TokenValue::RegExp from `Token` (#1926)
- Introduce `ThisParameter` (#1728)

## [0.4.0] - 2023-12-08

### Features

- Start on `function_name` transform. (#1510)
- Eslint-plugin-unicorn (recommended) prefer-node-protocol (#1618)
- Binaryish expressions with parens (#1597)
- Check parens for `(let)[a] = 1` (#1585)
- Wrap return statements with parentheses (#1583)
- Add enter node and scope for `VisitMut` trait (#1570)
- TypeScript Enum (#1173)
- Eslint-lugin-unicorn no_useless_length_check (#1541)
- Implement new proposal-import-attributes (#1476)
- Add parens to conditional and arrow expr (#1530)
- Improve format of ExportDefaultDeclaration  (#1520)
- Parse jsdoc on `PropertyDefinition` (#1517)
- Turn off preserve_parens and start working on need-parens (#1487)
- Add infra for need_parens (#1450)
- Print `ExportAllDeclaration` (#1381)
- Sort regex flags (#1372)
- Print statements with newlines (#1367)
- Start formatting `ModuleDeclaration` and `ArrowExpression` (#1354)
- Add the basics of comment printing (#1313)
- Add to_string function to VariableDelcartionKind (#1303)

### Bug Fixes

- Remove debug_assertions from `debug_name`
- Disallow ReservedWord in NamedExports (#1230)

### Refactor

- Clean up object::print_object_properties (#1573)
- Move to workspace lint table (#1444)
- VariableDeclarationKind::to_string -> as_str (#1321)

## [0.3.0] - 2023-11-06

### Features

- Read comment pragma @jsxRuntime classic / automatic (#1133)
- Implement more of react transform attributes (#1081)
- Implement key extraction for react automatic (#1077)
- Implement react get_attribute_name (#1076)
- Start implementing react jsx transform (#1057)
- Implement some of needs_explicit_esm for typescript (#1047)
- Eslint/no-fallthrough (nursery)
- Add utils to make logical_assignment_operators pass (#1017)
- ES2020 Nullish Coalescing Operator (#1004)
- Finish 2016 exponentiation operator (#996)
- Beauty class print (#995)
- Check non-simple lhs expression of assignment expression (#994)
- Implement the basics of non-minifying codegen (#987)
- Re-enable mangler (#972)
- Enter/leave scopes in Visit
- Partially re-enable minifier (#963)
- Class Static Block (#962)
- Shorthand Properties (#960)
- TypeScript 5.2 (#811)
- Logical assignment operators (#923)
- Transformer prototype (#918)

### Bug Fixes

- Revert changes to JSX attribute strings (#1101)
- Jsx attribute value and text child should be jsx string (#1089)
- Fix some typescript codegen problems (#989)
- Ts parsing error (#940)

### Refactor

- Remove returning None from transform functions (#1079)
- Split syntax_directed_operations into separate files
- Allow clippy::too_many_lines
- Allow struct_excessive_bools
- Clean up some methods
- Fix the lifetime annotations around Vist and VisitMut (#973)
- Change the arguments order for some `new` functions

## [0.2.0] - 2023-09-14

### Features

- Add to ChainExpression and ExpressionArrayElement to ASTKind (#785)
- Add `SymbolId` and `ReferenceId` (#755)
- AstKind::debug_name() (#665)

### Performance

- Lazily build trivia map instead of build in-place (#903)
- Reduce mallocs (#654)

### Documentation

- Document why Directive.directive is a raw string

### Refactor

- Improve code coverage in various places (#721)
- Use `atom` for `Directive` and `Hashbang` (#701)

