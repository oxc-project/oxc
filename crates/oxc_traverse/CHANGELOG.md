# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).



## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)
- 7332ae4 ast: [**BREAKING**] Box `rest` fields of `ArrayAssignmentTarget` and `ObjectAssignmentTarget` (#12698) (Copilot)

### 📚 Documentation

- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)




## [0.78.0] - 2025-07-24

### 🚀 Features

- dee25f4 ast: Add `pife` field to `Function` (#12469) (sapphi-red)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)



## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)






## [0.74.0] - 2025-06-23

### 💥 BREAKING CHANGES

- 8ef1be2 traverse: [**BREAKING**] Introduce `TraverseCtx<'a, State>` (#11770) (Boshen)




## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.0] - 2025-05-24

### Features

- 4feeeee span: Add `Atom::from_strs_array_in` method (#11261) (overlookmotel)
- c2c0268 syntax: Introduce `CommentNodeId` (#11214) (overlookmotel)

### Refactor

- 202ffd2 transformer: Use `StringBuilder` instead of `String` (#11260) (overlookmotel)

## [0.71.0] - 2025-05-20

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Refactor


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

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

### Performance

- 79e462d transformer: Faster UID generation (#10759) (overlookmotel)

### Refactor

- ae57125 traverse: Remove `get_unique_name_impl` (#10755) (overlookmotel)

## [0.67.0] - 2025-04-27

### Performance

- bdcbeb4 traverse: Use `ArenaString` instead `CompactString` to store UID name (#10562) (Dunqing)

### Refactor

- b31ab87 traverse: Take `&str` instead of `CompactStr` in `TraverseScoping::rename_symbol` (#10610) (Dunqing)
- f35efd3 traverse, semantic: Move `rename_symbol` from `TraverseScoping` to `Scoping` (#10611) (Dunqing)

## [0.66.0] - 2025-04-23

### Bug Fixes

- 43ad4e9 ast: Box `this_param` in `TSCallSignatureDeclaration` (#10558) (Yuji Sugiura)

### Styling

- e10dfc8 traverse: Remove excess whitespace (#10544) (overlookmotel)

## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

### Refactor


## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)

### Bug Fixes

- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)
- 5850a0d parse: `type x = typeof import('')` -> ` TSTypeQuery(TSImportType)` (#10317) (Boshen)
- 1e683f9 traverse: `ChildScopeCollector` visit all scopes (#10292) (overlookmotel)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- f48f895 transfomer/using: Remove use of child ids (#9961) (camc314)

### Refactor

- d43e864 ast_tools: Generate `ChildScopesCollector` in `oxc_ast_tools` (#10248) (overlookmotel)
- e53708e traverse: Remove repeated code (#10245) (overlookmotel)

## [0.62.0] - 2025-04-01

### Bug Fixes

- 2c53a72 transformer/using: Correctly reparent scopes in for of stmt (#9960) (camc314)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features


## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.59.0] - 2025-03-18

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Bug Fixes


### Refactor

- 43e8efd transformer/using: Remove use of child scope ids in enter_program (#9748) (camc314)
- 1d60e85 traverse: Use `ArenaBox` alias (#9759) (overlookmotel)
- bda4b9a traverse: Add `TraverseCtx::insert_scope_below_statements` (#9757) (overlookmotel)
- d7dda5c traverse: Add doc comment for `TraverseScoping::insert_scope_below_statements` and rename var (#9756) (overlookmotel)

## [0.58.0] - 2025-03-13

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

- 3c6f140 semantic: [**BREAKING**] Make `Scoping` methods consistent (#9628) (Boshen)

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 7331656 semantic: [**BREAKING**] Rename `SymbolTable` and `ScopeTree` methods (#9613) (Boshen)

- 23738bf semantic: [**BREAKING**] Introduce `Scoping` (#9611) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

### Refactor


## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

- 48a0394 ast: [**BREAKING**] Add `scope_id` to `TSFunctionType` (#9559) (camc314)

### Features


## [0.54.0] - 2025-03-04

- a8d1d48 parser,codegen: [**BREAKING**] Parse and print`#__NO_SIDE_EFFECTS__` (#9496) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

- abb15e0 ast: [**BREAKING**] Add `pure` field to `Function`, `CallExpression`, and `NewExpression` (#9207) (overlookmotel)

### Features

- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)

## [0.53.0] - 2025-02-26

### Bug Fixes

- 6a8f53f ast/estree: Visit `JSXOpeningFragment` and `JSXClosingFragment` (#9342) (overlookmotel)

### Refactor

- 7427900 ast: Re-order `ExportDefaultDeclaration` fields (#9348) (overlookmotel)
- f39be5f traverse: Reduce scope of `unsafe` blocks (#9323) (overlookmotel)

## [0.52.0] - 2025-02-21

### Refactor

- 97cc1c8 ast: Remove `TSLiteral::NullLiteral` (replaced by `TSNullKeyword`) (#9147) (Boshen)
- ef856f5 oxc: Apply `clippy::needless_pass_by_ref_mut` (#9253) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)
- 9c3549d traverse: Apply `clippy::redundant_pub_crate` (#9258) (Boshen)

## [0.51.0] - 2025-02-15

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

- 9091387 ast: [**BREAKING**] Remove `TSType::TSQualifiedName` (#9051) (Dunqing)

### Features


## [0.50.0] - 2025-02-12

- d9189f1 ast: [**BREAKING**] Remove `PrivateInExpression::operator` field (#9041) (overlookmotel)

### Refactor


## [0.49.0] - 2025-02-10

### Refactor

- 04786ac ast: Use `=` syntax for `#[scope]` attrs (#8878) (overlookmotel)

## [0.48.0] - 2025-01-24

### Refactor

- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.47.0] - 2025-01-18

### Refactor

- 04bc259 traverse: Remove unnecessary `#[allow]` (#8518) (overlookmotel)
- a368726 traverse: Harden soundness of `Traverse` and document safety invariants better (#8507) (overlookmotel)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)

## [0.44.0] - 2024-12-25

- ad2a620 ast: [**BREAKING**] Add missing `AssignmentTargetProperty::computed` (#8097) (Boshen)

### Features

- e632a7b transformer: Remove typescript symbols after transform (#8069) (Boshen)

### Bug Fixes


## [0.43.0] - 2024-12-21

- de4c772 traverse: [**BREAKING**] Rename `Ancestor::is_via_*` methods to `is_parent_of_*` (#8031) (overlookmotel)

- ed75e42 semantic: [**BREAKING**] Make SymbolTable fields `pub(crate)` instead of `pub` (#7999) (Boshen)

### Features

- 6b6444b traverse: Record current block scope (#8007) (overlookmotel)

### Performance

- 2736657 semantic: Allocate `UnresolvedReferences` in allocator (#8046) (Boshen)
- 2e8872c semantic: Allocate child scope in allocator (#8045) (Boshen)
- 414e828 semantic: Allocate symbol data in Allocator (#8012) (Boshen)
- 7aebed0 semantic: Allocate `Bindings` in allocator (#8021) (Boshen)

### Refactor

- f1adf9f semantic: `ScopeTree::rename_binding` remove old binding first (#8020) (overlookmotel)

## [0.42.0] - 2024-12-18

### Features

- 53e2bc0 traverse: Add `TraverseScoping::rename_symbol` method (#7871) (overlookmotel)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.41.0] - 2024-12-13

- fb325dc ast: [**BREAKING**] `span` field must be the first element (#7821) (Boshen)

### Refactor


## [0.40.0] - 2024-12-10

- 5d6fa25 traverse: [**BREAKING**] Remove `TraverseCtx::is_static` (#7760) (overlookmotel)

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- ff73c7f traverse: Add `TraverseCtx::generate_uid_in_current_hoist_scope_based_on_node` (#7642) (Dunqing)

### Refactor


## [0.39.0] - 2024-12-04

- f2f31a8 traverse: [**BREAKING**] Remove unsound APIs (#7514) (overlookmotel)

### Features

- 9c9deae traverse: Add `generate_uid_in_current_hoist_scope` method (#7423) (Dunqing)

### Bug Fixes


### Documentation

- 4d157c5 traverse: Document soundness hole (#7515) (overlookmotel)

## [0.38.0] - 2024-11-26

### Features

- 971c91a traverse: Add methods to `BoundIdentifier` + `MaybeBoundIdentifier` to create `SimpleAssignmentTarget`s (#7418) (overlookmotel)

### Documentation

- be5f843 traverse: Fix docs for `BoundIdentifier` + `MaybeBoundIdentifier` (#7417) (overlookmotel)

### Styling

- 10ea830 traverse: Fix indentation in codegen (#7475) (overlookmotel)

## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 1cbc624 traverse: [**BREAKING**] Rename `TraverseCtx` methods for creating `IdentifierReference`s (#7300) (overlookmotel)

- e84ea2c traverse: [**BREAKING**] Remove `TraverseCtx::clone_identifier_reference` (#7266) (overlookmotel)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- 6cfb5df transformer: Support generate proper binding name from ChainExpression (#7326) (Dunqing)
- 234c7b9 traverse: Implement `GatherNodeParts` for member expression types (#7363) (overlookmotel)
- faf8dde traverse: Add methods for creating `Expression::Identifier`s (#7301) (overlookmotel)
- 8c754b1 traverse: Introduce `MaybeBoundIdentifier` (#7265) (overlookmotel)

### Bug Fixes


### Documentation

- 834c94d traverse: Tidy doc comments for `TraverseCtx::is_static` (#7267) (overlookmotel)

### Refactor

- de472ca ast: Move `StringLiteral` definition higher up (#7270) (overlookmotel)
- 7a48728 traverse: Reorder imports (#7264) (overlookmotel)

## [0.36.0] - 2024-11-09

- b11ed2c ast: [**BREAKING**] Remove useless `ObjectProperty::init` field (#7220) (Boshen)

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- 843bce4 ast: [**BREAKING**] `IdentifierReference::reference_id` return `ReferenceId` (#7126) (overlookmotel)

### Features


### Refactor

- cacfb9b traverse: Use `symbol_id` etc methods (#7129) (overlookmotel)

## [0.35.0] - 2024-11-04

### Bug Fixes

- caaf00e parser: Fix incorrect parsed `TSIndexSignature` (#7016) (Boshen)

### Refactor

- d9edef6 transformer: Combine ObjectRestSpread into a single file (#7002) (Boshen)
- 938ee87 traverse: Do not use `AstBuilder::*_from_*` methods (#7069) (overlookmotel)

## [0.34.0] - 2024-10-26

### Features

- 419343b traverse: Implement `GetAddress` for `Ancestor` (#6877) (overlookmotel)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)
- a366fae traverse: Rename `TraverseScoping::generate_binding_in_current_scope` (#6832) (overlookmotel)
- 3b99fe6 traverse: Move `generate_binding` to `TraverseScoping` (#6831) (overlookmotel)
- 60f487a traverse: `TraverseCtx::generate_binding` take an `Atom` (#6830) (overlookmotel)

## [0.33.0] - 2024-10-24

- aeaa27a ast, parser, transformer, traverse: [**BREAKING**] Remove `BindingIdentifier::new` methods (#6786) (overlookmotel)

- ecc9151 ast, parser, transformer, traverse: [**BREAKING**] Remove `IdentifierReference::new` methods (#6785) (overlookmotel)

- 1248557 ast: [**BREAKING**] Remove `AstKind::FinallyClause` (#6744) (Boshen)

### Features

- 10484cd transformer: Class static block transform (#6733) (overlookmotel)
- c96e739 traverse: Add `generate_binding` and `generate_binding_current_scope` APIs in context (#6805) (Dunqing)
- ce1d8cf traverse: Add `BoundIdentifier::from_binding_ident` method (#6814) (overlookmotel)

### Documentation

- 55c07f2 traverse: Correct doc comment for `BoundIdentifier` (#6810) (overlookmotel)

### Refactor

- 47bc368 traverse: `BoundIdentifier` methods only take `&TraverseCtx` (#6811) (overlookmotel)
- 1370c2d traverse: Change `generate_uid_in_based_on_node` to accept a generic type parameter as node type (#6708) (Dunqing)

## [0.32.0] - 2024-10-19

- 2808973 ast: [**BREAKING**] Add `Program::comments` (#6445) (Boshen)

### Features

- d9718ad ast_tools: Support `#[scope(exit_before)]` (#6350) (DonIsaac)

### Bug Fixes

- 834ee2a semantic: `TSConditionalType` scope enter/exit locations (#6351) (DonIsaac)

### Performance

- ac77c87 traverse: Optimize `TraverseScoping::generate_uid_name` (#6663) (overlookmotel)

### Refactor

- 073b02a ast: Type params field before params in TS function declaration types (#6391) (overlookmotel)

## [0.31.0] - 2024-10-08

- 01b878e parser: [**BREAKING**] Use `BindingIdentifier` for `namespace` declaration names (#6003) (DonIsaac)

- 409dffc traverse: [**BREAKING**] `generate_uid` return a `BoundIdentifier` (#6294) (overlookmotel)

### Features

- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)

### Documentation

- c7636d7 traverse: Remove erroneous doc comment (#6328) (overlookmotel)

### Refactor

- 7b62966 transformer: Move `BoundIdentifier` into `oxc_traverse` crate (#6293) (overlookmotel)
- 0dd9a2e traverse: Add helper methods to `BoundIdentifier` (#6341) (overlookmotel)
- c0e2fef traverse: Function to get var name from node (#6317) (overlookmotel)
- adc5381 traverse: `TraverseAncestry` use `NonEmptyStack` (#6217) (overlookmotel)

## [0.30.0] - 2024-09-23

### Features

- 635e918 traverse: `generate_uid_name` method (#5839) (overlookmotel)

### Refactor

- 1c1353b transformer: Use AstBuilder instead of using struct constructor (#5778) (Boshen)

## [0.29.0] - 2024-09-13

- c3dd2a0 ast: [**BREAKING**] Revert: reduce byte size of `TaggedTemplateExpression::quasi` by `Boxing` it (#5679) (#5715) (overlookmotel)

### Performance


### Refactor

- cc0408b semantic: S/AstNodeId/NodeId (#5740) (Boshen)

## [0.28.0] - 2024-09-11

- afc4548 ast: [**BREAKING**] Educe byte size of `TaggedTemplateExpression::quasi` by `Boxing` it (#5679) (Boshen)

- 7415e85 ast: [**BREAKING**] Reduce byte size of `TSImportType::attributes` by `Box`ing it (#5678) (Boshen)

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

### Features

- 17226dd traverse: Add methods for deleting references (#5559) (overlookmotel)

### Performance

- e8013d2 traverse: Faster string operations generating UIDs (#5626) (overlookmotel)
- 4996874 traverse: `generate_uid` cache available binding names (#5611) (overlookmotel)

### Documentation

- 1c051ae traverse: Correct code comment 2 (#5607) (overlookmotel)
- 2e24a15 traverse: Correct code comment (#5604) (overlookmotel)

### Refactor

- 2de6ea0 index, traverse: Remove unnecessary type annotations (#5650) (overlookmotel)
- 19cdcc5 traverse: Revert changes to `walk.rs` (#5652) (overlookmotel)- 26d9235 Enable clippy::ref_as_ptr  (#5577) (夕舞八弦)

### Styling

- e52d006 traverse: Fix formatting of traverse codegen (#5651) (overlookmotel)
- 97e99bd traverse: Remove excess line break (#5603) (overlookmotel)

### Testing

- 2e367c9 traverse: Enable tests for `oxc_traverse` crate (#5625) (overlookmotel)

## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

### Features


### Bug Fixes

- 0eb32a6 traverse: Invalid variable name generated by `generate_uid_based_on_node` (#5407) (Dunqing)

### Refactor

- d9d7e7c ast: Remove `IdentifierName` from `TSThisParameter` (#5327) (overlookmotel)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 23e8456 traverse: [**BREAKING**] `TraverseCtx::ancestor` with level 0 = equivalent to `parent` (#5294) (overlookmotel)

- 582ce9e traverse: [**BREAKING**] `TraverseCtx::ancestor` return `Ancestor::None` if out of bounds (#5286) (overlookmotel)

- 234a24c ast: [**BREAKING**] Merge `UsingDeclaration` into `VariableDeclaration` (#5270) (Kevin Deng 三咲智子)

- c100826 semantic: [**BREAKING**] Always create a scope for `for` statements (#5110) (overlookmotel)

- d304d6f semantic: [**BREAKING**] Always create a scope for `CatchClause` (#5109) (overlookmotel)

### Features

- 5505749 ast: Add `accessibility` field to `AccessorProperty` (#5290) (Dunqing)
- 49cd5db ast,parser: Add `definite` flag to `AccessorProperty` node (#5182) (DonIsaac)
- c2fa725 ast,parser: Parse `TSTypeAnnotations` on `AccessorProperty` (#5179) (DonIsaac)

### Bug Fixes

- d594818 traverse: `insert_scope_below` update child scopes records (#5409) (overlookmotel)
- 25d6e20 traverse: Add missing visitors to `ChildScopeCollector` (#5118) (overlookmotel)

### Refactor

- 946c867 ast: Box `TSThisParameter` (#5325) (overlookmotel)
- 05d25e2 semantic: Combine add scope methods (#5262) (overlookmotel)
- a17cf33 semantic: Remove `ScopeTree::child_ids` (#5232) (Boshen)
- b43a394 traverse: Correct code comments (#5293) (overlookmotel)
- d71f0ed traverse: Inline all passthrough methods (#5279) (overlookmotel)
- 188ce07 traverse: Improve safety via type system (#5277) (overlookmotel)
- 0f4a8b3 traverse: Add debug asserts for safety invariants (#5272) (overlookmotel)
- 341e42a traverse: Make `Ancestor` an owned type (#5269) (overlookmotel)
- eba5033 traverse: Codegen `ChildScopeCollector` (#5119) (overlookmotel)
- f771d7c traverse: Remove unnecessary imports (#5116) (overlookmotel)
- c6590ae traverse: Move generated files into separate folder (#5115) (overlookmotel)
- fc2e9ad traverse: Remove support for `#[scope(if(...))]` attr (#5114) (overlookmotel)
- 1ba11a3 traverse: Refactor `ChildScopeCollector` (#5112) (overlookmotel)
- 40e2f6e traverse: Remove unnecessary branch in `ChildScopeCollector` (#5111) (overlookmotel)

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- f2b8d82 semantic: [**BREAKING**] `ScopeTree::get_child_ids` + `get_child_ids_mut` return value not `Option` (#5058) (overlookmotel)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 6b885fe traverse: Expose `generate_uid_based_on_node` and `generate_uid_in_current_scope_based_on_node` from `TraverseCtx` (#4965) (Dunqing)

### Refactor

- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)
- 59d15c7 semantic: `root_unresolved_references` contain only `ReferenceId` (#4959) (overlookmotel)

## [0.24.3] - 2024-08-18

### Features

- fd34640 traverse: Support `generate_uid_based_on_node` method in `TraverseCtx` (#4940) (Dunqing)
- 72a37fc traverse: Support `clone_identifier_reference` method in `TraverseCtx` (#4880) (Dunqing)

## [0.24.0] - 2024-08-08

- 75f2207 traverse: [**BREAKING**] Replace `find_scope` with `ancestor_scopes` returning iterator (#4693) (overlookmotel)

- 506709f traverse: [**BREAKING**] Replace `find_ancestor` with `ancestors` returning iterator (#4692) (overlookmotel)

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Refactor

- e0832f8 minifier: Use `oxc_traverse` for AST passes (#4725) (Boshen)
- 54f9897 traverse: Simpler code for entering/exiting unconditional scopes (#4685) (overlookmotel)
- 83546d3 traverse: Enter node before entering scope (#4684) (overlookmotel)

## [0.23.1] - 2024-08-06

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

## [0.23.0] - 2024-08-01

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Refactor

- e6a8af6 traverse: Speed up tests (#4538) (overlookmotel)

## [0.22.1] - 2024-07-27

### Bug Fixes

- c04b9aa transformer: Add to `SymbolTable::declarations` for all symbols (#4460) (overlookmotel)

### Performance

- 348c1ad semantic: Remove `span` field from `Reference` (#4464) (overlookmotel)

### Refactor

- f17254a semantic: Populate `declarations` field in `SymbolTable::create_symbol` (#4461) (overlookmotel)

## [0.22.0] - 2024-07-23

- 85a7cea semantic: [**BREAKING**] Remove name from `reference` (#4329) (Dunqing)

- f68b659 ast: [**BREAKING**] Reorder fields of `ArrowFunctionExpression` (#4364) (Dunqing)

### Bug Fixes

- aece1df ast: Visit `Program`s `hashbang` field first (#4368) (overlookmotel)

### Performance

- e70c67b semantic: Remove a branch from `add_scope` (#4384) (overlookmotel)
- 7eb2864 traverse: Speed up finding UID binding name (#4356) (overlookmotel)

### Refactor

- 5f1c7ec ast: Rename the `visited_node` marker to `ast`. (#4289) (rzvxa)

## [0.21.0] - 2024-07-18

### Features

- af4dc01 ast: Align ts ast scope with typescript (#4253) (Dunqing)
- 20cdb1f semantic: Align class scope with typescript (#4195) (Dunqing)

### Bug Fixes

- 1108f2a semantic: Resolve references to the incorrect symbol (#4280) (Dunqing)

### Refactor

- 2c7bb9f ast: Pass final `ScopeFlags` into `visit_function` (#4283) (overlookmotel)
- 3e099fe ast: Move `enter_scope` after `visit_binding_identifier` (#4246) (Dunqing)
- c418bf5 semantic: Directly record `current_node_id` when adding a scope (#4265) (Dunqing)
- ace4f1f semantic: Update the order of `visit_function` and `Visit` fields in the builder to be consistent (#4248) (Dunqing)
- fc0b17d syntax: Turn the `AstNodeId::dummy` into a constant field. (#4308) (rzvxa)

## [0.20.0] - 2024-07-11

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)

## [0.17.0] - 2024-07-05

- 4a0eaa0 ast: [**BREAKING**] Rename `visit_enum` to `visit_ts_enum_declaration`. (#3998) (rzvxa)

- c98d8aa ast: [**BREAKING**] Rename `visit_arrow_expression` to `visit_arrow_function_expression`. (#3995) (rzvxa)

### Refactor


## [0.16.3] - 2024-07-02

### Refactor

- 0fe22a8 ast: Reorder fields to reflect their visit order. (#3994) (rzvxa)

## [0.16.2] - 2024-06-30

### Features

- dc6d45e ast,codegen: Add `TSParenthesizedType` and print type parentheses correctly (#3979) (Boshen)

### Performance

- 1eac3d2 semantic: Use `Atom<'a>` for `Reference`s (#3972) (Don Isaac)

### Refactor

- 5845057 transformer: Pass in symbols and scopes (#3978) (Boshen)

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
- 2a16ce0 traverse: Disable syntax check and disable build module record (#3794) (Boshen)

### Bug Fixes

- 08fcfb3 transformer: Fix spans and scopes in TS enum transform (#3911) (overlookmotel)
- 17ad8f7 transformer: Create new scopes for new blocks in TS transform (#3908) (overlookmotel)

### Refactor

- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- 4cf3c76 parser: Improve parsing of TypeScript types (#3903) (Boshen)
- 1061baa traverse: Separate `#[scope]` attr (#3901) (overlookmotel)
- fcd21a6 traverse: Indicate scope entry point with `scope(enter_before)` attr (#3882) (overlookmotel)
- 24979c9 traverse: Use camel case props internally (#3880) (overlookmotel)
- 2045c92 traverse: Improve parsing attrs in traverse codegen (#3879) (overlookmotel)

## [0.15.0] - 2024-06-18

- 0578ece ast: [**BREAKING**] Remove `ExportDefaultDeclarationKind::TSEnumDeclaration` (#3666) (Dunqing)

### Bug Fixes

- 90743e2 traverse: Change visit order for `Function` (#3685) (overlookmotel)

### Refactor


## [0.14.0] - 2024-06-12

### Refactor

- 60cbdec traverse: `generate_uid_in_root_scope` method (#3611) (overlookmotel)

## [0.13.5] - 2024-06-08

### Bug Fixes

- 48bb97e traverse: Do not publish the build script (Boshen)

## [0.13.4] - 2024-06-07

### Bug Fixes

- c00598b transformer: JSX set `reference_id` on refs to imports (#3524) (overlookmotel)

### Refactor

- 6978269 transformer/typescript: Replace reference collector with symbols references (#3533) (Dunqing)

## [0.13.3] - 2024-06-04

### Refactor

- 7bbd3da traverse: `generate_uid` return `SymbolId` (#3520) (overlookmotel)

## [0.13.2] - 2024-06-03

### Features

- bcdc658 transformer: Add `TraverseCtx::generate_uid` (#3394) (overlookmotel)

### Bug Fixes

- 3967a15 traverse: Exit scope early if enter it late (#3493) (overlookmotel)

### Refactor

- 55bbde2 ast: Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488) (overlookmotel)

## [0.13.1] - 2024-05-22

### Features

- 0c09047 traverse: Mutable access to scopes tree + symbol table (#3314) (overlookmotel)
- 421107a traverse: Pass `&mut TraverseCtx` to visitors (#3312) (overlookmotel)

### Refactor

- 723a46f ast: Store `ScopeId` in AST nodes (#3302) (overlookmotel)
- 2b5b3fd traverse: Split context code into multiple files (#3367) (overlookmotel)
- f8b5e1e traverse: Move `parent` method etc into `TraverseAncestry` (#3308) (overlookmotel)
- 05c71d2 traverse: `Traverse` produce scopes tree using `Semantic` (#3304) (overlookmotel)

## [0.13.0] - 2024-05-14

### Features

- eefb66f ast: Add type to AccessorProperty to support TSAbractAccessorProperty (#3256) (Dunqing)
- be87ca8 transform: `oxc_traverse` crate (#3169) (overlookmotel)
- 46c02ae traverse: Add scope flags to `TraverseCtx` (#3229) (overlookmotel)

### Bug Fixes

- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)
- 6fd7a3c traverse: Create scopes for functions (#3273) (overlookmotel)
- a23ba71 traverse: Allow `TraverseCtx::find_ancestor` closure to return AST node (#3236) (overlookmotel)
- 4e20b04 traverse: Create scope for function nested in class method (#3234) (overlookmotel)

### Documentation

- a4f881f transform: Improve docs for `TraverseCtx::ancestors_depth` (#3194) (overlookmotel)

### Refactor

- 4208733 ast: Order AST type fields in visitation order (#3228) (overlookmotel)
- 762677e transform: `retag_stack` use `AncestorType` (#3173) (overlookmotel)
- ec41dba traverse: Simplify build script (#3231) (overlookmotel)
- 132db7d traverse: Do not expose `TraverseCtx::new` (#3226) (overlookmotel)

