# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.103.0] - 2025-12-15

### 🚀 Features

- d221921 semantic: ScopeFlags::With (#16291) (Aapo Alasuutari)

## [0.100.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 74cf572 ast: [**BREAKING**] Make `source` field of `TSImportType` a `StringLiteral` (#16114) (copilot-swe-agent)
- 43156ae ast: [**BREAKING**] Rename `TSImportType` `argument` field to `source` (#16110) (overlookmotel)

## [0.99.0] - 2025-11-24

### 💥 BREAKING CHANGES

- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

## [0.96.0] - 2025-10-30

### 🚀 Features

- bec7a7d semantic: Add scope to `TSConstructorType` (#14676) (camc314)
- f45d2f0 semantic: Add scope to `TSCallSignatureDeclaration` (#14672) (camc314)

### 🐛 Bug Fixes

- be94bfd semantic: Add scope tracking for `with` statements (#14652) (Boshen)






## [0.91.0] - 2025-09-22

### 🚀 Features

- a14aa79 npm/oxlint: Convert to ES modules (#13876) (Boshen)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-21

### 🚀 Features

- a14aa79 npm/oxlint: Convert to ES modules (#13876) (Boshen)









## [0.83.0] - 2025-08-29

### 🚀 Features

- bb10c88 ast_visit: Add `Utf8ToUtf16Converter::convert_program` method (#13341) (overlookmotel)
- 59d2c08 ast_visit: `Utf8ToUtf16` translate back from UTF-16 to UTF-8 (#13340) (overlookmotel)

### 🐛 Bug Fixes

- 1cdc420 ast_visit: Correct and expand comments in `Utf8ToUtf16Converter` (#13310) (overlookmotel)

### 🚜 Refactor

- cd5a9ca ast_visit: `Utf8ToUtf16` record end offset of multi-byte chars (#13339) (overlookmotel)





## [0.82.0] - 2025-08-12

### 💥 BREAKING CHANGES

- 128b527 data_structures: [**BREAKING**] Remove `PointerExt` trait (#12903) (overlookmotel)

### 🚜 Refactor

- c072e01 all: Add missing lifetimes in function return types (#12895) (overlookmotel)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)

### 📚 Documentation

- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)



## [0.79.0] - 2025-07-30

### 🚜 Refactor

- a696227 linter: Remove AstKind for SimpleAssignmentTarget (#12401) (Tyler Earls)





## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- ee761de ast: Remove `AstKind` for `AssignmentTarget` (#12252) (Tyler Earls)
- c025868 ast: Remove `AstKind` for `TSFunctionType` (#12287) (camc314)


## [0.77.0] - 2025-07-12

### 🚜 Refactor

- 8814c53 ast: Remove `AstKind` for `PropertyKey` (#12108) (camchenry)


## [0.76.0] - 2025-07-08

### 🚜 Refactor

- e8e2a25 ast: Remove `AstKind` for `AssignmentTargetPattern` (#12105) (camchenry)


## [0.75.1] - 2025-07-03

### 🚜 Refactor

- f1d4086 ast: Remove `AstKind` for `ModuleDeclaration` (#12022) (camchenry)
- 754c05a ast: Remove `AstKind` for `TSTypeName` (#11990) (camchenry)
- f7a2ae4 ast: Add `AstKind` for `AssignmentTargetPropertyIdentifier`, `AssignmentTargetPropertyProperty` (#11985) (camc314)
- cfa52c2 ast: Add `AstKind` for `AssignmentTargetRest` (#11984) (camc314)
- 54582cb ast: Add `AstKind` for `BindingProperty` (#11974) (camc314)
- 9f6784a ast: Add `AstKind` for `TSInterfaceBody` (#11967) (camc314)
- 3f50cef ast: Add `AstKind` for `TSIndexSignature` (#11966) (camc314)
- 03bce3f ast: Add `AstKind` for `TSConstructorType` (#11965) (camc314)
- 0cef370 ast: Add `AstKind::TemplateElement` (#11955) (camchenry)


## [0.75.0] - 2025-06-25

### 🚜 Refactor

- 87b8496 ast: Remove `AstKind` for `MemberExpression` and replace with `StaticMemberExpression` and `PrivateFieldExpression` (#11767) (camchenry)
- 190e390 ast: Add `AstKind` for `ComputedMemberExpression` (#11766) (camchenry)




## [0.73.1] - 2025-06-17

### 🚀 Features

- 17f5dbe ast: Add AstKind to ImportAttribute node (#11765) (therewillbecode)
- eb9db97 ast: Add AstKind to AccessorProperty node (#11764) (therewillbecode)
- 584844c ast: Add AstKind to TSNamespaceExportDeclaration node (#11754) (therewillbecode)
- 6095438 ast: Add AstKind to TSRestType node (#11752) (therewillbecode)
- b8237b8 ast: Add AstKind to TSOptionalType node (#11751) (therewillbecode)
- b949ece ast: Add AstKind to TSTupleType node (#11749) (therewillbecode)
- d74c4af ast: Add AstKind to `TSTypePredicate` node (#11726) (therewillbecode)
- c25b153 ast: Add AstKind to ` TSCallSignatureDeclaration` node (#11725) (therewillbecode)
- 866470a ast: Add AstKind to `TSIndexSignature` node (#11724) (therewillbecode)
- 37d4c9a ast: Add `AstKind` for `WithClause` (#11711) (camchenry)

### 🚜 Refactor

- 3d89012 ast: Add `AstKind` for `TSTypeOperator` (#11747) (camchenry)
- 5ca3d04 ast: Add `TSArrayType` as `AstKind` (#11745) (camchenry)
- 4fbe4b1 ast: Remove AstKind from `TSModuleReference` node (#11732) (therewillbecode)
- 219adcc ast: Don't generate AstKind for ArrayExpressionElement (#11684) (Ulrich Stark)


## [0.73.0] - 2025-06-13

### 🚜 Refactor

- 8e30c5f ast: Don't generate AstKind for ForStatementInit (#11617) (Ulrich Stark)
- 9136685 ast: Create AstKinds for jsdoc types (#11597) (Ulrich Stark)
- d41fb13 ast: Get jsx types out of AstKind exceptions (#11535) (Ulrich Stark)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- c79a7d0 data_structures: Introduce `PointerExt` trait (#11095) (overlookmotel)

### Refactor

- bb8bde3 various: Update macros to use `expr` fragment specifier (#11113) (overlookmotel)

## [0.70.0] - 2025-05-15

### Bug Fixes

- 2bdb338 ast_visit: Fix visitation order for `FormalParameters` in `Utf8ToUtf16Converter` (#11019) (overlookmotel)
- 282420c ast_visit: Fix visitation order for `TSTemplateLiteralType` in `Utf8ToUtf16Converter` (#11007) (overlookmotel)
- d0d04e3 ast_visit: Fix visitation order for `BindingPattern` in `Utf8ToUtf16Converter` (#11003) (overlookmotel)

### Refactor

- 287b3c3 ast_visit: Descriptive method param names in `Utf8ToUtf16Converter` (#10987) (overlookmotel)

### Testing

- a05361e ast/estree: Check span offsets are converted in ascending order in ESTree conformance tests (#10887) (overlookmotel)

## [0.69.0] - 2025-05-09

- 2b5d826 ast: [**BREAKING**] Fix field order for `TSTypeAssertion` (#10906) (overlookmotel)

- 1f35910 ast: [**BREAKING**] Fix field order for `TSNamedTupleMember` (#10905) (overlookmotel)

- 8a3bba8 ast: [**BREAKING**] Fix field order for `PropertyDefinition` (#10902) (overlookmotel)

- 5746d36 ast: [**BREAKING**] Fix field order for `NewExpression` (#10893) (overlookmotel)

- 0139793 ast: [**BREAKING**] Re-order fields of `TaggedTemplateExpression` (#10889) (overlookmotel)

- 6646b6b ast: [**BREAKING**] Fix field order for `JSXOpeningElement` (#10882) (overlookmotel)

- cc2ed21 ast: [**BREAKING**] Fix field order for `JSXElement` and `JSXFragment` (#10881) (overlookmotel)

### Bug Fixes

- 2c09243 ast: Fix field order for `AccessorProperty` (#10878) (overlookmotel)

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

- d6b7982 ast: [**BREAKING**] Improve pluralization of `TSClassImplements` (#10489) (overlookmotel)

### Refactor


## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features


### Bug Fixes

- 6d3734b ast_visit: `Utf8ToUtf16Converter` process decorators before class (#10449) (overlookmotel)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features


## [0.59.0] - 2025-03-18

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Bug Fixes


## [0.58.0] - 2025-03-13

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

- 48a0394 ast: [**BREAKING**] Add `scope_id` to `TSFunctionType` (#9559) (camc314)

### Features


### Bug Fixes

- bbb4f98 semantic: Insert binding into correct scope for TSInferType (#9567) (camc314)

## [0.55.0] - 2025-03-05

### Features

- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

## [0.54.0] - 2025-03-04

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)

