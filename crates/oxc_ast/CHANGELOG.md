# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.82.0] - 2025-08-12

### 🚜 Refactor

- bf9ae3b ast/estree: Remove custom serializer for `TSImportTypeQualifier` (#12961) (overlookmotel)
- e5e2496 minifier: Clean up `try_compress_typeof_undefined` (#12958) (Boshen)
- c072e01 all: Add missing lifetimes in function return types (#12895) (overlookmotel)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)

### 🐛 Bug Fixes

- 47e2c59 estree: Fix serialization of `TSImportTypeQualifier` (#12801) (camc314)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)
- 7332ae4 ast: [**BREAKING**] Box `rest` fields of `ArrayAssignmentTarget` and `ObjectAssignmentTarget` (#12698) (Copilot)

### 📚 Documentation

- d7a3e03 ast: Improve docs for `ImportSpecifier` (#12740) (overlookmotel)
- eb22805 ast: Fix JavaScript/TypeScript code snippets to use proper backticks in documentation (#12729) (Copilot)
- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)



## [0.79.0] - 2025-07-30

### 🚀 Features

- 26249cf ast: Add `AstKind::ident_reference_is_assigned_to_in_parent` method (#12597) (overlookmotel)
- 106f56f ast: Add `MemberExpressionKind::is_assigned_to_in_parent` method (#12596) (overlookmotel)
- c763e0e ast: Implement `GetAddress` for `MemberExpressionKind` (#12593) (overlookmotel)
- 23f7f82 minifier: Remove unused assignment expression (#12509) (Boshen)
- 515b473 ast: Add `as_property` method to `ObjectPropertyKind` to access `ObjectProperty` if relevent (#12495) (camc314)

### 🐛 Bug Fixes

- 94b06ef ast: Correct logic in `Expression::is_call_like_expression` (#12534) (Dunqing)

### 🚜 Refactor

- a696227 linter: Remove AstKind for SimpleAssignmentTarget (#12401) (Tyler Earls)


## [0.78.0] - 2025-07-24

### 🚀 Features

- dee25f4 ast: Add `pife` field to `Function` (#12469) (sapphi-red)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)


## [0.77.2] - 2025-07-17

### 🚀 Features

- 37be46c ast: Introduce `NewExpression::arguments_span` (#12368) (camc314)
- 6b8f77d ast: Introduce `CallExpression::arguments_span` (#12321) (camc314)


## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- ee761de ast: Remove `AstKind` for `AssignmentTarget` (#12252) (Tyler Earls)
- c025868 ast: Remove `AstKind` for `TSFunctionType` (#12287) (camc314)
- c68b607 ast: Rename `TemplateLiteral::quasi` to `TemplateLiteral::single_quasi` (#12266) (Dunqing)
- 32c32af ast: Check whether there is a single `quasi` in `TemplateLiteral::quasi` (#12265) (Dunqing)


## [0.77.0] - 2025-07-12

### 🚜 Refactor

- 8814c53 ast: Remove `AstKind` for `PropertyKey` (#12108) (camchenry)

### ⚡ Performance

- 4c35f4a napi/parser: Optimize raw transfer deserializer for `TSClassImplements` (#12158) (overlookmotel)


## [0.76.0] - 2025-07-08

### 💥 BREAKING CHANGES

- 8b30a5b codegen: [**BREAKING**] Introduce `CommentOptions` (#12114) (Boshen)

### 🚜 Refactor

- e8e2a25 ast: Remove `AstKind` for `AssignmentTargetPattern` (#12105) (camchenry)


## [0.75.1] - 2025-07-03

### 🚀 Features

- 79c93e3 ast: Add `range` field to custom serializers (#11890) (Bacary Bruno Bodian)
- 6a4db24 ast: Derive `Debug, Clone, Copy` on `MemberExpressionKind` (#11903) (Boshen)
- be5e5e8 ast: Add `MemberExpressionKind::static_property_info` (#11900) (Boshen)

### 🐛 Bug Fixes

- 6d865af parser: Allow `extends this.B` syntax (#12038) (camchenry)
- 43e6962 ast/estree: Add `range` field to `TSModuleDeclaration` (#12006) (overlookmotel)

### 🚜 Refactor

- f5ec26b ast: Fix misspelling (#12055) (overlookmotel)
- f1d4086 ast: Remove `AstKind` for `ModuleDeclaration` (#12022) (camchenry)
- 754c05a ast: Remove `AstKind` for `TSTypeName` (#11990) (camchenry)
- 016634f ast/estree: Introduce `serialize_span` method (#12013) (overlookmotel)
- 754f01d ast/estree: Move `start` and `end` fields to last (#12012) (overlookmotel)
- 8f83067 ast: Remove dummy comment (#12010) (overlookmotel)
- 4597311 ast/estree: Remove temp vars for `ranges` from serializer (#12007) (overlookmotel)
- dc04dbc ast/estree: Simplify adding `range` field (#12005) (overlookmotel)
- f7a2ae4 ast: Add `AstKind` for `AssignmentTargetPropertyIdentifier`, `AssignmentTargetPropertyProperty` (#11985) (camc314)
- cfa52c2 ast: Add `AstKind` for `AssignmentTargetRest` (#11984) (camc314)
- 54582cb ast: Add `AstKind` for `BindingProperty` (#11974) (camc314)
- 9f6784a ast: Add `AstKind` for `TSInterfaceBody` (#11967) (camc314)
- 3f50cef ast: Add `AstKind` for `TSIndexSignature` (#11966) (camc314)
- 03bce3f ast: Add `AstKind` for `TSConstructorType` (#11965) (camc314)
- 0cef370 ast: Add `AstKind::TemplateElement` (#11955) (camchenry)

### 📚 Documentation

- 81ff679 ast: Improve docs for TS types (#12053) (overlookmotel)
- 4a408c3 napi/parser: Document options (#12008) (overlookmotel)

### ⚡ Performance

- 735c0d3 ast: Call `Expression::without_parentheses` only once (#12056) (overlookmotel)


## [0.75.0] - 2025-06-25

### 💥 BREAKING CHANGES

- 9a2548a napi/parser: [**BREAKING**] Add `range` option (#11728) (Bacary Bruno Bodian)

### 🐛 Bug Fixes

- fe78dcf ast: `MemberExpression::static_property_info` use `cooked` not `raw` for `TemplateElement`s (#11879) (overlookmotel)

### 🚜 Refactor

- 87b8496 ast: Remove `AstKind` for `MemberExpression` and replace with `StaticMemberExpression` and `PrivateFieldExpression` (#11767) (camchenry)
- 190e390 ast: Add `AstKind` for `ComputedMemberExpression` (#11766) (camchenry)

### ⚡ Performance

- 5500d2d ast: Remove redundant checks from `MemberExpression::static_property_info` and `ComputedMemberExpression::static_property_name` (#11882) (overlookmotel)


## [0.74.0] - 2025-06-23

### 🐛 Bug Fixes

- 8627ced ast: `ComputedMemberExpression::static_property_name` use `cooked` for `TemplateElement`s (#11829) (overlookmotel)

### 🚜 Refactor

- 08e666f ast/estree: Add `#[estree]` attrs to `RegExpFlagsAlias` (#11794) (overlookmotel)



## [0.73.1] - 2025-06-17

### 🚀 Features

- 371473c ast, semantic: Implement `GetAddress` for `AstKind` and `AstNode` (#11758) (overlookmotel)
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

### 🐛 Bug Fixes

- 6feab7e ast/estree: Remove custom serializer for `MethodDefinition` `key` field (#11763) (overlookmotel)

### 🚜 Refactor

- 3d89012 ast: Add `AstKind` for `TSTypeOperator` (#11747) (camchenry)
- 5ca3d04 ast: Add `TSArrayType` as `AstKind` (#11745) (camchenry)
- 4fbe4b1 ast: Remove AstKind from `TSModuleReference` node (#11732) (therewillbecode)
- 219adcc ast: Don't generate AstKind for ArrayExpressionElement (#11684) (Ulrich Stark)


## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- 3b03fd3 parser: Produce correct syntax error for `interface I extends (typeof T)` (#11610) (Boshen)

### 🚜 Refactor

- 8e30c5f ast: Don't generate AstKind for ForStatementInit (#11617) (Ulrich Stark)
- 9136685 ast: Create AstKinds for jsdoc types (#11597) (Ulrich Stark)
- d41fb13 ast: Get jsx types out of AstKind exceptions (#11535) (Ulrich Stark)

### ⚡ Performance

- d5866e8 ast/estree: Convert `NumericLiteral` `raw` field to JSON without escaping (#11562) (overlookmotel)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Features

- 590b809 span: Add `Atom::from_cow_in` method (#11480) (overlookmotel)

### Bug Fixes

- 90b8b33 ast: Restore `#[ts]` attribute to `FormalParameter` `decorators` field (#11497) (overlookmotel)
- f79b43a ast: Emit parameter decorators in js estree (#11494) (Boshen)
- 8451bee ast/estree: Remove repeat fields from `BindingPattern` in TS-ESTree AST (#11500) (overlookmotel)
- 953e61b ast/estree: Fix field order of `PropertyKey` constructor in raw transfer TS-ESTree AST (#11463) (overlookmotel)
- ec4fc83 estree: Use consistent field order in serialization (#11385) (Yuji Sugiura)

### Documentation

- ca3214d ast: Correct and improve docs for `BindingPattern` (#11499) (overlookmotel)

## [0.72.2] - 2025-05-31

### Features

- 1df6732 ast/estree: Add decorators to ESTree AST (#11393) (overlookmotel)

### Documentation

- a30cd3f ast: Fix typo in docs for `TSPropertySignature` (#11352) (overlookmotel)

### Refactor

- 12690a1 ast/estree: Re-order fields in visitation order (#11362) (overlookmotel)
- 75ee3a5 ast/estree: Do not put TS struct fields last (#11360) (overlookmotel)
- 4c49274 parser: Rewrite import/export specifier parsing (#11356) (camchenry)

## [0.72.1] - 2025-05-28

### Features

- f88f666 minifier: Normalize `Number.NaN` as `NaN` (#11275) (Boshen)

### Refactor

- 3a49220 ast: Remove `AstKind::get_container_scope_id` (#11282) (Dunqing)

## [0.72.0] - 2025-05-24

### Features

- 03390ad allocator: `TakeIn` trait with `AllocatorAccessor` (#11201) (Boshen)
- 23182b8 ast/estree: Add `phase` field to `ImportExpression` in TS-ESTree AST (#11193) (overlookmotel)
- 4feeeee span: Add `Atom::from_strs_array_in` method (#11261) (overlookmotel)
- c2c0268 syntax: Introduce `CommentNodeId` (#11214) (overlookmotel)

### Refactor

- f05c506 ast: Move `AstBuilder` type definition (#11238) (overlookmotel)

## [0.71.0] - 2025-05-20

- 1a4fec0 codegen: [**BREAKING**] A legal comment can also be a jsdoc comment (#11158) (Boshen)

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- c60382d allocator/vec2: Change `len` and `cap` fields from `usize` to `u32` (#10884) (Dunqing)
- d47b305 ast/estree: Add `phase` field to `ImportExpression` in ESTree AST (#11165) (overlookmotel)
- 1bc8d29 ast/estree: Add `phase` field to `ImportDeclaration` in ESTree AST (#11157) (overlookmotel)
- 9e90e00 ast_tools: Introduce `#[js_only]` attr for struct fields and converters (#11160) (overlookmotel)

### Bug Fixes


### Performance

- 6571b9b ast: Use bitflags for storing comment newline state (#11096) (camchenry)
- 2b0a69f ast: Re-order struct fields to reduce padding (#11056) (overlookmotel)
- b9e51e2 ast: Reduce size of `Comment` to 16 bytes (#11062) (camchenry)

### Documentation

- e92bf1f napi: Update docs for `oxc-parser` (#11156) (overlookmotel)

### Refactor

- 20a4a8c ast: Shorten `CommentNewlines` methods (#11152) (overlookmotel)
- 58c7de6 ast: Rename `CommentNewlines` fields (#11151) (overlookmotel)
- e5438dc ast: Inline `Comment` newline flags setters (#11150) (overlookmotel)

### Styling

- b3519ca ast: Remove dummy comment (#11162) (overlookmotel)

## [0.70.0] - 2025-05-15

### Features

- 1673ffb codegen: Rework printing normal / legal / annotation comments (#10997) (Boshen)

### Bug Fixes

- 6f3f9d7 ast/estree: Fix `raw_deser` for `TSMappedTypeOptional` serializer (#10971) (overlookmotel)
- 53329f8 ast/estree: Fix field order for `FormalParameter` (#10962) (overlookmotel)
- 8b8f78f ast/estree: Fix field order and type def for `RestElement` in `FormalParameters` (#10961) (overlookmotel)
- 2b76ab5 ast/estree: Fix `TSModuleDeclaration` raw deserializer (#10924) (overlookmotel)
- d036cf5 estree: Ensure the same key order for `AssignmentPattern` (#10953) (Yuji Sugiura)

### Performance

- a4b5716 ast/estree: Streamline raw deserializer for `WithClause` (#10974) (overlookmotel)

### Documentation

- 206e07d ast: Fix docs for `ExportSpecifier` (#10891) (overlookmotel)
- b23fef7 ast/estree: Standardize doc comments for ESTree serializers (#10968) (overlookmotel)

### Refactor

- 6cc74ff ast/estree: Add semi-colon (#10973) (overlookmotel)
- 2958ff7 ast/estree: Rename serializers (#10969) (overlookmotel)
- b8018b9 ast/estree: Split custom serializers into multiple files (#10967) (overlookmotel)
- 08f1f3f ast/estree: Remove custom serializer for `Elision` (#10965) (overlookmotel)
- d01a47b ast/estree: Simplify custom serializers for `FormalParameters` (#10964) (overlookmotel)
- bfc6b9a ast/estree: Rename custom serializers for fields containing `FormalParameters` (#10963) (overlookmotel)

### Styling

- 6d4936f ast/estree: Reformat `raw_deser` code (#10972) (overlookmotel)

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

- d066516 ast_tools: Support `#[estree(prepend_to)]` (#10849) (overlookmotel)

### Bug Fixes

- 2c09243 ast: Fix field order for `AccessorProperty` (#10878) (overlookmotel)
- 581d068 ast/estree: Fix TS type def for `RegExpLiteral` (#10876) (overlookmotel)
- e7228fa ast/estree: Fix `optional` field of `TSMappedType` in TS-ESTree AST (#10874) (overlookmotel)
- 6f0638a ast/estree: Remove `TSImportTypeOptions` custom serializer (#10873) (overlookmotel)
- e6657ae ast/estree: Reorder fields for TS `Identifier` types in TS-ESTree AST (#10864) (overlookmotel)

### Performance

- 49a6f97 napi/parser: Faster fixup of `BigInt`s and `RegExp`s (#10820) (overlookmotel)

### Documentation

- 4863eef ast: Improve docs for `TSImportType` (#10903) (overlookmotel)

### Refactor

- 805735b ast: Remove quotes from `#[estree(append_to)]` attributes (#10863) (overlookmotel)
- 0f7e755 ast/estree: Use `#[estree(prepend_to)]` (#10862) (overlookmotel)
- b16331e ast/estree: Generalize concatenating fields with `Concat2` (#10848) (overlookmotel)
- daba0a7 estree: Remove regular expression types from ESTree AST (#10855) (overlookmotel)

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.68.1] - 2025-05-04

### Bug Fixes

- 663fb53 ast/estree: For TS type def for `BigIntLiteral` (#10786) (overlookmotel)
- c33eb9c ast/estree: Fix raw deser for `TSTypeReference` (#10787) (overlookmotel)

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

- a0a37e0 ast: [**BREAKING**] `AstBuilder` methods require an `Atom` with correct lifetime (#10735) (overlookmotel)

### Features

- 60347e7 ast_tools: `AstBuilder` generator support types with more than 1 `Atom` field (#10737) (overlookmotel)

### Bug Fixes

- 8bfd5e3 ast: `AssignmentTargetMaybeDefault::identifier` preserve lifetime (#10720) (overlookmotel)
- 61d825b ast/estree: Rename `assert` to `with` in `TSImportType` `options` in TS-ESTree AST (#10681) (overlookmotel)

### Refactor

- 050ecd9 ast/estree: Remove custom serializer for `TSMappedTypeModifierOperator` (#10747) (overlookmotel)

### Styling

- c18c81f ast: Import `Display` trait (#10724) (overlookmotel)

## [0.67.0] - 2025-04-27

### Bug Fixes

- 24ab2f3 ast/estree: Convert `TSClassImplements::expression` to `MemberExpression` in TS-ESTree AST (#10607) (overlookmotel)
- 0825834 ast/estree: Correct `this` in `TSTypeName` in TS-ESTree AST (#10603) (overlookmotel)
- d1f5abb ast/estree: Fix TS-ESTree AST for `TSModuleDeclaration` (#10574) (overlookmotel)
- 66e384c ast/estree: Add missing fields to `ObjectPattern` in TS-ESTree AST (#10570) (overlookmotel)

### Styling

- 02300aa ast/estree: Add semicolons to end of lines in `raw_deser` impls (#10563) (overlookmotel)

## [0.66.0] - 2025-04-23

### Bug Fixes

- 43ad4e9 ast: Box `this_param` in `TSCallSignatureDeclaration` (#10558) (Yuji Sugiura)
- 8eb3c0a ast/estree: Fix raw deser for `FormalParameter` (#10548) (overlookmotel)
- f19b287 estree: Add `TSParameterProperty` (#10534) (Yuji Sugiura)

## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

- d6b7982 ast: [**BREAKING**] Improve pluralization of `TSClassImplements` (#10489) (overlookmotel)

### Bug Fixes

- 1952e30 ast/estree: Serialize class constructor key as `Identifier` in TS-ESTree AST (#10471) (overlookmotel)
- fbf0ae2 estree: Add missing fixed fields to `AssignmentPattern` (#10490) (Yuji Sugiura)
- a42d85f estree: `FormalParameters` serializer for TS types (#10462) (Yuji Sugiura)

### Refactor

- cfdfa2e ast/estree: Move `FunctionFormalParameters` serializer to be with other serializers for `FormalParameters` (#10496) (overlookmotel)
- 852c8a7 ast/estree: Improve readability of custom serializers (#10474) (overlookmotel)

## [0.64.0] - 2025-04-17

- 09f4c30 ast: [**BREAKING**] Remove `ArrayAssignmentTarget::new_with_elements` and `ObjectAssignmentTarget::new_with_properties` (#10440) (overlookmotel)

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)
- 2c66ac2 codegen: Preserve code coverage ignore comments (e.g. `v8 ignore`) (#10338) (Boshen)

### Bug Fixes

- f3ddefb ast/estree: Add missing fields to `AssignmentTargetRest` in TS-ESTree AST (#10456) (overlookmotel)
- 77b6f7e ast/estree: Fix start span of `Program` in TS-ESTree AST where first statement is `@dec export class C {}` (#10448) (overlookmotel)
- 4817c7e ast/estree: Add fields to `AssignmentTargetPattern` in TS-ESTree AST (#10423) (overlookmotel)
- b3094b3 ast/estree: Add `optional` field to `AssignmentTargetProperty` in TS-ESTree AST (#10412) (overlookmotel)
- a7fd30f ast/estree: Add fields to `BindingRestElement` in TS-ESTree AST (#10411) (overlookmotel)
- 9d82402 ast/estree: Fix `Program` span when contains no statements (#10408) (overlookmotel)
- cc07efd ast/estree: Fix `JSXOpeningFragment` (#10208) (therewillbecode)
- 48ed6a1 ast/estree: Fix span for `TemplateElement` in TS AST (#10315) (overlookmotel)
- 2520b25 estree: Align `TSMappedType` fields (#10392) (Yuji Sugiura)
- 3ed3669 estree: Rename `JSDocXxxType` to `TSJSDocXxxType` (#10358) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)
- 5850a0d parse: `type x = typeof import('')` -> ` TSTypeQuery(TSImportType)` (#10317) (Boshen)
- dd2aef0 semantic: Incorrect `SymbolFlags` of `TSModuleDeclaration` (#10350) (Dunqing)

### Documentation

- c6f08d0 ast: Improve docs for `AssignmentTargetRest` (#10442) (overlookmotel)

### Refactor

- 192717c ast: `ModuleDeclaration::with_clause` return `Option<&WithClause>` (#10441) (overlookmotel)
- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)
- 294d24b ast/estree: Simplify serialization for `JSXOpeningFragment` (#10316) (overlookmotel)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

- 5ad41c9 ast: [**BREAKING**] Remove unnecessary `AstBuilder::alloc_*` methods (#10267) (overlookmotel)

- 14ff99c ast: [**BREAKING**] Remove all `AstBuilder::move_xxxx` methods (#10171) (Dunqing)

### Features

- 8bbaedc ast_tools: Derive `Hash` for `AstType` (#10212) (lazytype)

### Bug Fixes

- 08a0d5e ast: Fix `Display` impl for `RegExpFlags` (#10235) (overlookmotel)
- e42c040 ast/estree: Add TS fields to `LabelIdentifier` (#10295) (overlookmotel)
- 06fc07c ast/estree: Fix `TSImportType` (#10200) (therewillbecode)
- 760188e ast/estree: Fix `BindingProperty` (#10193) (therewillbecode)
- db0b2eb ast/estree: Add ts-estree fields to `FormalParametersRest` custom serializer (#10192) (therewillbecode)
- abc9ea4 ast/estree: Better align program span start (#10134) (therewillbecode)
- f547d76 ast/estree: Add `TSEnumBody` to `TSEnumDeclaration.body` (#10017) (Yuji Sugiura)
- 34d5c00 ast/estree: Fix `ExportDefaultDeclaration` node (#10165) (therewillbecode)
- 498b479 ast/estree: Fix `AccessorProperty` node (#10067) (therewillbecode)
- bf90072 ast/estree: Fix `ObjectProperty` node (#10018) (therewillbecode)
- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)

### Performance

- 5b4c2ae ast: Remove `write!` macro where unnecessary (#10234) (overlookmotel)

### Documentation

- 4d33af7 ast: Correct comment (#10176) (overlookmotel)

### Refactor

- 5525206 ast/estree: Simplify static converters (#10197) (overlookmotel)
- b662df4 ast/estree: Alter `Program` start span with converter (#10195) (overlookmotel)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)
- 2dde8be ast: Reformat AST types (#10278) (overlookmotel)
- c290faa ast: Reformat `AstBuilder` doc comments (#10263) (overlookmotel)

## [0.62.0] - 2025-04-01

- 45015ee ast: [**BREAKING**] Remove unused `Function::symbol_id` method (#10061) (Dunqing)

### Features

- 8cd7430 allocator: `TakeIn` trait (#9969) (overlookmotel)

### Bug Fixes

- 95e69f6 ast/estree: Fix `StringLiteral`s containing lone surrogates (#10036) (overlookmotel)
- 8408606 ast/estree: Fix `TSMethodSignature` (#10032) (therewillbecode)
- 1a0bd7c ast/estree: Fix `TSPropertySignature` (#10031) (therewillbecode)
- 707a776 ast/estree: Fix TS type defs for `TSIndexSignature` and `TSIndexSignatureName` (#10003) (overlookmotel)
- c98d3f4 ast/estree: Add custom serializer for extends field of TSInterfaceDeclaration (#9996) (therewillbecode)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Refactor

- a2b03d3 ast: Methods for fieldless enums take `self` not `&self` + remove unnecessary `matches!` (#10013) (overlookmotel)
- 5f45005 ast: `AstBuilder::move_*` methods use `TakeIn` (#10001) (overlookmotel)
- 0fa58d7 semantic: Always use `SymbolFlags::Function` for function id (#7479) (Dunqing)

## [0.61.2] - 2025-03-23

### Bug Fixes

- 89cb368 ast/estree: Add decorators field to `AssignmentPattern` (#9967) (therewillbecode)
- 4980b73 ast/estree: Add missing estree fields to `TSIndexSignature` and `TSIndexSignatureName` (#9968) (therewillbecode)
- b9f80b9 ast/estree: Fix `TSFunctionType` and `TSCallSignatureDeclaration`  (#9959) (therewillbecode)
- 0cdeedd ast/estree: Fix `ArrayPattern` (#9956) (therewillbecode)
- 6fcd342 ast/estree: Fix `FormalParameter` (#9954) (therewillbecode)
- 9d1035e ast/estree: Fix TS type def for `TSThisParameter` (#9942) (overlookmotel)
- 8228b74 ast/estree: Fix `Function.this_param` (#9913) (hi-ogawa)
- d69cc34 ast/estree: Fix `BindingIdentifier` (#9822) (hi-ogawa)
- 5631ebd ast/extree: Fix `TSModuleDeclaration.global` (#9941) (overlookmotel)

### Refactor

- d3d7d98 ast: Shorten generated code for `CloneIn` (#9939) (overlookmotel)
- db642eb ast/estree: Shorten raw deser code (#9944) (overlookmotel)
- dc3e725 ast/estree: Expose `INCLUDE_TS_FIELDS` constant on `Serializer` (#9943) (overlookmotel)
- 28179cd ast_tools: Simplify `CloneIn` derive (#9938) (overlookmotel)
- 30990d2 ast_tools/estree: Re-order custom serializers (#9945) (overlookmotel)

## [0.61.1] - 2025-03-21

### Features

- bc0670c tasks,oxc_allocator: Add new method clone_in_with_semantic_ids for `CloneIn` trait (#9894) (IWANABETHATGUY)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

### Bug Fixes

- 28a2ed3 estree/ast: Fix `IdentifierName` and `IdentifierReference` (#9863) (hi-ogawa)- 68018e1 Ast changes (Boshen)

### Performance

- 5f97f28 ast/estree: Speed up raw deser for `JSXElement` (#9895) (overlookmotel)

### Refactor

- 62e2859 ast/ast_builder: Use `self.vec_from_iter` instead of `Vec::from_iter_in` for consistency (#9909) (Dunqing)

## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


### Bug Fixes

- bc8bc08 semantic: Use correct scope flags for using declarations (#9751) (camc314)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)
- 8abb4f6 parser: Correctly set `export_kind` for `ExportNamedDeclaration` (#9827) (camc314)

### Refactor


## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features


### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)
- 0d1e2c5 transformer: Fix `TSInstantiationExpression` not being removed (#9725) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 554c4ce minifier: Compress constant integer index access (#9604) (sapphi-red)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

## [0.56.4] - 2025-03-07

### Refactor

- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

- 48a0394 ast: [**BREAKING**] Add `scope_id` to `TSFunctionType` (#9559) (camc314)

### Features


### Bug Fixes

- bbb4f98 semantic: Insert binding into correct scope for TSInferType (#9567) (camc314)

## [0.55.0] - 2025-03-05

- 4056560 ast/estree: [**BREAKING**] Option to return JS-only AST (#9520) (overlookmotel)

### Features

- 59a9f1d ast: Add `AstBuilder::*_with_pure` etc methods (#9417) (Dunqing)
- af02a87 ast/estree: `Property` have consistent field order (#9547) (overlookmotel)
- 3e4f909 ast/estree: ESTree AST `ExportNamedDeclaration` always have `attributes` field (#9546) (overlookmotel)
- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

### Performance

- 6b4a8c6 ast, codegen, transformer: Avoid allocations when converting `RegExpFlags` to string (#9550) (overlookmotel)

### Refactor

- 734b6b6 ast: `RegExpFlags::to_string` return flags in alphabetical order (#9548) (overlookmotel)
- c1a8cea ast/estree: Simplify serializing `RegExpLiteral`s (#9551) (overlookmotel)

## [0.54.0] - 2025-03-04

- 098f652 codegen: [**BREAKING**] Add `CommentAnnotation` to avoid parsing comments again (#9506) (Boshen)

- a8d1d48 parser,codegen: [**BREAKING**] Parse and print`#__NO_SIDE_EFFECTS__` (#9496) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

- abb15e0 ast: [**BREAKING**] Add `pure` field to `Function`, `CallExpression`, and `NewExpression` (#9207) (overlookmotel)

### Features

- 26fde56 minifier: Inline simple IIFEs in `remove_unused_expression` (#9465) (sapphi-red)
- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)
- 9b7017c parser,codegen: Pure annotations (#9351) (Boshen)

### Bug Fixes

- b371fdc estree/ast: Camel-case `typeAnnotation` field (#9420) (overlookmotel)

### Performance

- 10e4431 ast/estree: Skip escaping identifiers (#9398) (overlookmotel)
- b0a0a82 ast/estree: Reduce overhead serializing static strings (#9396) (overlookmotel)

### Refactor

- 8916335 ast/estree: Convert enums with converters on variants (#9438) (overlookmotel)
- dcff40c ast_tools: Generate layout assertions in multiple crates (#9448) (overlookmotel)

## [0.53.0] - 2025-02-26

### Features

- 5c775ea ast/estree: Enable serialization without TS fields (#9285) (overlookmotel)

### Bug Fixes

- 6a8f53f ast/estree: Visit `JSXOpeningFragment` and `JSXClosingFragment` (#9342) (overlookmotel)
- e303767 ast/estree: Fix ESTree AST for imports and exports (#9282) (overlookmotel)

### Performance

- 82adab9 ast/estree: Speed up building UTF8-UTF16 translation table with SIMD (#9359) (overlookmotel)
- 61939ca ast/estree: Faster UTF-8 to UTF-16 span conversion (#9349) (overlookmotel)
- 1bfc459 ast/estree: Pre-allocate `CodeBuffer` for JSON output (#9340) (overlookmotel)

### Refactor

- 7427900 ast: Re-order `ExportDefaultDeclaration` fields (#9348) (overlookmotel)
- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)
- 55ed1df ast/estree: Shorten `ESTree` impls for enums (#9275) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

- 611b029 ast: [**BREAKING**] Remove `AstBuilder::copy` (#9210) (Dunqing)

### Features


### Bug Fixes

- 72bab88 ast/estree: Remove unused TS type def for `WithClause` (#9250) (overlookmotel)

### Documentation

- d4d01c3 ast: Correct and improve docs for `get_identifier` methods on JSX types (#9225) (overlookmotel)
- 70726e9 ast: Docs for `AstBuilder` `*_with_*` methods list extra field names, not field types (#9206) (overlookmotel)

### Refactor

- 97cc1c8 ast: Remove `TSLiteral::NullLiteral` (replaced by `TSNullKeyword`) (#9147) (Boshen)
- 46b9151 ast/estree: Simplify serializer for `BindingProperty` (#9255) (overlookmotel)
- 2e2823b ast/estree: Define custom ESTree serializers on struct fields via meta types (#9251) (overlookmotel)
- 3236fe4 ast/estree: Simplify serializing `FormalParameters` (#9195) (overlookmotel)

## [0.51.0] - 2025-02-15

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

- 9091387 ast: [**BREAKING**] Remove `TSType::TSQualifiedName` (#9051) (Dunqing)

### Features


### Bug Fixes

- 38f81af ast/estree: Order fields same as Acorn (#9128) (overlookmotel)
- 67f8932 ast/estree: `CatchParameter` do not include `type` and `Span` twice (#9125) (overlookmotel)
- 1b02fe0 ast/estree: `FormalParameter` do not include `Span` twice (#9124) (overlookmotel)
- d3b5fb0 ast/estree: Fix TS type for `AssignmentTargetPropertyIdentifier` (#9092) (overlookmotel)
- d8d80a9 ast/estree: Fix TS types for `BigIntLiteral` and `RegExpLiteral` (#9091) (overlookmotel)
- 15f23f1 napi/parser: Utf16 span for module record (#9093) (hi-ogawa)
- 9edfb1d napi/parser: Fix unicode comment panic (#9084) (hi-ogawa)

### Documentation

- 896d770 ast: Add missing docs and fix broken code for assignment targets (#9095) (camchenry)

### Refactor

- 08b2d80 ast/estree: Define TS types for extra fields on converters (#9118) (overlookmotel)
- 6c5a435 ast/estree: Define types for `#[estree(add_fields)]` converters (#9116) (overlookmotel)

## [0.50.0] - 2025-02-12

- d9189f1 ast: [**BREAKING**] Remove `PrivateInExpression::operator` field (#9041) (overlookmotel)

### Bug Fixes

- 22d93be ast: Estree compat `AssignmentTargetPropertyIdentifier` (#9006) (hi-ogawa)
- cd2e199 ast/estree: Fix serializing `RegExpLiteral` (#9043) (overlookmotel)
- f705c64 ast/estree: Serialize `PrivateInExpression` as `BinaryExpression` (#9033) (hi-ogawa)
- 2948804 ast/estree: Fix `ExportAllDeclaration` attributes (#9032) (hi-ogawa)
- 2371dd4 ast/estree: Fix serializing import and export `attributes` (#9030) (hi-ogawa)
- 2b47299 ast/estree: Fix serializing `RegExpLiteral` flags (#9027) (overlookmotel)
- e75e1d2 ast/estree: Fix serializing `PrivateFieldExpression` (#9025) (overlookmotel)
- fcb5490 ast/estree: Fix serializing `ImportExpression`s (#9024) (overlookmotel)
- cac5545 ast/estree: Fix serializing `BigInt`s (#9022) (overlookmotel)
- 9427007 ast/estree: Use `#[estree(append_to)]` for `TSModuleBlock` (#9020) (overlookmotel)

### Documentation

- 11829bf ast: Improve and reformat doc comments (#9017) (overlookmotel)

### Refactor

- cb3240c ast/estree: Remove redundant `ts_type` (#9037) (hi-ogawa)
- eef8874 ast/estree: Re-arrange and comment custom serialization code (#9019) (overlookmotel)
- 8db4727 ast/estree: Remove serialization wrapper (#9018) (overlookmotel)

### Styling

- 50a87c2 ast: Reformat `#[estree]` attribute (#9016) (overlookmotel)

## [0.49.0] - 2025-02-10

- bbb075d ast: [**BREAKING**] Name `AstBuilder` enum builders after variant name not type name (#8890) (overlookmotel)

- b7ff7e1 span: [**BREAKING**] Export `ContentEq` trait from root of `oxc_span` crate (#8869) (overlookmotel)

### Features

- c940826 ast_tools: Get extra params for visitor methods from `#[visit(args)]` attr (#8887) (overlookmotel)
- f2d28f3 transformer: Support for transforming legacy decorator (#8614) (Dunqing)

### Bug Fixes

- 1daa8fe ast: Estree compat `AssignmentTargetPropertyProperty` (#9005) (hi-ogawa)
- e0646d7 ast: Estree compat `ArrayAssignmentTarget` (#8998) (hi-ogawa)
- d7802a7 ast: Serialize `ArrowFunctionExpression.body: FunctionBody | Expression` (#8988) (Hiroshi Ogawa)
- 7e6a537 ast: Include `directives` in `body` (#8981) (hi-ogawa)
- ec1d593 ast: Add missing estree props for `ArrowFunctionExpression` (#8980) (camchenry)
- 8eccdec ast: Estree compat `CatchClause` (#8975) (hi-ogawa)
- 2ee1d6c ast: Estree compat `Property` (#8974) (hi-ogawa)
- 801d78e ast: Estree compat `UnaryExpression` (#8973) (hi-ogawa)
- a2883b1 ast: Estree compat `Function` (#8972) (hi-ogawa)
- a520986 ast: Estree compat `Program.sourceType` (#8919) (Hiroshi Ogawa)
- e30cf6a ast: Estree compat `MemberExpression` (#8921) (Hiroshi Ogawa)
- 315fec6 ast: Fix lifetimes on custom `Serialize` impls (#8899) (overlookmotel)
- 0c55dd6 ast: Serialize `Function.params` like estree (#8772) (Hiroshi Ogawa)
- 9a5a926 ecmascript: Fix may_have_side_effects for binary expressions (#8991) (sapphi-red)

### Performance

- fcfc274 ast: Assume `serde_json` output is valid UTF8 string (#8928) (overlookmotel)

### Documentation

- 705208b ast: Do not include `r#` prefix in doc comments for `AstBuilder` methods (#8873) (overlookmotel)

### Refactor

- 0929320 ast: Add `NULL` value for serializing null types (#8987) (camchenry)
- a1ca2eb ast: Support `#[estree(ts_alias)]` attr on enums and use it on `RegExpPattern` and `RegExpFlags` (#8953) (overlookmotel)
- 515eb52 ast: Shorten code for `impl Serialize for Elision` (#8940) (overlookmotel)
- 640db88 ast: Introduce `#[estree(ts_alias)]` attr and use it on `Elision` (#8939) (overlookmotel)
- a6884e4 ast: Simplify serializing literal types (#8937) (overlookmotel)
- c58f785 ast: Simplify serializing `SourceType` (#8936) (overlookmotel)
- 647a288 ast: Rename `#[estree(add_entry)]` to `#[estree(add_fields)]` (#8926) (overlookmotel)
- cbb4e9c ast: Generated `Serialize` impls flatten struct fields (#8904) (overlookmotel)
- 5cb8466 ast: Estree `via` on struct fields implement `From` (#8898) (overlookmotel)
- 223eb8d ast: Override TS type defs with `#[estree(custom_ts_def)]` attribute on type (#8897) (overlookmotel)
- a36bc5f ast: Generate `Visit` methods in order of type definitions (#8891) (overlookmotel)
- 893339d ast: Record plural names in `#[plural]` attr (#8889) (overlookmotel)
- 5f5188c ast: Shorten generated code for `GetSpan` (#8883) (overlookmotel)
- 571fb70 ast: Derive `ContentEq` on literal types (#8880) (overlookmotel)
- 04786ac ast: Use `=` syntax for `#[scope]` attrs (#8878) (overlookmotel)
- 6628e1d ast: Move `#[estree(skip)]` to types (#8877) (overlookmotel)
- 3407d47 ast: Move `#[clone_in(default)]` to types (#8876) (overlookmotel)
- 6839433 ast: Communicate type of `RegExpFlags` to `oxc_ast_tools` with alias type (#8868) (overlookmotel)
- abfe5bf ast: Shorten generated code for numbers (#8864) (overlookmotel)
- f6f92db ast: Re-order generated code (#8863) (overlookmotel)
- f69de07 ast: Remove unneeded lint attrs from generated code (#8862) (overlookmotel)
- 6d1e1d8 ast: Make generated code consistent (#8872) (overlookmotel)
- a5bdf32 ast_tools: Re-vamp `oxc_ast_tools` (#8844) (overlookmotel)
- 768ad4f syntax: Add `#[ast]` attr to semantic ID types (#8867) (overlookmotel)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.48.2] - 2025-02-02

### Features

- d553318 minifier: Complete `MangleIf` (#8810) (Boshen)
- e353a01 minifier: Compress `a != null ? a.b : undefined` to `a?.b` (#8802) (sapphi-red)

### Documentation

- 57b7ca8 ast: Add documentation for all remaining JS AST methods (#8820) (Cam McHenry)
- b00b8c8 ast: Correct documentation for `JSXExpression::EmptyExpression`  (#8816) (Dunqing)
- c63291a ast: Add more docs for JS expressions, declarations, and module AST types (#8800) (Cam McHenry)
- fb5b1fa ast: Reformat `AstBuilder` doc comments (#8774) (overlookmotel)

### Refactor

- 0568210 ast: Remove excess line breaks from generated code (#8830) (overlookmotel)
- 30eec26 ast: Make generated code for `Visit` more understandable (#8825) (overlookmotel)
- d4eee50 ast: Comments for enums with no `AstKind` in generated code for `Visit` trait (#8796) (overlookmotel)
- 87a7711 ast: Shorten generated code for `VisitMut` (#8795) (overlookmotel)
- 70ad879 ast: Remove unnecessary lint from generated code for `AstKind` (#8794) (overlookmotel)
- beeda9a ast: Alter comments in generated `Visit` trait (#8793) (overlookmotel)
- 8cf9d34 ast: Rename `#[estree(type)]` attr on struct fields to `#[estree(ts_type)]` (#8767) (overlookmotel)
- a316b10 ast: Rename `#[estree(type)]` attr on types to `#[estree(rename)]` (#8766) (overlookmotel)
- a861d93 minifier: Port esbuild's `mangleStmts` (#8770) (Boshen)

## [0.48.1] - 2025-01-26

### Features

- b7f13e6 ast: Implement utf8 to utf16 span converter (#8687) (Boshen)
- e0117db minifier: Replace `const` with `let` for non-exported read-only variables (#8733) (sapphi-red)

## [0.48.0] - 2025-01-24

### Refactor

- 997859c ast: Align `#[estree(via)]` behavior (#8599) (sapphi-red)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.47.0] - 2025-01-18

- 19d3677 ast: [**BREAKING**] Always return `Array<ImportDeclarationSpecifier>` for `ImportDeclaration.specifiers` (#8560) (sapphi-red)

- 7066d1c ast, span, syntax, regular_expression: [**BREAKING**] Remove `ContentHash` (#8512) (overlookmotel)

### Features

- a6d71f8 ast: Add `AstKind::ty` method (#8521) (overlookmotel)

### Bug Fixes

- 855c839 codegen: Shorthand assignment target identifier consider mangled names (#8536) (Boshen)

### Performance

- 3fff7d2 span: Align `Span` same as `usize` (#8298) (overlookmotel)

### Refactor

- ac05134 allocator: `String` type (#8568) (overlookmotel)
- fcbca32 ast: Rename `#[estree(with)]` to `#[estree(via)]` (#8564) (overlookmotel)
- 007e8c0 ast, regular_expression: Shorten `ContentEq` implementations (#8519) (overlookmotel)

## [0.46.0] - 2025-01-14

- 7eb6ccd ast: [**BREAKING**] Remove unused and not useful `ContentHash` (#8483) (Boshen)

### Features


## [0.45.0] - 2025-01-11

- 7f69561 ast: [**BREAKING**] `oxc_ast` do not export `BigUint` (#8428) (overlookmotel)

- d8b27af ast: [**BREAKING**] No unneccesary trailing underscores on `AstBuilder` method names (#8283) (overlookmotel)

- 5106088 ast: [**BREAKING**] Remove `FromIn<Expression> for Statement` (#8280) (overlookmotel)

### Features

- 3212bcd ast_tools: Ignore `raw` field of `NumericLiteral` and `StringLiteral` in `ContentEq` (#8417) (Boshen)
- 8d52cd0 minifier: Merge assign expression in conditional expression (#8345) (sapphi-red)
- e84f267 minifier: Compress more property keys (#8253) (Boshen)
- ccdc039 minifier: Always put literals on the rhs of equal op `1==x` => `x==1` (#8240) (Boshen)
- 213364a minifier: Minimize `if (x) if (y) z` -> `if (x && y) z` (#8136) (Boshen)
- fef0b25 minifier: Collapse `var` into for loop initializer (#8119) (Boshen)

### Bug Fixes

- 97a7992 ast: Fix `ContentEq` and `ContentHash` impls for literal types (#8426) (overlookmotel)

### Documentation

- c8e4843 ast: Fix doc comment (#8286) (overlookmotel)

### Refactor

- b29655f ast: Rearrange impls for literal types in same order as they are defined (#8425) (overlookmotel)
- 0db2a22 ast: `AstBuilder` enum builder methods use `alloc_*` methods (#8281) (overlookmotel)
- aea9551 ast: Simplify `get_identifier_reference` of `TSType` and `TSTypeName` (#8273) (Dunqing)

## [0.44.0] - 2024-12-25

- ad2a620 ast: [**BREAKING**] Add missing `AssignmentTargetProperty::computed` (#8097) (Boshen)

### Features

- c2daa20 ast: Add `Expression::into_inner_expression` (#8048) (overlookmotel)

### Bug Fixes


## [0.43.0] - 2024-12-21

### Features

- 63a95e4 ast: Add `AstBulder::move_property_key` (#7998) (overlookmotel)

### Performance

- c0dd3f8 ast: `move_expression` and `move_statement` produce dummy with no span (#7995) (overlookmotel)

### Documentation

- df5c341 ast: Improve docs for `AstBuilder::move_*` methods (#7994) (overlookmotel)

## [0.42.0] - 2024-12-18

### Features

- 8b7c5ae ast: Add `AstBuilder::atom_from_cow` (#7974) (overlookmotel)
- c30a982 span: Add `impl From<ArenaString> for Atom` (#7973) (overlookmotel)
- 6bc530d transformer/class-properties: Transform super call expression that is inside static prop initializer (#7831) (Dunqing)

### Bug Fixes

- 3659e6d cfg: Include export default code in CFG instructions (#7862) (Jan Olaf Martin)

### Performance

- a5f04a7 ast: Faster `Comment::is_jsdoc` (#7905) (overlookmotel)

### Documentation

- e49de81 ast: Document `Expression::is_*` methods (#7853) (overlookmotel)

### Refactor

- beb982a ast: Use exhaustive match for `Argument` to `ArrayExpressionElement` conversion (#7848) (overlookmotel)
- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.41.0] - 2024-12-13

- fb325dc ast: [**BREAKING**] `span` field must be the first element (#7821) (Boshen)

- 96a26d3 ast: [**BREAKING**] Rename `is_strict` methods to `has_use_strict_directive` (#7783) (overlookmotel)

### Features

- 8991f33 ast: Add `visit_span` to `Visit` and `VisitMut` (#7816) (overlookmotel)
- f7900ab ast: Add `ArrowFunctionExpression::has_use_strict_directive` method (#7784) (overlookmotel)

### Refactor


## [0.40.0] - 2024-12-10

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- 7dcf6b4 ast, transformer: Add `AstBuilder::use_strict_directive` method (#7770) (overlookmotel)
- bd9d38a linter: Implement eslint:yoda (#7559) (tbashiyy)

### Bug Fixes

- 2179b93 estree: Make type of `BigIntLiteral::raw` prop in ESTree AST optional (#7663) (overlookmotel)
- cbba26c estree: `raw: null` in ESTree AST for generated `NullLiteral`s (#7662) (overlookmotel)
- 1d59fc8 estree: `raw: null` in ESTree AST for generated `BooleanLiteral`s (#7661) (overlookmotel)

### Refactor

- 98afe65 ast: `AstBuilder` extra methods use `SPAN` (#7769) (overlookmotel)
- 8993e89 ast: Shorten code (#7659) (overlookmotel)
- 746c8aa ast: Rename vars (#7658) (overlookmotel)

### Styling

- 0c9cc48 ast: Import `Atom` (#7657) (overlookmotel)

## [0.39.0] - 2024-12-04

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features


## [0.38.0] - 2024-11-26

### Features

- eb70219 ast: Derive `GetAddress` on all enum types (#7472) (overlookmotel)

## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 41a0e60 ast: [**BREAKING**] Remove `impl GetAddress for Function` (#7343) (overlookmotel)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)
- 897d3b1 ast: Serialize StringLiterals to ESTree without `raw` (#7263) (ottomated)
- 224775c transformer: Transform object rest spread (#7003) (Boshen)
- 885e37f transformer: Optional Chaining (#6990) (Boshen)

### Bug Fixes


### Performance

- c84e892 ast: `AstBuilder::vec1` use `Vec::from_array_in` (#7334) (overlookmotel)

### Documentation

- f0affa2 ast: Improve docs examples for `PropertyDefinition` (#7287) (overlookmotel)
- 740ba4b ast: Correct doc comment for `StringLiteral` (#7255) (overlookmotel)

### Refactor

- de472ca ast: Move `StringLiteral` definition higher up (#7270) (overlookmotel)
- d3d58f8 ast: Remove `inherit_variants!` from `TSEnumMemberName` (#7248) (overlookmotel)

### Styling

- 10cdce9 ast: Add line break (#7271) (overlookmotel)

## [0.36.0] - 2024-11-09

- b11ed2c ast: [**BREAKING**] Remove useless `ObjectProperty::init` field (#7220) (Boshen)

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- 092de67 types: [**BREAKING**] Append `rest` field into `elements` for objects and arrays to align with estree (#7212) (ottomated)

- d1d1874 ast: [**BREAKING**] Change `comment.span` to real position that contain `//` and `/*` (#7154) (Boshen)

- 843bce4 ast: [**BREAKING**] `IdentifierReference::reference_id` return `ReferenceId` (#7126) (overlookmotel)

### Features

- cc8a191 ast: Methods on AST nodes to get `scope_id` etc (#7127) (overlookmotel)
- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)
- 9d6cc9d estree: ESTree compatibility for all literals (#7152) (ottomated)

### Bug Fixes


### Refactor

- d27e14f ast: `AstKind::as_*` methods take `self` (#5546) (overlookmotel)
- fac5042 ast: Use `scope_id` etc methods (#7130) (overlookmotel)

## [0.35.0] - 2024-11-04

- f543a8d ast: [**BREAKING**] Remove `AstBuilder::*_from_*` methods (#7073) (overlookmotel)

### Features

- 854870e ast: Label AST fields with #[ts] (#6987) (ottomated)
- ce5b609 ast: Remove explicit untagged marker on enums (#6915) (ottomated)
- 9725e3c ast_tools: Add #[estree(always_flatten)] to Span (#6935) (ottomated)
- fbc297e ast_tools: Move tsify custom types to estree attribute macro (#6934) (ottomated)
- 169fa22 ast_tools: Default enums to rename_all = "camelCase" (#6933) (ottomated)
- 6516f9e codegen: Print inline legal comments (#7054) (Boshen)
- 1e2f012 linter: Add `oxc/no-map-spread` (#6751) (DonIsaac)

### Bug Fixes

- 0601271 ast: Fix `StaticMemberExpression.get_first_object` (#6969) (tomoya yanagibashi)
- f5a7134 linter/no-unused-vars: False positive for discarded reads within sequences (#6907) (DonIsaac)
- caaf00e parser: Fix incorrect parsed `TSIndexSignature` (#7016) (Boshen)

### Performance

- 6ca01b9 ast: Reduce size of `Comment` (#6921) (overlookmotel)

### Refactor

- b0211a1 ast: `StaticMemberExpression::get_first_object` use loop instead of recursion (#7065) (overlookmotel)
- fc07458 ast: Move custom types `.d.ts` file (#6931) (overlookmotel)
- c41c013 ast: Rename lifetime (#6922) (overlookmotel)
- 4cf0085 ast_tools: Reorder imports in generated code (#6926) (overlookmotel)
- 4688a06 transformer: Use `*_with_scope_id` builder methods where possible (#7055) (overlookmotel)
- df3b089 transformer/react-refresh: Use `StatementInjector` to insert statements (#6881) (Dunqing)

## [0.34.0] - 2024-10-26

### Features

- 1145341 ast_tools: Output typescript to a separate package (#6755) (ottomated)
- 0d0bb17 transformer: Complete the async-to-generator plugin (#6658) (Dunqing)

### Bug Fixes

- a47c70e minifier: Fix remaining runtime bugs (#6855) (Boshen)

### Documentation

- 6eeb0e6 ast: Mention typescript-eslint, field ordering and shape (#6863) (Boshen)

### Refactor

- 3e7507f ast_tools: Reduce macro usage (#6895) (overlookmotel)
- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

### Styling

- 262b2ed ast: Move crate doc comment to top of file (#6890) (overlookmotel)

## [0.33.0] - 2024-10-24

- 718ccde ast: [**BREAKING**] Remove unused `new` methods (#6789) (overlookmotel)

- 4d2d214 ast, transformer: [**BREAKING**] Remove `StringLiteral::new` method (#6788) (overlookmotel)

- a1ca964 ast, parser: [**BREAKING**] Remove `NumericLiteral::new` method (#6787) (overlookmotel)

- aeaa27a ast, parser, transformer, traverse: [**BREAKING**] Remove `BindingIdentifier::new` methods (#6786) (overlookmotel)

- ecc9151 ast, parser, transformer, traverse: [**BREAKING**] Remove `IdentifierReference::new` methods (#6785) (overlookmotel)

- c91ffbc ast, transformer: [**BREAKING**] Remove `IdentifierName::new` method (#6784) (overlookmotel)

- 2bee4e2 ast, transformer: [**BREAKING**] Remove `BlockStatement::new` methods (#6783) (overlookmotel)

- 1248557 ast: [**BREAKING**] Remove `AstKind::FinallyClause` (#6744) (Boshen)

- 202c7f6 ast: [**BREAKING**] Remove `AstKind::ExpressionArrayElement` and `AstKind::ClassHeritage` (#6740) (Boshen)

### Features

- 78fee6e ast: Add `AstBuilder::*_with_scope_id` etc methods (#6760) (overlookmotel)
- b2f3040 ast: Add `GetAddress` trait (#6652) (Dunqing)

### Bug Fixes

- 53049fe wasm: Remove type defs for `ArrayExpressionElement` and `Elision` (#6683) (overlookmotel)

### Documentation

- 63ce9be ast: Enable crate-wide warnings on missing doc comments (#6716) (DonIsaac)
- 91651e0 ast: Fix comment for `ClassElement::r#static` (#6771) (overlookmotel)
- c916505 ast: Fix comment of `ClassElement::r#static` (#6731) (_Kerman)
- 46720be ast: Improve formatting of `AstBuilder` doc comments (#6756) (overlookmotel)
- a7dd5aa ast: Enforce doc comments on AST node methods (#6714) (DonIsaac)
- 8d27e2d ast: Enforce doc comments on generated ASTBuilder methods (#6713) (DonIsaac)
- bad8770 ast: Enforce doc comments on JSX nodes, literal nodes, and comments (#6712) (DonIsaac)

### Refactor

- ab8aa2f allocator: Move `GetAddress` trait into `oxc_allocator` (#6738) (overlookmotel)
- b66ae2e ast: Move `impl GetAddress for Statement` (#6742) (overlookmotel)
- 0e9b695 ast: Change `plain_function` to accept `FunctionBody` as a required parameter (#6709) (Dunqing)
- 85e69a1 ast_tools: Add line breaks to generated code for `ESTree` derive (#6680) (overlookmotel)
- ad8e293 ast_tools: Shorten generated code for `impl Serialize` (#6684) (overlookmotel)
- 9ba2b0e ast_tools: Move `#[allow]` attrs to top of generated files (#6679) (overlookmotel)
- 11458a5 ast_tools: Shorten generated code by avoiding `ref` in matches (#6675) (overlookmotel)

## [0.32.0] - 2024-10-19

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

- 2808973 ast: [**BREAKING**] Add `Program::comments` (#6445) (Boshen)

### Features

- 6f22538 ecmascript: Add `ToBoolean`, `ToNumber`, `ToString` (#6502) (Boshen)
- 590925a minifier: Finish implementing folding array expressions (#6575) (camc314)
- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)
- b5b0af9 regular_expression: Support RegExp Modifiers (#6410) (leaysgur)

### Bug Fixes

- 02bfbfe codegen: Preserve parenthesis for `ChainExpression` (#6430) (Dunqing)
- a71e8a0 minifier: Preserve init variable declarations when removing `for` statements during DCE (#6551) (magic-akari)
- 834ee2a semantic: `TSConditionalType` scope enter/exit locations (#6351) (DonIsaac)

### Refactor

- 073b02a ast: Type params field before params in TS function declaration types (#6391) (overlookmotel)
- 458f8f3 ast_tools: Consistent comments on `AstBuilder` methods (#6664) (overlookmotel)

## [0.31.0] - 2024-10-08

- 01b878e parser: [**BREAKING**] Use `BindingIdentifier` for `namespace` declaration names (#6003) (DonIsaac)

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

### Features

- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

### Refactor

- acab777 regular_expression: Misc fixes (#6234) (leaysgur)

## [0.30.2] - 2024-09-27

### Features

- 60c52ba ast: Allow passing span to `void_0` method (#6065) (Dunqing)
- 28da771 transformer: Do not transform `**` with bigint literals (#6023) (Boshen)

### Bug Fixes

- b1af73d semantic: Do not create a `global` symbol for `declare global {}` (#6040) (DonIsaac)

### Refactor

- 1fc80d1 ast: Move all ts ast related impl methods to `ast_impl` (#6015) (Dunqing)

## [0.30.1] - 2024-09-24

### Documentation

- 5a0d17c ast: Document more AST nodes (#6000) (DonIsaac)
- 1abfe8f semantic: Document `SymbolTable` (#5998) (DonIsaac)

## [0.30.0] - 2024-09-23

- 033b907 ast: [**BREAKING**] Apply `#[non_exhaustive]`, must use `AstBuilder` (#5787) (Boshen)

### Features

- ae89145 ast: Revert `#[non_exhaustive]` change (#5885) (Boshen)
- e8bf30a ast: Add `Comment::real_span` (#5764) (Boshen)
- bcdbba3 codegen: Print jsdoc comments that are attached to statements and class elements (#5845) (Boshen)
- 4a62703 isolated-declarations: Handle `export` in the `namespace` correctly (#5950) (Dunqing)
- 3bf7b24 linter: Make `typescript/no-duplicate-enum-values` a `correctness` rule (#5810) (DonIsaac)
- 8e7556f parser: Calculate leading and trailing position for comments (#5785) (Boshen)
- 65c337a prettier: Improve ts compatibility (#5900) (Alexander S.)
- 6d9ccdd prettier: Support TSMappedType (#5834) (Alexander S.)
- b5ac5a6 prettier: Support TSModuleDeclaration (#5813) (Alexander S.)

### Bug Fixes

- 66e919e ast: Correct TS types for JSX (#5884) (overlookmotel)
- 0d10521 ast: Serialize `JSXMemberExpressionObject` to estree (#5883) (overlookmotel)
- a822c9d ast: Serialize `JSXElementName` to estree (#5882) (Boshen)
- 8780c54 isolated-declarations: Do not union a undefined when the param type is any or unknown (#5930) (Dunqing)

### Documentation

- acc2d16 ast: Document most TypeScript AST nodes (#5983) (DonIsaac)
- 47c2faa ast: Document TryStatement and related nodes (#5970) (DonIsaac)

### Refactor

- f4fac0f ast: Remove `.iter()` where not needed (#5904) (camchenry)
- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)
- 7caae5b codegen: Add `GetSpan` requirement to `Gen` trait (#5772) (Boshen)
- 1c1353b transformer: Use AstBuilder instead of using struct constructor (#5778) (Boshen)

## [0.29.0] - 2024-09-13

- c3dd2a0 ast: [**BREAKING**] Revert: reduce byte size of `TaggedTemplateExpression::quasi` by `Boxing` it (#5679) (#5715) (overlookmotel)

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)

### Performance


## [0.28.0] - 2024-09-11

- afc4548 ast: [**BREAKING**] Educe byte size of `TaggedTemplateExpression::quasi` by `Boxing` it (#5679) (Boshen)

- 7415e85 ast: [**BREAKING**] Reduce byte size of `TSImportType::attributes` by `Box`ing it (#5678) (Boshen)

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

### Features

- 2da5ad1 ast: Add `JSXElementName::get_identifier` method (#5556) (overlookmotel)
- 68c3cf5 minifier: Fold `void 1` -> `void 0` (#5670) (Boshen)
- c6bbf94 minifier: Constant fold unary expression (#5669) (Boshen)

### Bug Fixes

- 28b934c coverage: Apply `always_strict` to test262 and typescript per the specifcation (#5555) (Boshen)
- 0511d55 regular_expression: Report more MayContainStrings error in (nested)class (#5661) (leaysgur)

### Performance


### Refactor

- 14ee086 ast: Inline `AstKind::as_*` methods (#5547) (overlookmotel)
- 2da42ef regular_expression: Improve AST docs with refactoring may_contain_strings (#5665) (leaysgur)- 26d9235 Enable clippy::ref_as_ptr  (#5577) (夕舞八弦)

## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

- 87c5df2 ast: [**BREAKING**] Rename `Expression::without_parentheses` (#5448) (overlookmotel)

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)
- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)
- 68a1c01 ast_tools: Add dedicated `Derive` trait. (#5278) (rzvxa)
- 62f7fff semantic: Check for non-declared, non-abstract object accessors without bodies (#5461) (DonIsaac)
- 5407143 semantic: Check for non-declared, non-abstract class accessors without bodies (#5460) (DonIsaac)
- cedf7a4 xtask: Impl `as_ast_kind` method for each variant (#5491) (IWANABETHATGUY)

### Bug Fixes

- 0df1d9d ast, codegen, linter: Panics in fixers. (#5431) (rzvxa)- b96bea4 Add back lifetime (#5507) (IWANABETHATGUY)

### Documentation

- 64db1b4 ast: Clarify docs for `RegExpPattern` (#5497) (overlookmotel)

### Refactor

- a43e951 ast: Use loop instead of recursion (#5447) (overlookmotel)
- 2224cc4 ast: Renumber `JSXMemberExpressionObject` discriminants (#5464) (overlookmotel)
- a952c47 ast: Use loop not recursion (#5449) (overlookmotel)
- d9d7e7c ast: Remove `IdentifierName` from `TSThisParameter` (#5327) (overlookmotel)
- ccc8a27 ast, ast_tools: Use full method path for generated derives trait calls. (#5462) (rzvxa)
- fdb8857 linter: Use "parsed pattern" in `no_div_regex` rule. (#5417) (rzvxa)
- b47aca0 syntax: Use `generate_derive` for `CloneIn` in types outside of `oxc_ast` crate. (#5280) (rzvxa)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

- c100826 semantic: [**BREAKING**] Always create a scope for `for` statements (#5110) (overlookmotel)

- d304d6f semantic: [**BREAKING**] Always create a scope for `CatchClause` (#5109) (overlookmotel)

### Features

- 180b1a1 ast: Add `Function::name()` (#5361) (DonIsaac)
- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 49cd5db ast,parser: Add `definite` flag to `AccessorProperty` node (#5182) (DonIsaac)
- c2fa725 ast,parser: Parse `TSTypeAnnotations` on `AccessorProperty` (#5179) (DonIsaac)
- f81e8a1 linter: Add `oxc/no-async-endpoint-handlers` (#5364) (DonIsaac)

### Bug Fixes

- 8ebc23f ast: Serialize `TSParenthesizedType` with camelCase (#5199) (Kevin Deng 三咲智子)
- 8a17807 parser: Treat JSX element tags starting with `_` or `$` as `IdentifierReference`s (#5343) (overlookmotel)

### Performance

- 292f217 ast: Optimize `JSXIdentifier::is_reference` (#5344) (overlookmotel)

### Refactor

- c2d8c9e ast: Reduce allocations in `AstBuilder::move_assignment_target` (#5367) (overlookmotel)
- 946c867 ast: Box `TSThisParameter` (#5325) (overlookmotel)
- 960e1d5 ast: Rename `IdentifierReference::new_with_reference_id` (#5157) (overlookmotel)
- f63b568 ast: Remove `#[non_exhaustive]` attr from `AstBuilder` (#5130) (overlookmotel)
- d236554 parser: Move `JSXIdentifier` conversion code into parser (#5345) (overlookmotel)

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 714373d ast: `inherit_variants!` macro add `into_*` methods (#5005) (overlookmotel)

### Bug Fixes

- 7f3129e ast: Correct code comment (#5004) (overlookmotel)
- 1365feb transformer: Remove an `AstBuilder::copy` call for TS `AssignmentTarget` transform (#4984) (overlookmotel)- b7db235 Comments gen regression (#5003) (IWANABETHATGUY)

### Refactor

- cca7440 ast: Replace `AstBuilder::move_statement_vec` with `move_vec` (#4988) (overlookmotel)
- 4012260 ast: `AstBuilder::move_identifier_reference` do not allocate empty string (#4977) (overlookmotel)
- 96422b6 ast: Make AstBuilder non-exhaustive (#4925) (DonIsaac)
- 4796ece transformer: TS annotations transform use `move_expression` (#4982) (overlookmotel)

## [0.24.3] - 2024-08-18

### Features

- fd34640 traverse: Support `generate_uid_based_on_node` method in `TraverseCtx` (#4940) (Dunqing)

### Bug Fixes

- c0b26f4 ast: Do not include `scope_id` fields in JSON AST (#4858) (overlookmotel)
- 879a271 minifier: Do not join `require` calls for `cjs-module-lexer` (#4875) (Boshen)
- 248a757 transformer/typescript: Typescript syntax within `SimpleAssignmentTarget` with `MemberExpressions` is not stripped (#4920) (Dunqing)

### Documentation

- 47c9552 ast, ast_macros, ast_tools: Better documentation for `Ast` helper attributes. (#4856) (rzvxa)

### Refactor

- 90d0b2b allocator, ast, span, ast_tools: Use `allocator` as var name for `Allocator` (#4900) (overlookmotel)
- 1eb59d2 ast, isolated_declarations, transformer: Mark `AstBuilder::copy` as an unsafe function (#4907) (overlookmotel)
- 8e8fcd0 ast_tools: Rename `oxc_ast_codegen` to `oxc_ast_tools`. (#4846) (rzvxa)

## [0.24.2] - 2024-08-12

### Documentation

- 8827659 ast: More doc comments for JSX nodes (#4830) (DonIsaac)

### Refactor

- 0ea697b ast, ast_codegen: `CloneIn` implementations now initialize semantic related cells with `Default` value. (#4819) (rzvxa)
- ecfa124 ast_codegen: Add line break to generated code (#4829) (overlookmotel)
- 096ac7b linter: Clean up jsx-a11y/anchor-is-valid (#4831) (DonIsaac)

## [0.24.1] - 2024-08-10

### Bug Fixes

- fff9da3 ast, ast_codegen: Use `generate_derive` instead of visitable for generating span derives. (#4747) (rzvxa)
- f5eeebd ast_macros: Raise compile error on invalid `generate_derive` input. (#4766) (rzvxa)

### Refactor

- daa0b2e ast: `oxc_ast` crate re-export AST types from other crates (#4773) (overlookmotel)
- d4a3be8 ast_codegen: Line breaks between types in layout assertions (#4781) (overlookmotel)
- dbb5f4c ast_codegen: Remove unnecessary imports from generated files (#4774) (overlookmotel)
- 2dea0ca ast_codegen: Consistent import order (#4761) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- 51c1ca0 ast: Derive `CloneIn` for AST types, using `generate_derive`. (#4732) (rzvxa)
- e12bd1e ast: Allow conversion from TSAccessibility into &'static str (#4711) (DonIsaac)
- fd2d9da ast: Improve `AstKind::debug_name` (#4553) (DonIsaac)
- b3b7028 ast: Implement missing Clone, Hash, and Display traits for literals (#4552) (DonIsaac)
- 54047e0 ast: `GetSpanMut` trait (#4609) (overlookmotel)
- eae401c ast, ast_macros: Apply stable repr to all `#[ast]` enums (#4373) (rzvxa)
- ec0b4cb ast_codegen: Add `derive_clone_in` generator. (#4731) (rzvxa)
- 82e2f6b ast_codegen: Process AST-related `syntax` types. (#4694) (rzvxa)
- 0c52c0d ast_codegen: Add alignment and size data to the schema. (#4615) (rzvxa)
- 07607d3 ast_codegen, span: Process `Span` through ast_codegen (#4703) (overlookmotel)
- 125c5fd ast_codegen, span: Process `SourceType` through ast_codegen. (#4696) (rzvxa)
- eaddc8f linter: Add fixer for eslint/func_names (#4714) (DonIsaac)

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Documentation

- c69ada4 ast: Improve AST node documentation (#4051) (Rintaro Itokawa)

### Refactor

- 579b797 ast: Use type identifier instead of `CloneIn::Cloned` GAT. (#4738) (rzvxa)
- 475266d ast: Use correct lifetimes for name-related methods (#4712) (DonIsaac)
- 83b6ca9 ast: Add explicit enum discriminants. (#4689) (rzvxa)
- ba70001 ast: Put `assert_layouts.rs` behind `debug_assertions` (#4621) (rzvxa)
- 2218340 ast, ast_codegen: Use `generate_derive` for implementing `GetSpan` and `GetSpanMut` traits. (#4735) (rzvxa)

### Testing

- 49d5196 ast: Fix `assert_layouts.rs` offset tests on 32bit platforms. (#4620) (rzvxa)

## [0.23.1] - 2024-08-06

### Features

- fd2d9da ast: Improve `AstKind::debug_name` (#4553) (DonIsaac)
- b3b7028 ast: Implement missing Clone, Hash, and Display traits for literals (#4552) (DonIsaac)
- 54047e0 ast: `GetSpanMut` trait (#4609) (overlookmotel)
- eae401c ast, ast_macros: Apply stable repr to all `#[ast]` enums (#4373) (rzvxa)
- 0c52c0d ast_codegen: Add alignment and size data to the schema. (#4615) (rzvxa)

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Documentation

- c69ada4 ast: Improve AST node documentation (#4051) (Rintaro Itokawa)

### Refactor

- ba70001 ast: Put `assert_layouts.rs` behind `debug_assertions` (#4621) (rzvxa)

### Testing

- 49d5196 ast: Fix `assert_layouts.rs` offset tests on 32bit platforms. (#4620) (rzvxa)

## [0.23.0] - 2024-08-01

### Features

- 35654e6 codegen: Align operator precedence with esbuild (#4509) (Boshen)
- b952942 linter: Add eslint/no-unused-vars (⭐ attempt 3.2) (#4445) (DonIsaac)
- 85e8418 linter: Add react/jsx-curly-brace-presence (#3949) (Don Isaac)

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Performance

- c9c38a1 parser: Support peeking over bytes (#4304) (lucab)

### Documentation

- 0914e47 ast: Add doc comments to literal nodes (#4551) (DonIsaac)
- c6a11be ast: Auto-generate doc comments for AstBuilder methods (#4471) (DonIsaac)

## [0.22.1] - 2024-07-27

### Features

- 2477330 ast: Add `AstKind::TSExportAssignment` (#4501) (Dunqing)
- aaee07e ast: Add `AstKind::AssignmentTargetPattern`, `AstKind::ArrayAssignmentTarget` and `AstKind::ObjectAssignmentTarget` (#4456) (Dunqing)
- fd363d1 ast: Add AstKind::get_container_scope_id (#4450) (DonIsaac)

### Bug Fixes

- 368112c ast: Remove `#[visit(ignore)]` from `ExportDefaultDeclarationKind`'s `TSInterfaceDeclaration` (#4497) (Dunqing)

### Documentation

- f5f0ba8 ast: Add doc comments to more AST nodes (#4413) (Don Isaac)

### Refactor

- 9c5d2f9 ast/builder: Use `Box::new_in` over `.into_in` (#4428) (overlookmotel)

## [0.22.0] - 2024-07-23

- f68b659 ast: [**BREAKING**] Reorder fields of `ArrowFunctionExpression` (#4364) (Dunqing)

### Features

- d345b84 ast: Add `#[ast]` attribute to non-visited AST types. (#4309) (rzvxa)
- 3c0c709 linter: Add typescript-eslint/no-extraneous-class (#4357) (Jaden Rodriguez)
- 68efcd4 linter/react-perf: Handle new objects and arrays in prop assignment patterns (#4396) (DonIsaac)

### Bug Fixes

- aece1df ast: Visit `Program`s `hashbang` field first (#4368) (overlookmotel)

### Performance
- a207923 Replace some CompactStr usages with Cows (#4377) (DonIsaac)

### Refactor

- d213773 ast: Replace serde rename "lowercase" with "camelCase" (#4376) (overlookmotel)
- abfccbd ast: Reduce `#[cfg_attr]` boilerplate in AST type defs (#4375) (overlookmotel)
- 5f1c7ec ast: Rename the `visited_node` marker to `ast`. (#4289) (rzvxa)
- 59aea73 ast: Scope is created only if CatchClause has param (#4346) (Dunqing)
- 7a3e925 ast_codegen: Better visit marker parsing. (#4371) (rzvxa)

## [0.21.0] - 2024-07-18

### Features

- af4dc01 ast: Align ts ast scope with typescript (#4253) (Dunqing)
- 20cdb1f semantic: Align class scope with typescript (#4195) (Dunqing)
- 92ee774 semantic: Add `ScopeFlags::CatchClause` for use in CatchClause (#4205) (Dunqing)

### Bug Fixes

- e167ef7 codegen: Print parenthesis properly (#4245) (Boshen)
- 1108f2a semantic: Resolve references to the incorrect symbol (#4280) (Dunqing)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)
- 3e099fe ast: Move `enter_scope` after `visit_binding_identifier` (#4246) (Dunqing)
- aab7aaa ast/visit: Fire node events as the outermost one. (#4203) (rzvxa)
- ace4f1f semantic: Update the order of `visit_function` and `Visit` fields in the builder to be consistent (#4248) (Dunqing)
- 7f1addd semantic: Correct scope in CatchClause (#4192) (Dunqing)
- 1458d81 visit: Add `#[inline]` to empty functions (#4330) (overlookmotel)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Features

- 67fe75e ast, ast_codegen: Pass the `scope_id` to the `enter_scope` event. (#4168) (rzvxa)

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)

### Documentation

- bdcc298 ast: Update the note regarding the `ast_codegen` markers. (#4149) (rzvxa)

### Refactor


## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Features

- 485c871 ast: Allow conversion from `Expression` into `Statement` with `FromIn` trait. (#4124) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features

- 2f53bdf semantic: Check for abstract ClassElements in non-abstract classes (#4127) (DonIsaac)
- c4ee9f8 semantic: Check for abstract initializations and implementations (#4125) (Don Isaac)

## [0.17.2] - 2024-07-08

### Features

- e386b62 semantic: Check for invalid type import assignments (#4097) (DonIsaac)

## [0.17.1] - 2024-07-06

### Bug Fixes

- aa585d3 ast_codegen, ast: Visit `ExpressionArrayElement` as `Expression`. (#4061) (rzvxa)

### Refactor

- 8fa98e0 ast: Inline trivial functions and shorten code (#4066) (overlookmotel)

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

