# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.3] - 2024-06-04

### Bug Fixes

* transformer: JSX set `symbol_id` on imports (#3523)
* transformer: TS namespace transform do not track var decl names (#3501)
* transformer: use correct scope for TS namespaces (#3489)
* transformer: output empty file for TS definition files (#3500)

### Performance

* transformer: React JSX reduce allocations (#3522)
* transformer: React JSX reuse same `Atom`s (#3521)

### Refactor

* traverse: `generate_uid` return `SymbolId` (#3520)

## [0.13.2] - 2024-06-03

### Refactor

* ast: move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488)
* transformer: explicit skip TS statements in TS namespace transform (#3479)
* transformer: shorter code in TS namespace transform (#3478)
* transformer: panic on illegal cases in TS namespace transform (#3477)
* transformer: rename var (#3476)
* transformer: shorten code in TS namespace transform (#3468)
* transformer: remove unreachable code from TS namespace transform (#3475)
* transformer/typescript: use a memory-safe implementation instead (#3481)
* typescript/namespace: reuse TSModuleBlock's scope id (#3459)

### Features

* oxc_codegen: preserve annotate comment (#3465)
* tasks/coverage: turn on idempotency testing for transformer (#3470)
* transformer: support `targets` option of preset-env (#3371)
* transformer/typescript: report error that do not allow namespaces (#3448)
* transformer/typescript: report error for namespace exporting non-const (#3447)
* transformer/typescript: if within a block scope, use let to declare enum name (#3446)
* transformer/typescript: if binding exists, variable declarations are not created for namespace name (#3445)
* transformer/typescript: if the binding exists, the identifier reference is not renamed (#3387)

### Bug Fixes

* transformer: use UIDs for React imports (#3431)
* transformer: use UIDs in TS namespace transforms (#3395)
* transformer/jsx-source: add filename statement only after inserting the source object (#3469)
* transformer/typescript: variable declarations are not created when a function has a binding with the same name (#3460)

## [0.13.1] - 2024-05-22

### Refactor

* diagnostics: s/warning/warn
* transformer: correct spelling of var name (#3369)
* transformer: improve indentation (#3282)
* traverse: `Traverse` produce scopes tree using `Semantic` (#3304)

### Features

* transformer: report errors when options have unknown fields (#3322)
* transformer: support `from_babel_options` in TransformOptions (#3301)
* transformer: do not add self attribute in react/jsx plugin (#3287)
* traverse: pass `&mut TraverseCtx` to visitors (#3312)

### Bug Fixes

* transformer: do no add __self when the jsx is inside constructor (#3258)

## [0.13.0] - 2024-05-14

### Features

* syntax: add `ToJsInt32` trait for f64 (#3132)
* syntax: add `ToJsString` trait for f64 (#3131)
* transformer: report ambient module cannot be nested error (#3253)
* transformer: do not elide jsx imports if a jsx element appears somewhere (#3237)
* transformer: add arrow-functions plugin (#3083)
* transformer: implement typescript namespace (#3025)
* transformer/jsx-source: get the correct lineNumber and columnNumber from the span. (#3142)
* transformer/react: support development mode (#3143)
* transformer/react: enable jsx plugin when development is true (#3141)

### Refactor

* ast: squash nested enums (#3115)
* syntax: move number related functions to number module (#3130)
* transform: transformer use `Traverse` (#3182)
* transformer: clean up more diagnostics
* transformer: unify diagnostics
* transformer: remove no-op scopes code (#3210)
* transformer: remove the requirement of `Semantic` (#3140)
* transformer/typescript: reimplementation of Enum conversion based on Babel (#3102)- remove all usages of `Into<Error>` |

### Bug Fixes

* transform: implement `transform-react-display-name` with bottom-up lookup (#3183)
* transformer: correctly jsx-self inside arrow-function (#3224)
* transformer/arrow-functions: should not transform `this` in class (#3129)

## [0.12.5] - 2024-04-22

### Performance

* ast: box typescript enum variants. (#3065)
* ast: box enum variants (#3058)
* ast: box `ImportDeclarationSpecifier` enum variants (#3061)

## [0.12.4] - 2024-04-19

### Features

* transform_conformance: skip tests with plugin.js (#2978)
* transform_conformance: skip plugins we don't support yet (#2967)
* transformer: add "_jsxFileName" variable in jsx source plugin (#3000)
* transformer: add import helpers to manage module imports (#2996)
* transformer: add diagnostics to react transform (#2974)
* transformer: apply jsx self and source plugin inside jsx transform (#2966)
* transformer: react jsx transform (#2961)
* transformer: start on TypeScript annotation removal (#2951)
* transformer: add the most basic plugin toggles (#2950)
* transformer: implement react-jsx-source (#2948)
* transformer: implement react-jsx-self (#2946)
* transformer: transform TypeScript namespace (#2942)
* transformer: add filename (#2941)
* transformer/react: reports duplicate __self/__source prop error (#3009)
* transformer/typescript: report error for export = <value> (#3021)
* transformer/typescript: reports error for import lib = require(...); (#3020)
* transformer/typescript: insert this assignment after the super call (#3018)
* transformer/typescript: support for transform TSImportEqualsDeclaration (#2998)
* transformer/typescript: support for transform enum (#2997)
* transformer/typescript: correct elide imports/exports statements (#2995)

### Bug Fixes

* transformer: `TypeScriptOptions` deserialize should fallback to default (#3012)
* transformer: react `development` default value should be false (#3002)
* transformer: deserialize ReactJsxRuntime with camelCase (#2972)
* transformer: turn on react preset by default (#2968)
* transformer: fix incorrect jsx whitespace text handling (#2969)
* transformer/typescript: modifiers should not be removed (#3005)

### Refactor

* transformer: remove boilerplate code around decorators to reduce noise (#2991)
* transformer: clean up some code (#2949)

## [0.12.3] - 2024-04-11

### Features

* transformer: implement plugin-transform-react-display-name top-down (#2937)
* transformer: add transform context to all plugins (#2931)
* transformer: add transform callback methods (#2929)
* transformer: add react preset (#2921)

## [0.12.1] - 2024-04-03

### Bug Fixes

* transformer: add serde "derive" feature to fix compile error

### Features

* transformer: add compiler assumptions (#2872)
* transformer: add proposal-decorators (#2868)
* transformer: add react plugins (#2867)
* transformer: add `transform-typescript` boilerplate (#2866)

## [0.11.0] - 2024-03-30

### Bug Fixes

* parser: add support for empty module declaration (#2834)
* transformer: optional-catch-binding unused variable side effect (#2822)

### Refactor

* ast: add walk_mut functions (#2776)
* ast: get rid of unsafe transmutation in VisitMut trait. (#2764)
* sourcemap: change sourcemap name to take a reference (#2779)
* transformer: pass options via context. (#2794)

### Features

* transformer: numeric separator plugin. (#2795)
* transformer: add transform literal for numeric literals. (#2797)
* transformer/typescript: remove `verbatim_module_syntax` option (#2796)

## [0.10.0] - 2024-03-14

### Refactor

* ast: refactor `Trivias` API - have less noise around it (#2692)- rename `CompactString` to `CompactStr` (#2619) |

### Features

* span: remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)
* transformer/decorators: handling the coexistence of class decorators and member decorators (#2636)

### Bug Fixes

* ast: parse `with_clause` in re-export declaration (#2634)

## [0.9.0] - 2024-03-05

### Bug Fixes

* ast: support FormalParameter.override (#2577)
* transformer/decorators: missing check private function (#2607)

### Refactor

* codegen: clean up API around building sourcemaps (#2602)
* transformer/typescript: improve implementation of remove import/export (#2530)

### Features

* ast: add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)
* transformer: call build module record (#2529)
* transformer/typescript: support transform constructor method (#2551)

## [0.8.0] - 2024-02-26

### Features

* ast: update arrow_expression to arrow_function_expression (#2496)
* codegen: configurable typescript codegen (#2443)
* transformer/decorators: insert only one private in expression (#2486)
* transformer/decorators: insert instanceBrand function (#2480)
* transformer/decorators: transform getter function (#2473)

### Refactor

* ast: remove `TSEnumBody` (#2509)
* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: s/ArrowExpression/ArrowFunctionExpression to align estree
* transformer/decorators: if it is a private method definition, transform it (#2427)
* transformer/decorators: move get_decorator_info inside the decorators (#2426)

## [0.6.0] - 2024-02-03

### Features

* ast: remove generator property from ArrowFunction (#2260)
* ast: remove expression property from Function (#2247)
* transformer: add decorators plugin (#2139)
* transformer/decorators: support for static and private member decorators (#2246)
* transformer/decorators: support method decorator and is not static (#2238)
* transformer/decorators: support static member (#2235)
* transformer/decorators: ensure property key consistency (#2233)
* transformer/decorators: support transform member decorators (#2171)
* transformer/decorators: support version 2023-05 (#2152)
* transformer/decorators: support transform the class decorators in export declaration (#2145)
* transformer/typescript: improve function parameters name (#2079)
* transformer/typescript: support only_remove_type_imports option (#2077)
* transformer/typescript: support transform exported TSModuleBlock (#2076)
* transformer/typescript: support transform namespace (#2075)
* transformer/typescript: keep imports if import specifiers is empty (#2058)
* transformer/typescript: remove type-related exports (#2056)
* transformer/typescript: remove type only imports/exports correctly (#2055)
* transformer/typescript: remove export specifier that import_kind is type (#2015)
* transformer/typescript: remove import if only have type reference (#2001)
* transfrom: transform-json-strings (#2168)

### Refactor

* ast: improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153)
* transformer: use `is_identifier_part`
* transformer: use `is_identifier_name` from `oxc_syntax`
* transformer/decorators: optimizing code with ast.private_field (#2249)
* transformer/decorators: align the implementation of all versions (#2159)
* transformer/typescript: move the ExportNamedDeclaration logic to its function (#2074)

### Bug Fixes

* transformer: always create valid identifiers (#2131)

## [0.5.0] - 2024-01-12

### Features

* transform: support es2015 new target (#1967)
* transformer: call enter_node/leave_node in visit_xxx (#1990)
* transformer: support for transform TSImportEqualsDeclaration (#1994)
* transformer: add partial support for babel-plugin-transform-instanceof (#1802)
* transformer: add arrow_functions plugin (#1663)
* transformer: duplicate keys (#1649)
* transformer/react-jsx: returns ThisExpression when identifier is this (#1661)

### Refactor

* ast: introduce `ThisParameter` (#1728)

## [0.4.0] - 2023-12-08

### Features

* semantic: support scope descendents starting from a certain scope. (#1629)
* transform: TypeScript Enum (#1173)
* transformer: Start on `function_name` transform. (#1510)
* transformer: handle invalid react jsx  runtime (#1502)
* transformer: add transform property-literal plugin (#1458)
* transformer: support importSource option in react_jsx (#1115)
* transformer/react: handle babel 8 breaking removed-options (#1489)
* transformer/react-jsx: set `automatic` to the default value for `runtime` (#1270)
* transformer/react-jsx: support for throwing SpreadChildrenAreNotSupported error (#1234)
* transformer/react-jsx: support for throwing ImportSourceCannotBeSet error (#1224)
* transformer/react-jsx: support throw valueless-key error (#1221)
* transformer/react-jsx: implement `throwIfNamespace` option (#1220)
* transformer/react-jsx: when the source type is a script, use require to import the react (#1207)
* transformer/react-jsx: throw the `pragma and pragmaFrag cannot be set when runtime is automatic` error (#1196)
* transformer/react-jsx: support the `sourceType` is a `script` (#1192)
* transformer/react-jsx: support `@jsxFrag` annotation (#1189)
* transformer/react-jsx: support `@jsx` annotation (#1182)
* transformer/react-jsx: support `pragmaFrag` option (#1181)
* transformer/react-jsx: support `pragma` option (#1180)
* transformer/react-jsx: support `@jsxImportSource` annotation (#1179)

### Refactor

* rust: move to workspace lint table (#1444)
* transformer/react-jsx: use extend instead of for-in with push (#1236)
* transformer/react-jsx: improve SpreadChildrenAreNotSupported error implementation (#1235)

### Bug Fixes

* transformer/react-jsx: missing import jsxs in nested fragment (#1218)
* transformer/react-jsx: missing default options when plugin without config (#1219)
* transformer/react-jsx: undetectable comments in multiline comments (#1211)
* transformer/react-jsx: no need to wrap the Array when there is only one correct child element (#1205)

## [0.3.0] - 2023-11-06

### Features

* codegen: implement the basics of non-minifying codegen (#987)
* playground: add transform and minify (#993)
* transfomer: implement react has_key_after_props_spread (#1075)
* transform: support TemplateLiteral of babel/plugin-transform-template-literals (#1132)
* transform: transform jsx element name (#1070)
* transform: sticky-regex (#968)
* transform_conformance: move Formatter to codegen (#986)
* transform_conformance: add jsx and ts tests
* transformer: implement some of jsx decode entities (#1086)
* transformer: implement more of react transform attributes (#1081)
* transformer: import jsxs when children is static (#1080)
* transformer: finish transform jsx attribute value (#1078)
* transformer: implement key extraction for react automatic (#1077)
* transformer: implement react get_attribute_name (#1076)
* transformer: add props `null` to React.createElement (#1074)
* transformer: implement react transform attributes (#1071)
* transformer: start implementing react jsx transform (#1057)
* transformer: strip implicit type import for typescript (#1058)
* transformer: implement some of needs_explicit_esm for typescript (#1047)
* transformer: drop `this` parameter from typescript functions (#1019)
* transformer: add utils to make logical_assignment_operators pass (#1017)
* transformer: ES2020 Nullish Coalescing Operator (#1004)
* transformer: add unit tests and test coverage (#1001)
* transformer: finish 2016 exponentiation operator (#996)
* transformer: RegexpFlags (#977)
* transformer: Class Static Block (#962)
* transformer: Shorthand Properties (#960)
* transformer: setup typescript and react transformers (#930)
* transformer: logical assignment operators (#923)
* transformer: add babel conformance test suite (#920)
* transformer: transformer prototype (#918)
* transformer/jsx: escape xhtml in jsx attributes (#1088)
* transformer/react: read comment pragma @jsxRuntime classic / automatic (#1133)
* transformer/react: implement fixup_whitespace_and_decode_entities (#1091)
* transformer_conformance: read plugins options from babel `options.json` (#1006)

### Refactor

* ast: clean up some methods
* ast: fix the lifetime annotations around Vist and VisitMut (#973)
* ast: change the arguments order for some `new` functions
* minifier: make the minifier api only accept an ast (#990)
* transform_conformance: improve report format
* transformer: move Semantic into Transformer (#1130)
* transformer: remove returning None from transform functions (#1079)
* transformer: add an empty SPAN utility for creating AST nodes (#1067)
* transformer: add TransformerCtx struct for easier access to symbols and scopes
* transformer: clean up the transformer constructor code

### Bug Fixes

* ast: jsx attribute value and text child should be jsx string (#1089)
* linter: revert changes to JSX attribute strings (#1101)
* transformer: fix position of inserted react import statement (#1082)
* transformer/react_jsx: add imports to the top body (#1087)

