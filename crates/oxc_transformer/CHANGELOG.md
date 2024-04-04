# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.1] - 2024-04-03

### Features

- Add compiler assumptions (#2872)
- Add proposal-decorators (#2868)
- Add react plugins (#2867)
- Add `transform-typescript` boilerplate (#2866)

### Bug Fixes

- Add serde "derive" feature to fix compile error

## [0.11.0] - 2024-03-30

### Features

- Numeric separator plugin. (#2795)
- Add transform literal for numeric literals. (#2797)
- Remove `verbatim_module_syntax` option (#2796)

### Bug Fixes

- Optional-catch-binding unused variable side effect (#2822)
- Add support for empty module declaration (#2834)

### Refactor

- Pass options via context. (#2794)
- Add walk_mut functions (#2776)
- Get rid of unsafe transmutation in VisitMut trait. (#2764)
- Change sourcemap name to take a reference (#2779)

## [0.10.0] - 2024-03-14

### Features

- Handling the coexistence of class decorators and member decorators (#2636)
- Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)

### Bug Fixes

- Parse `with_clause` in re-export declaration (#2634)

### Refactor

- Refactor `Trivias` API - have less noise around it (#2692)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Features

- Support transform constructor method (#2551)
- Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)
- Call build module record (#2529)

### Bug Fixes

- Missing check private function (#2607)
- Support FormalParameter.override (#2577)

### Refactor

- Clean up API around building sourcemaps (#2602)
- Improve implementation of remove import/export (#2530)

## [0.8.0] - 2024-02-26

### Features

- Insert only one private in expression (#2486)
- Update arrow_expression to arrow_function_expression (#2496)
- Insert instanceBrand function (#2480)
- Transform getter function (#2473)
- Configurable typescript codegen (#2443)

### Refactor

- Remove `TSEnumBody` (#2509)
- S/NumberLiteral/NumericLiteral to align with estree
- S/ArrowExpression/ArrowFunctionExpression to align estree
- If it is a private method definition, transform it (#2427)
- Move get_decorator_info inside the decorators (#2426)

## [0.6.0] - 2024-02-03

### Features

- Remove generator property from ArrowFunction (#2260)
- Remove expression property from Function (#2247)
- Support for static and private member decorators (#2246)
- Support method decorator and is not static (#2238)
- Support static member (#2235)
- Ensure property key consistency (#2233)
- Support transform member decorators (#2171)
- Transform-json-strings (#2168)
- Support version 2023-05 (#2152)
- Support transform the class decorators in export declaration (#2145)
- Add decorators plugin (#2139)
- Improve function parameters name (#2079)
- Support only_remove_type_imports option (#2077)
- Support transform exported TSModuleBlock (#2076)
- Support transform namespace (#2075)
- Keep imports if import specifiers is empty (#2058)
- Remove type-related exports (#2056)
- Remove type only imports/exports correctly (#2055)
- Remove export specifier that import_kind is type (#2015)
- Remove import if only have type reference (#2001)

### Bug Fixes

- Always create valid identifiers (#2131)

### Refactor

- Optimizing code with ast.private_field (#2249)
- Align the implementation of all versions (#2159)
- Improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153)
- Use `is_identifier_part`
- Use `is_identifier_name` from `oxc_syntax`
- Move the ExportNamedDeclaration logic to its function (#2074)

## [0.5.0] - 2024-01-12

### Features

- Call enter_node/leave_node in visit_xxx (#1990)
- Support for transform TSImportEqualsDeclaration (#1994)
- Support es2015 new target (#1967)
- Add partial support for babel-plugin-transform-instanceof (#1802)
- Add arrow_functions plugin (#1663)
- Returns ThisExpression when identifier is this (#1661)
- Duplicate keys (#1649)

### Refactor

- Introduce `ThisParameter` (#1728)

## [0.4.0] - 2023-12-08

### Features

- Support scope descendents starting from a certain scope. (#1629)
- Start on `function_name` transform. (#1510)
- Handle invalid react jsx  runtime (#1502)
- TypeScript Enum (#1173)
- Handle babel 8 breaking removed-options (#1489)
- Add transform property-literal plugin (#1458)
- Set `automatic` to the default value for `runtime` (#1270)
- Support for throwing SpreadChildrenAreNotSupported error (#1234)
- Support for throwing ImportSourceCannotBeSet error (#1224)
- Support throw valueless-key error (#1221)
- Implement `throwIfNamespace` option (#1220)
- When the source type is a script, use require to import the react (#1207)
- Throw the `pragma and pragmaFrag cannot be set when runtime is automatic` error (#1196)
- Support the `sourceType` is a `script` (#1192)
- Support `@jsxFrag` annotation (#1189)
- Support `@jsx` annotation (#1182)
- Support `pragmaFrag` option (#1181)
- Support `pragma` option (#1180)
- Support `@jsxImportSource` annotation (#1179)
- Support importSource option in react_jsx (#1115)

### Bug Fixes

- Missing import jsxs in nested fragment (#1218)
- Missing default options when plugin without config (#1219)
- Undetectable comments in multiline comments (#1211)
- No need to wrap the Array when there is only one correct child element (#1205)

### Refactor

- Move to workspace lint table (#1444)
- Use extend instead of for-in with push (#1236)
- Improve SpreadChildrenAreNotSupported error implementation (#1235)

## [0.3.0] - 2023-11-06

### Features

- Support TemplateLiteral of babel/plugin-transform-template-literals (#1132)
- Read comment pragma @jsxRuntime classic / automatic (#1133)
- Implement fixup_whitespace_and_decode_entities (#1091)
- Escape xhtml in jsx attributes (#1088)
- Implement some of jsx decode entities (#1086)
- Implement more of react transform attributes (#1081)
- Import jsxs when children is static (#1080)
- Finish transform jsx attribute value (#1078)
- Implement key extraction for react automatic (#1077)
- Implement react get_attribute_name (#1076)
- Implement react has_key_after_props_spread (#1075)
- Add props `null` to React.createElement (#1074)
- Implement react transform attributes (#1071)
- Transform jsx element name (#1070)
- Start implementing react jsx transform (#1057)
- Strip implicit type import for typescript (#1058)
- Implement some of needs_explicit_esm for typescript (#1047)
- Drop `this` parameter from typescript functions (#1019)
- Add utils to make logical_assignment_operators pass (#1017)
- Read plugins options from babel `options.json` (#1006)
- ES2020 Nullish Coalescing Operator (#1004)
- Add unit tests and test coverage (#1001)
- Finish 2016 exponentiation operator (#996)
- Add transform and minify (#993)
- Implement the basics of non-minifying codegen (#987)
- Move Formatter to codegen (#986)
- RegexpFlags (#977)
- Sticky-regex (#968)
- Class Static Block (#962)
- Shorthand Properties (#960)
- Setup typescript and react transformers (#930)
- Add jsx and ts tests
- Logical assignment operators (#923)
- Add babel conformance test suite (#920)
- Transformer prototype (#918)

### Bug Fixes

- Revert changes to JSX attribute strings (#1101)
- Jsx attribute value and text child should be jsx string (#1089)
- Add imports to the top body (#1087)
- Fix position of inserted react import statement (#1082)

### Refactor

- Move Semantic into Transformer (#1130)
- Remove returning None from transform functions (#1079)
- Add an empty SPAN utility for creating AST nodes (#1067)
- Add TransformerCtx struct for easier access to symbols and scopes
- Improve report format
- Clean up the transformer constructor code
- Clean up some methods
- Make the minifier api only accept an ast (#990)
- Fix the lifetime annotations around Vist and VisitMut (#973)
- Change the arguments order for some `new` functions

