# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.2] - 2025-02-02

### Features

- 3bc05fa transformer: Implement jsx spread child (#8763) (Boshen)

### Documentation

- 57b7ca8 ast: Add documentation for all remaining JS AST methods (#8820) (Cam McHenry)

## [0.48.1] - 2025-01-26

### Bug Fixes

- e7ab96c transformer/jsx: Incorrect `isStaticChildren` argument for `Fragment` with multiple children (#8713) (Dunqing)
- 3e509e1 transformer/typescript: Enum merging when same name declared in outer scope (#8691) (branchseer)

## [0.48.0] - 2025-01-24

### Refactor

- a3dc4c3 crates: Clean up snapshot files (#8680) (Boshen)
- e66da9f isolated_declarations, linter, minifier, prettier, semantic, transformer: Remove unnecessary `ref` / `ref mut` syntax (#8643) (overlookmotel)
- b8d9a51 span: Deal only in owned `Atom`s (#8641) (overlookmotel)
- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)
- a730f99 transformer: Move `create_prototype_member` to utils module (#8657) (Dunqing)
- 61d96fd transformer/class-properties: Correct comments (#8636) (overlookmotel)

## [0.47.1] - 2025-01-19

### Bug Fixes

- 7421a52 transformer/typescript: Correctly resolve references to non-constant enum members (#8543) (branchseer)

## [0.47.0] - 2025-01-18

- fae4cd2 allocator: [**BREAKING**] Remove `Vec::into_string` (#8571) (overlookmotel)

### Features

- f413bb5 transformer/optional-chaining: Change parent scope for expression when it wrapped with an arrow function (#8511) (Dunqing)

### Bug Fixes

- b552f5c transformer: `wrap_in_arrow_function_iife` take span of input `Expression` (#8547) (overlookmotel)
- 9963533 transformer/arrow-functions: Visit arguments to `super()` call (#8494) (overlookmotel)
- 06ccb51 transformer/async-to-generator: Move parameters to the inner generator function when they could throw errors (#8500) (Dunqing)
- 356f0c1 transformer/class-properties: Handle nested `super()` calls (#8506) (overlookmotel)
- a048337 transformer/class-static-blocks: Static block converted to IIFE use span of original block (#8549) (overlookmotel)

### Performance

- 53ef263 transformer/arrow-functions: Bail out of visiting early when inserting `_this = this` after `super()` (#8482) (overlookmotel)

### Refactor

- 712633f transformer: `wrap_statements_in_arrow_function_iife` utility function (#8548) (overlookmotel)
- 5206c6a transformer: Rename `wrap_in_arrow_function_iife` (#8546) (overlookmotel)
- 61077ca transformer: `wrap_arrow_function_iife` receive an owned `Expression` (#8545) (overlookmotel)
- 6820d24 transformer: Move `wrap_arrow_function_iife` to root utils module (#8529) (Dunqing)
- 52bd0b1 transformer: Move common utils functions to the root module (#8513) (Dunqing)
- c30654a transformer/arrow-function: Wrapping arrow function iife by using `wrap_arrow_function_iife` (#8530) (Dunqing)
- 2bc5175 transformer/arrow-functions: Rename method (#8481) (overlookmotel)
- 72f425f transformer/class-properties: Fix lint warning in release mode (#8539) (overlookmotel)
- 7e61b23 transformer/typescript: Shorten code (#8504) (overlookmotel)

## [0.46.0] - 2025-01-14

### Bug Fixes

- c444de8 transformer/arrow-functions: Transform `this` and `super` incorrectly in async arrow function (#8435) (Dunqing)
- 270245f transformer/typescript: Correct the semantic for TSImportEqualsDeclaration transformation (#8463) (Dunqing)
- 2a400d6 transformer/typescript: Retain TSImportEqualsDeclaration when it is exported (Dunqing)
- ab694b0 transformer/typescript: Retain `TSImportEqualsDeclaration` in `namespace` when its binding has been referenced or `onlyRemoveTypeImports` is true (#8458) (Dunqing)

### Refactor

- c83ce5c transformer/typescript: Improve transforming namespace (#8459) (Dunqing)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 41ddf60 minfier: Add `CompressOptions::target` (#8179) (Boshen)
- 0592a8b transformer/class-properties: Transform private in expression (#8202) (Dunqing)
- ad77ad5 transformer/class-properties: Transform static/instance accessor methods (#8132) (Dunqing)
- e405f79 transformer/class-properties: Transform static private method invoking (#8117) (Dunqing)
- 3303e99 transformer/class-properties: Insert statements after statement of class expression (#8116) (Dunqing)
- 0cc71cf transformer/class-properties: Transform super expressions and identifiers that refers to class binding in private method (#8106) (Dunqing)
- 58ed832 transformer/class-properties: Transform private field expression which invokes private method (#8102) (Dunqing)
- f14567a transformer/class-properties: Transform callee which invokes private method (#8100) (Dunqing)
- 13349ef transformer/class-properties: Transform private methods (#8099) (Dunqing)

### Bug Fixes

- 3eaff2a transformer: Ensure last expression statement in arrow function expression is wrapped in return (#8192) (Dunqing)
- 3feac27 transformer/arrow-functions: Outer `super()` in nested class (#8382) (Dunqing)
- 335065d transformer/arrow-functions: Do not transform super that inside nested non-async method (#8335) (Dunqing)
- e4d66e4 transformer/arrow-functions: Store `super_methods` on a `Stack` to fix nested async methods (#8331) (Dunqing)
- 775a289 transformer/arrow-functions: `_this = this` should be inserted after super call expression (#8024) (Dunqing)
- ac72adb transformer/private-methods: Fix panic if instance private accessor in class (#8362) (overlookmotel)
- f1f129b transformer/private-methods: Create brand binding `var` in hoist scope (#8361) (overlookmotel)
- ab61425 transformer/private-methods: No temp var for class when unused private methods (#8360) (overlookmotel)
- 9a03bd2 transformer/typescript: Remove type-only `import =` when `only_remove_type_imports` is true (#8275) (Dunqing)
- 0df1866 transformer/typescript: Create `Reference` for `require` (#8355) (overlookmotel)
- 78d7c97 transformer/typescript: Create `Reference` for `Infinity` (#8354) (overlookmotel)
- 2e7207f transformer/typescript: Should strip import specifiers type with `only_remove_type_imports` (#8141) (underfin)

### Performance

- 07edf74 transformer/arrow-function: Stop traversal at function as super() can't appear in a nested function (#8383) (Dunqing)
- 62e3f7e transformer/arrow-functions: Reduce size of inlined visitor (#8322) (overlookmotel)
- aebe0ea transformer/arrow-functions: Use `NonEmptyStack` instead of `Stack` (#8318) (overlookmotel)

### Documentation

- 05cba5b transformer/private-methods: Amend comments (#8398) (overlookmotel)

### Refactor

- 109b8fc transformer: Elide lifetimes where possible (#8285) (overlookmotel)
- fb389f7 transformer/arrow-function: Create a new ident instead of clone (#8338) (Dunqing)
- dddbd29 transformer/arrow-functions: Reorder assertions (#8386) (overlookmotel)
- ce6c445 transformer/arrow-functions: Add TODO comments (#8328) (overlookmotel)
- 73d0025 transformer/arrow-functions: Reduce repeated code (#8323) (overlookmotel)
- 3dd08e9 transformer/arrow-functions: Do not inline non-trivial visitor method (#8321) (overlookmotel)
- ea9cefb transformer/arrow-functions: Reorder visitor methods (#8320) (overlookmotel)
- 37199a4 transformer/arrow-functions: Rename lifetime (#8319) (overlookmotel)
- 57e9dcf transformer/arrow-functions: Shorten `AstBuilder` call (#8317) (overlookmotel)
- a5e3528 transformer/async-to-generator: Pass `TraverseCtx` to function not `AstBuilder` (#8279) (overlookmotel)
- e7c89ba transformer/class-properties: TODO comments (#8392) (overlookmotel)
- 6790d1d transformer/class-properties: Simplify determining if class is declaration (#8357) (overlookmotel)
- c786a13 transformer/class-properties: Share `replace_class_name_with_temp_var` in class_properties (#8105) (Dunqing)
- f54f48e transformer/class-properties: Remove all `*_if_super` methods in `static_block_and_prop_init` (#8104) (Dunqing)
- d82fb52 transformer/class-properties: Move `supers` to `super_converter` (#8103) (Dunqing)
- 3dad85e transformer/private-methods: Remove unnecessary clone (#8400) (overlookmotel)
- aa5e65f transformer/private-methods: Simplify finding parent statement of class expression (#8364) (overlookmotel)
- c786fd1 transformer/private-methods: TODO comments (#8363) (overlookmotel)

### Styling

- 45e2402 transformer/private-methods: Move comments (#8399) (overlookmotel)
- 0a1ffc0 transformer/private-methods: Rename var (#8397) (overlookmotel)

## [0.44.0] - 2024-12-25

### Features

- e632a7b transformer: Remove typescript symbols after transform (#8069) (Boshen)

### Bug Fixes

- 3057686 transformer/class-properties: Unwrap parenthesised expressions (#8049) (overlookmotel)
- e67cd05 transformer/class-properties: Correctly resolve private fields pointing to private accessors (#8047) (overlookmotel)
- 6b08c6e transformer/class-properties: Correctly resolve private fields pointing to private methods (#8042) (overlookmotel)
- 274f117 transformer/nullish-coalescing: Use correct scope id for binding (#8053) (camc314)

### Refactor

- cbd5169 transformer/class-properties: Do not recreate private field if not transforming it (#8044) (overlookmotel)
- 98e8a72 transformer/class-properties: Do not take mut ref when immut ref will do (#8040) (overlookmotel)

## [0.43.0] - 2024-12-21

- de4c772 traverse: [**BREAKING**] Rename `Ancestor::is_via_*` methods to `is_parent_of_*` (#8031) (overlookmotel)

- ed75e42 semantic: [**BREAKING**] Make SymbolTable fields `pub(crate)` instead of `pub` (#7999) (Boshen)

### Features

- 897a1a8 transformer/class-properties: Exit faster from super replacement visitor (#8028) (overlookmotel)
- 3ea4109 transformer/class-properties: Transform super update expressions within static prop initializer (#7997) (Dunqing)
- cc57db3 transformer/class-properties: Transform super assignment expressions within static prop initializer (#7991) (Dunqing)

### Bug Fixes

- 043252d transformer/class-properties: Replace `this` and class name in static blocks (#8035) (overlookmotel)
- 273795d transformer/class-properties: Run other transforms on static properties, static blocks, and computed keys (#7982) (overlookmotel)

### Performance

- 2736657 semantic: Allocate `UnresolvedReferences` in allocator (#8046) (Boshen)
- 0f9308f transformer/react-refresh: Reduce allocations (#8018) (overlookmotel)
- 0deb9e6 transformer/react-refresh: Reserve capacity in hook key string (#8016) (overlookmotel)
- 7b70347 transformer/react-refresh: Avoid allocating string in each hook call (#8013) (Dunqing)

### Refactor

- ac097e9 transformer/class-properties: Rename file (#8036) (overlookmotel)
- 059a5dd transformer/class-properties: Do not pass `ScopeId` into `insert_instance_inits` (#8001) (overlookmotel)
- 0a38eea transformer/class-properties: Use `temp_var_name_base` to generate temp var names for `super` transform (#8004) (overlookmotel)
- d1b7181 transformer/class-properties: Rename var (#8006) (overlookmotel)
- 5a23d72 transformer/class-properties: Remove outdated comment (#8000) (overlookmotel)
- b3a5f3e transformer/class-properties: Mark `transform_assignment_expression_if_super_member_assignment_target` as inline (#7993) (Dunqing)

## [0.42.0] - 2024-12-18

- c071494 semantic: [**BREAKING**] Remove `SymbolTable::rename` method (#7868) (overlookmotel)

### Features

- c16a851 napi/transform: Add `jsx: 'preserve'` option (#7965) (Boshen)
- c30a982 span: Add `impl From<ArenaString> for Atom` (#7973) (overlookmotel)
- 02b653c transformer/class-properties: Do not create temp var for template literal computed key (#7919) (overlookmotel)
- feac02e transformer/class-properties: Only rename symbols if necessary (#7896) (overlookmotel)
- 6bc530d transformer/class-properties: Transform super call expression that is inside static prop initializer (#7831) (Dunqing)
- 53e2bc0 traverse: Add `TraverseScoping::rename_symbol` method (#7871) (overlookmotel)

### Bug Fixes

- 9a30910 oxc_transformer: Inject_global_variables should considering string imported name (#7768) (IWANABETHATGUY)
- 4924073 semantic: `ScopeTree::rename_binding` preserve order of bindings (#7870) (overlookmotel)
- bb38065 transformer/class-properties: Do not transform `super.prop` in nested method within static prop initializer (#7978) (overlookmotel)
- e76fbb0 transformer/class-properties: Fix symbol clashes in instance prop initializers (#7872) (overlookmotel)
- c0576fa transformer/class-properties: Use UID for `args` in created class constructor (#7866) (overlookmotel)
- d660d8d transformer/optional-chaining: Do not create unused reference when `noDocumentAll` assumption (#7847) (overlookmotel)
- 4920c6a transformer/optional-chaining: Avoid creating a useless reference when `noDocumentAll` is true (#7832) (Dunqing)

### Performance

- b31f123 transformer/class-properties: Do not re-generate same method key (#7915) (overlookmotel)
- 8ca8fce transformer/class-properties: Reduce work updating scopes when transforming static prop initializers (#7904) (overlookmotel)
- 80d0b3e transformer/class-properties: Fast path for instance prop initializer scope re-parenting (#7901) (overlookmotel)
- 38aafa2 transformer/class-properties: Reduce size of `transform_call_expression_for_super_member_expr` (#7859) (overlookmotel)

### Documentation

- 10a86b9 transformer: Fix comments (#7925) (overlookmotel)
- f4cb5d3 transformer: Clarify comment (#7918) (overlookmotel)
- 41a1456 transformer/class-properties: Correct doc comments (#7966) (overlookmotel)
- 18441af transformer/class-properties: Remove oudated todo for assignment expression (#7955) (Dunqing)
- 1317c00 transformer/class-properties: Clarify doc comments (#7914) (overlookmotel)
- 9989b58 transformer/class-properties: Re-order file list in doc comment (#7911) (overlookmotel)
- 7390048 transformer/class-properties: Reformat doc comment (#7909) (overlookmotel)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)
- d59bbae transformer: Remove unneeded lint `#[allow]` (#7971) (overlookmotel)
- 2c94236 transformer: Improve encapsulation of transforms (#7888) (overlookmotel)
- 34091b2 transformer: Use `Expression::is_super` (#7852) (overlookmotel)
- d4d7bc0 transformer/async-to-generator: Avoid allocating unnecessary `Atom`s (#7975) (overlookmotel)
- 2e5ffd3 transformer/class-properties: Store `temp_var_is_created` on `ClassBindings` (#7981) (overlookmotel)
- 27cc6da transformer/class-properties: Store `is_declaration` only on `ClassDetails` (#7980) (overlookmotel)
- ee282f8 transformer/class-properties: Remove `move_expression`s (#7979) (overlookmotel)
- 94b376a transformer/class-properties: Simplify logic for when to create temp binding (#7977) (overlookmotel)
- ff9d1b3 transformer/class-properties: Comments about shorter output (#7976) (overlookmotel)
- 6fc40f0 transformer/class-properties: Pass `BoundIdentifier`s by reference (#7968) (overlookmotel)
- 69eeeea transformer/class-properties: Methods take `&self` where possible (#7967) (overlookmotel)
- 98340bb transformer/class-properties: Use stack of `ClassDetails` (#7947) (overlookmotel)
- 088dd48 transformer/class-properties: Shorten code (#7913) (overlookmotel)
- 544ffbf transformer/class-properties: Split up code into multiple files (#7912) (overlookmotel)
- dcaf674 transformer/class-properties: Rename file (#7910) (overlookmotel)
- 6243980 transformer/class-properties: Instance prop inits visitor use `Visit` (#7867) (overlookmotel)
- eb47d43 transformer/class-properties: Re-use existing `Vec` (#7854) (overlookmotel)
- 1380b7b transformer/class-properties: Reduce visibility of method (#7858) (overlookmotel)
- 0f5e078 transformer/class-properties: Rename `*_owner` to `owned_*` (#7855) (Dunqing)
- 4ea90d4 transformer/react-refresh: Calculate signature key once (#7970) (Dunqing)
- 15b9bff transformer/typescript: Reuse `Atom` (#7969) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

### Testing

- e766051 transformer: Skip test which uses filesystem under miri (#7874) (overlookmotel)
- f39e65e transformer: Prevent lint error when running miri (#7873) (overlookmotel)

## [0.41.0] - 2024-12-13

- fb325dc ast: [**BREAKING**] `span` field must be the first element (#7821) (Boshen)

### Features

- e727ae9 transformer/class-properties: Transform super member expressions that are inside static prop initializer (#7815) (Dunqing)

### Bug Fixes

- 5b7e1ad transformer: Remove span of define value (#7811) (Hiroshi Ogawa)
- 14896cb transformer/class-properties: Create temp vars in correct scope (#7824) (overlookmotel)
- 25bb6da transformer/class-properties: Fix `ScopeId`s in instance prop initializers (#7823) (overlookmotel)
- 65b109a transformer/class-properties: No `raw` for generated `StringLiteral` (#7825) (overlookmotel)
- 2964a61 transformer/class-properties: Unwrap failed when private field expression doesn't contain optional expression in `ChainExpression` (#7798) (Dunqing)
- 6fa6785 transformer/class-properties: Panic when the callee or member is `ParenthesisExpression` or TS-syntax expressions. (#7795) (Dunqing)
- bb22c67 transformer/class-properties: Fix `ScopeId`s in static prop initializers (#7791) (overlookmotel)
- caa57f1 transformer/class-properties: Fix scope flags in static prop initializers (#7786) (overlookmotel)

### Refactor

- b290ebd transformer: Handle `<CWD>` in test runner (#7799) (Dunqing)
- e70deb9 transformer/class-properties: Locate instance props insertion location in separate step (#7819) (overlookmotel)
- afc5f1e transformer/class-properties: De-deduplicate code (#7805) (overlookmotel)
- 47a91d2 transformer/class-properties: Shorten code (#7804) (overlookmotel)
- 54ef2b9 transformer/class-properties: Rename `debug_assert_expr_is_not_parenthesis_or_typescript_syntax` (#7803) (overlookmotel)
- 3cdc47c transformer/class-properties: `#[inline(always)]` on `assert_expr_neither_parenthesis_nor_typescript_syntax` (#7802) (overlookmotel)

### Testing

- d72c888 transformer/replace-global-defines: Remove panicking test (#7838) (overlookmotel)

## [0.40.0] - 2024-12-10

- 5913200 transformer/class-properties: [**BREAKING**] Rename `ClassPropertiesOptions::loose` (#7716) (overlookmotel)

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features

- 7dcf6b4 ast, transformer: Add `AstBuilder::use_strict_directive` method (#7770) (overlookmotel)
- cf2ee06 data_structures: Add rope (#7764) (Boshen)
- 2803aec napi/transform: Return helpers information (#7737) (Boshen)
- c98457d napi/transformer: Add runtime helper mode (#7727) (Boshen)
- 2e69720 transformer/class-properties: Support `private_fields_as_properties` assumption (#7717) (overlookmotel)
- 86d4c90 transformer/class-properties: Support for transforming `AssignmentTarget` (#7697) (Dunqing)
- c793d71 transformer/class-properties: Transform `ChainExpression` (#7575) (Dunqing)
- e010b6a transformer/logical-assignment-operators: No temp vars for literals (#7759) (overlookmotel)
- 8705a29 transformer/var-declaration: Add a series of `create_var*` methods (#7665) (Dunqing)
- e8518e9 transformer/var-declarations: Add `insert_var_with_init` method (#7667) (Dunqing)

### Bug Fixes

- f7d41dd oxc_transform: Overlap replacement (#7621) (IWANABETHATGUY)
- 245d7d9 oxc_transformer: Alias `es2015` to `es6` (#7673) (Kevin Deng 三咲智子)
- f42dbdf transformer/class-properties: Output is not the same with Babel when PrivateFieldExpression is optional (#7762) (Dunqing)
- f52b1db transformer/class-properties: Output is not the same with Babel when callee has optional (#7748) (Dunqing)
- 97e4185 transformer/class-properties: Fix `SymbolFlags` for `_super` function (#7709) (overlookmotel)
- de5b0b6 transformer/class-properties: Make `_super` function outside class strict mode (#7708) (overlookmotel)
- 72b5d58 transformer/class-properties: Create temp var for `this` in computed key (#7686) (overlookmotel)
- ad76d1d transformer/class-properties: Transform `delete` chain expression in static prop initializers (#7656) (overlookmotel)
- e48769a transformer/logic-assignment-operator: Always create `IdentifierReference`s with `ReferenceId` (#7745) (overlookmotel)

### Performance

- 6c82589 transformer/class-properties: Replace recursion with loop (#7652) (overlookmotel)
- 463fc5f transformer/logic-assignment-operator: Inline `enter_expression` visitor (#7744) (overlookmotel)

### Documentation

- 5806942 transformer/class-properties: Correct doc comment (#7741) (overlookmotel)
- 583b36b transformer/class-properties: Remove oudated todo (#7669) (Dunqing)

### Refactor

- 9c2a1b6 transformer: `duplicate_expression` do not produce temp var for `super` (#7757) (overlookmotel)
- a750ebc transformer: `duplicate_expression` take `mutated_symbol_needs_temp_var` param (#7756) (overlookmotel)
- b500f55 transformer: Introduce `TransformCtx::duplicate_expression` (#7754) (overlookmotel)
- 75ba4a9 transformer: Use `NONE` in AST builder calls (#7751) (overlookmotel)
- 1925ddc transformer: Rename `VarDeclarationsStore` methods (#7682) (overlookmotel)
- 0ca10e2 transformer: Use `ctx.var_declarations.create_var*` methods (#7666) (Dunqing)
- 016ae92 transformer/class-properties: Rename file (#7767) (overlookmotel)
- 9cacf64 transformer/class-properties: Transform the remaining PrivateFieldExpression in ChainExpression first (#7763) (Dunqing)
- 7e0f7eb transformer/class-properties: Prefer `contains` to `intersects` for bitflags (#7747) (overlookmotel)
- 7344d21 transformer/class-properties: TODO comments for future optimizations (#7711) (overlookmotel)
- dd55b84 transformer/class-properties: Shorten output when `_super` function (#7710) (overlookmotel)
- ac910ee transformer/class-properties: Move code out of `transform_assignment_target` (#7701) (overlookmotel)
- e67e981 transformer/class-properties: Shorten code (#7700) (overlookmotel)
- ab3e1c3 transformer/class-properties: Add TODO comments (#7702) (overlookmotel)
- 28ce187 transformer/class-properties: `duplicate_object_twice` method (#7685) (overlookmotel)
- 8883968 transformer/class-properties: Use `duplicate_object` in `transform_expression_to_wrap_nullish_check` (#7664) (Dunqing)
- 44fe854 transformer/class-properties: Move logic for handling `delete` of chain expression into `transform_unary_expression` (#7655) (overlookmotel)
- 3d593ec var-declarations: Remove unnecessary `init` parameter from `insert_var` (#7668) (Dunqing)

### Styling

- e5145b0 transformer/class-properties: Reformat doc comments (#7653) (overlookmotel)

## [0.39.0] - 2024-12-04

- f2f31a8 traverse: [**BREAKING**] Remove unsound APIs (#7514) (overlookmotel)

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features

- a784a82 oxc_transformer: Support jsx pragma that are long member expressions (#7538) (IWANABETHATGUY)
- a23ce15 oxc_transformer: Replace_global_define for assignmentTarget (#7505) (IWANABETHATGUY)
- 3539f56 transformer/class-properties: Support for transforming `TaggedTemplateExpresssion` (#7504) (Dunqing)

### Bug Fixes

- 64f92e9 oxc_transform: Oxc dot define is postfix of some MemberExpr (#7640) (IWANABETHATGUY)
- 6af8659 oxc_transformer: Correct generate `ThisExpr` and `import.meta` in jsx pragma (#7553) (IWANABETHATGUY)
- 58a125f transformer/async-to-generator: Correct the `SymbolFlags` of function id in module (#7470) (Dunqing)
- eb825ed transformer/class-properties: Replace references to class name with temp var in static prop initializers (#7610) (overlookmotel)
- 0eadd9f transformer/class-properties: Create temp var for class where required (#7516) (overlookmotel)
- 199076b transformer/class-properties: Transform private property accesses in static prop initializers (#7483) (overlookmotel)
- 37842c1 transformer/object-rest-spread: Generate catch variable binding with correct `SymbolFlags` (#7469) (Dunqing)

### Performance

- 7ebe8c2 transformer: Use `FxDashMap` for browser query cache (#7521) (overlookmotel)
- 5ca6eea transformer/class-properties: Inline visitor methods (#7485) (overlookmotel)
- 3b1e63e transformer/jsx: No string comparisons generating pragma expression (#7620) (overlookmotel)

### Documentation

- 370d4b9 transformer/class-properties: Add missing docs (#7588) (overlookmotel)

### Refactor

- d21448b semantic, transformer: Simplify `FxIndexMap` type aliases (#7524) (overlookmotel)
- 7d1c12e transformer/class-properties: Rename misleadingly-named method (#7609) (overlookmotel)
- 802233d transformer/class-properties: Remove pointless method (#7592) (overlookmotel)
- a07f278 transformer/class-properties: `PrivatePropsStack` type (#7589) (overlookmotel)
- 7bd6350 transformer/class-properties: Move creating temp var out of main loop (#7587) (overlookmotel)
- ebd11fb transformer/class-properties: Exit `transform_class` faster if nothing to do (#7586) (overlookmotel)
- dccff38 transformer/class-properties: `ResolvedPrivateProp` type (#7532) (overlookmotel)
- 367b6c8 transformer/class-properties: `shortcut_static_class` take `SymbolId` (#7531) (overlookmotel)
- ab1214d transformer/class-properties: Rename `class_binding` (#7533) (overlookmotel)
- d5aaee7 transformer/class-properties: Remove defunct comments (#7527) (overlookmotel)
- 968863b transformer/class-properties: Move transform logic of `callee` of `CallExpression` to `transform_private_field_callee` (#7503) (Dunqing)
- 5261547 transformer/class-properties: Remove a branch from `transform_call_expression_impl` (#7507) (overlookmotel)
- 1c4b29c transformer/class-properties: Correct comments (#7506) (overlookmotel)
- 8ad52be transformer/jsx: `Pragma::parse` take a `&str` (#7619) (overlookmotel)
- ef62b9d transformer/react-refresh: Use `generate_uid_in_current_hoist_scope` to add hoisted binding (#7492) (Dunqing)

### Testing

- 71b3437 oxc_transformer: Define works differently with esbuild (#7593) (翠 / green)
- 2158c38 transformer/jsx: Move tests setup into a macro (#7618) (overlookmotel)

## [0.38.0] - 2024-11-26

- bb2c0c2 transformer: [**BREAKING**] Return `String` as error instead of OxcDiagnostic (#7424) (Boshen)

### Features

- 59e7e46 napi/transform: Add `TransformOptions::target` API (#7426) (Boshen)
- e9f9e82 oxc_transformer: Replace_global_define ThisExpression (#7443) (IWANABETHATGUY)
- 8797849 oxc_transformer: Replace_global_define destructuring assignment optimization (#7449) (IWANABETHATGUY)
- 4bb1dca oxc_transformer: ReplaceGlobalDefines for ChainExpr (#7433) (IWANABETHATGUY)
- d8c0931 oxc_transformer: Use better diagnostic message for `ReplaceGlobalDefinesPlugin` (#7439) (dalaoshu)
- 21614f2 oxc_transformer: ReplaceGlobalDefinesPlugin for ComputedMemberExpr (#7431) (IWANABETHATGUY)
- 9778298 transformer: Class properties transform (#7011) (overlookmotel)

### Bug Fixes

- 7ff9f13 transformer: Correct all ReferenceFlags (#7410) (Dunqing)
- 4d6bd07 transformer/async-generator-functions: Correct all binding scope id (#7425) (Dunqing)
- 97de0b7 transformer/class-properties: Transform `this` in static prop initializers (#7481) (overlookmotel)
- d2745df transformer/class-properties: Stop searching for `super()` in `TSModuleBlock`s (#7480) (overlookmotel)

### Performance

- e26916c transformer/optional-chaining: Mark `enter_expression` as inline (#7390) (Dunqing)

### Documentation

- 2a5954a transformer/class-properties: Document transform options (#7478) (overlookmotel)

### Refactor

- e5d49db transformer/class-properties: Placeholder method for transforming private field assignment patterns (#7482) (overlookmotel)
- abb0e0e transformer/class-properties: Rename var (#7477) (overlookmotel)
- 25823c8 transformer/class-properties: Safer use of `GetAddress` (#7474) (overlookmotel)
- 3396b69 transformer/exponentiation-operator: Correct comment (#7476) (overlookmotel)
- eb39a50 transformer/logic-assignment: Shorten code (#7419) (overlookmotel)
- 6fd0fcb transformer/object-rest-spread: Avoid multiple symbol lookups (#7420) (overlookmotel)
- 52784d2 transformer/optional-chaining: Avoid multiple symbol lookups (#7421) (overlookmotel)

### Styling

- 111d722 transformer/optional-chaining: Code style nit (#7468) (overlookmotel)

## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

- 41a0e60 ast: [**BREAKING**] Remove `impl GetAddress for Function` (#7343) (overlookmotel)

- 1cbc624 traverse: [**BREAKING**] Rename `TraverseCtx` methods for creating `IdentifierReference`s (#7300) (overlookmotel)

- e84ea2c traverse: [**BREAKING**] Remove `TraverseCtx::clone_identifier_reference` (#7266) (overlookmotel)

- 44375a5 ast: [**BREAKING**] Rename `TSEnumMemberName` enum variants (#7250) (overlookmotel)

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)
- d608012 transform_conformance: Snapshot our transformed outputs (#7358) (Boshen)
- 224775c transformer: Transform object rest spread (#7003) (Boshen)
- 885e37f transformer: Optional Chaining (#6990) (Boshen)
- 6a98ef1 transformer: Add `CompilerAssumptions` to `TransformContext` (#7369) (Boshen)
- faf8dde traverse: Add methods for creating `Expression::Identifier`s (#7301) (overlookmotel)

### Bug Fixes

- b57d00d tasks/compat_data: Fix misplaced features (#7284) (Boshen)
- c5f4ee7 transformer: Correct code comments (#7247) (overlookmotel)
- 389b84e transformer/arrow-function: Handle unicode when capitalizing property name (#7311) (overlookmotel)
- 7d75130 transformer/async-to-generator: `arguments` isn't correct after transformation (#7234) (Dunqing)
- 5b5c8a9 transformer/nullish-coalescing: Correct span (#7269) (overlookmotel)

### Performance

- 510b95d transformer: Use `AstBuilder::vec_from_array` (#7333) (overlookmotel)
- e09d2df transformer/arrow-function: Create super method binding names lazily (#7313) (overlookmotel)
- 0a24703 transformer/arrow-function: Optimize `generate_super_binding_name` (#7312) (overlookmotel)
- 44fd962 transformer/arrow-functions: Move arguments transform checks to aid inlining (#7322) (overlookmotel)
- 26d3e96 transformer/arrow-functions: Store state of whether arguments needs transform (#7321) (overlookmotel)

### Documentation

- e219ae8 transformer/nullish-coalescing: Clarify doc comment (#7268) (overlookmotel)

### Refactor

- 4acf2db transformer: Helper loader methods take `Span` (#7304) (overlookmotel)
- 871e19b transformer/arrow-function: Comments on possible improvement (#7320) (overlookmotel)
- ea08c1f transformer/arrow-function: Reserve correct capacity for `Vec` (#7319) (overlookmotel)
- 5cfe0b6 transformer/arrow-function: `generate_super_binding_name` take `&str` and `&TraverseCtx` (#7310) (overlookmotel)
- 5d85386 transformer/arrow-functions: Use `IndexMap` for `super` getter/setters (#7317) (overlookmotel)
- 9f5ae56 transformer/nullish-coalescing: Split main logic into separate function (#7273) (overlookmotel)
- 345fbb9 transformer/nullish-coalescing: Avoid repeated symbol lookups (#7272) (overlookmotel)

## [0.36.0] - 2024-11-09

- b11ed2c ast: [**BREAKING**] Remove useless `ObjectProperty::init` field (#7220) (Boshen)

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- 846711c transformer: [**BREAKING**] Change API to take a `&TransformOptions` instead of `TransformOptions` (#7213) (Boshen)

- d1d1874 ast: [**BREAKING**] Change `comment.span` to real position that contain `//` and `/*` (#7154) (Boshen)

- 843bce4 ast: [**BREAKING**] `IdentifierReference::reference_id` return `ReferenceId` (#7126) (overlookmotel)

### Features

- ad3a2f5 tasks/compat_data: Generate our own compat table (#7176) (Boshen)
- b4258ee transformer: Add defaulted `Module::Preserve` option (#7225) (Boshen)
- 324c3fe transformer: Add `TransformOptions::module` option (#7188) (Boshen)
- a166a4a transformer: Add esbuild comma separated target API `--target=es2020,chrome58` (#7210) (Boshen)
- 3a20b90 transformer: Add es target to `engineTargets` (#7193) (Boshen)
- 22898c8 transformer: Warn BigInt when targeting < ES2020 (#7184) (Boshen)
- a579011 transformer: Add features `ES2018NamedCapturingGroupsRegex` and `ES2018LookbehindRegex` (#7182) (Boshen)
- 8573f79 transformer: Turn on async_to_generator and async_generator_functions plugins in enable_all (#7135) (Dunqing)
- df77241 transformer: Enable `ArrowFunctionConverter` in `async-to-generator` and `async-generator-functions` plugins (#7113) (Dunqing)
- b6a5750 transformer/arrow-function-converter: Move scope to changed scope for `this_var` if scope have changed (#7125) (Dunqing)
- 1910227 transformer/async-to-generator: Support inferring the function name from the ObjectPropertyValue's key (#7201) (Dunqing)
- ffa8604 transformer/async-to-generator: Do not transform await expression if is not inside async function (#7138) (Dunqing)
- e536d47 transformer/babel: Add support for trying to get the `Module` from `BabelPlugins` (#7218) (Dunqing)
- 5cfdc05 transformer/typescript: Support transform `export =` and `import = require(...)` when module is commonjs (#7206) (Dunqing)

### Bug Fixes

- c82b273 transformer/async-generator-functions: Only transform object method in exit_function (#7200) (Dunqing)
- b2a888d transformer/async-generator-functions: Incorrect transformation for `for await` if it's not placed in a block (#7148) (Dunqing)
- 19892ed transformer/async-generator-functions: Transform incorrectly for `for await` if it's in LabeledStatement (#7147) (Dunqing)
- ede10dc transformer/async-to-generator: Incorrect transform when super expression is inside async method (#7171) (Dunqing)
- 293d072 transformer/async-to-generator: Only transform object method in exit_function (#7199) (Dunqing)
- ae692d7 transformer/async_to_generator: Fix checking if function is class method (#7117) (overlookmotel)
- eea4ab8 transformer/helper-loader: Incorrect `SymbolFlags` for default import when `SourceType` is script (#7226) (Dunqing)

### Refactor

- de56083 transformer: Add `impl TryFrom<EngineTargets> for EnvOptions` (#7191) (Boshen)
- 0a43c64 transformer: Move `ESTarget` to its own file (#7189) (Boshen)
- 0e1f12c transformer: Remove unimplemented `EnvOptions::bugfixes` (#7162) (Boshen)
- a981caf transformer: Add `Engine` enum for `EngineTargets` (#7161) (Boshen)
- 8340243 transformer: Rename `Query` to `BrowserslistQuery` (#7143) (Boshen)
- 481f7e6 transformer: Change `Targets` to `EngineTargets` (#7142) (Boshen)
- 55e6989 transformer: Deserialize engine target strings to specific keys (#7139) (Boshen)
- fdfd9a4 transformer: Use `scope_id` etc methods (#7128) (overlookmotel)
- ff8bd50 transformer: Move implementation of ArrowFunction to common/ArrowFunctionConverter (#7107) (Dunqing)
- 4a515be transformer/arrow-function-coverter: Rename function name and add some comments to explain confusing parts. (#7203) (Dunqing)
- c307e1b transformer/arrow-functions: Pass `ArenaBox` as function param (#7169) (overlookmotel)
- 217d433 transformer/arrow-functions: Remove unused `&mut self` function param (#7165) (overlookmotel)
- 426df71 transformer/arrow-functions: Use `scope_id` method (#7164) (overlookmotel)
- 11c5e12 transformer/arrow-functions: Correct comments (#7163) (overlookmotel)
- 1238506 transformer/async-generator-function: Remove inactive `#[allow(clippy::unused_self)]` attrs (#7167) (overlookmotel)
- 84ee581 transformer/async-generator-functions: Simplify identifying whether within an async generator function (#7170) (overlookmotel)
- 1b12328 transformer/async-generator-functions: Use `clone` not `clone_in` on `LabelIdentifier` (#7172) (overlookmotel)
- cd1006f transformer/async-generator-functions: Do not transform yield expression where inside generator function (#7134) (Dunqing)
- 2c5734d transformer/async-generator-functions: Do not transform await expression where inside ArrowFunctionExpression (#7132) (Dunqing)
- 5ce83bd transformer/async-generator-functions: Remove dead code for handle await expression (#7131) (Dunqing)
- e04ee97 transformer/async-generator-functions: Move handling of `MethodDefinition`'s value to `exit_function` (#7106) (Dunqing)
- b57d5a5 transformer/async-to-generator: Remove unused `&self` function param (#7166) (overlookmotel)
- f80085c transformer/async-to-generator: Move handling of `MethodDefinition`'s value to `exit_function` (#7105) (Dunqing)
- e2241e6 transformer/jsx-self: Remove unused `&self` function params (#7159) (overlookmotel)
- 1dfd241 transformer/optional-catch-binding: Remove inactive `#[allow(clippy::unused_self)]` attr (#7158) (overlookmotel)
- fd9b44c transformer/typescript: Remove inactive `#[allow(clippy::unused_self)]` attr (#7160) (overlookmotel)

### Styling

- 38a6df6 transformer/arrow-functions: Semicolon after return statements (#7168) (overlookmotel)
- 64b7e3a transformer/async-generator-functions: Import `oxc_allocator::Vec` as `ArenaVec` (#7173) (overlookmotel)

## [0.35.0] - 2024-11-04

- b8daab3 transformer: [**BREAKING**] API to `TryFrom<&EnvOptions> for TransformOptions` and `TryFrom<&BabelOptions> TransformOptions` (#7020) (Boshen)

### Features

- bfdbcf1 transformer: Add `EnvOptions::from_browerslist_query` API (#7098) (Boshen)
- 21b8e49 transformer: Add `ESTarget` (#7091) (Boshen)
- fcaba4a transformer: Add `TransformerOptions::env` with `EnvOptions` (#7037) (Boshen)
- 1d906c6 transformer: Class properties transform skeleton (#7038) (overlookmotel)
- 934cb5e transformer: Add `async_generator_functions` plugin (#6573) (Dunqing)

### Bug Fixes

- a2244ff transformer/async-to-generator: Output is incorrect when arrow function without params (#7052) (Dunqing)

### Refactor

- 7f1d1fe transform: Deserialize `BabelPreests::env` directly (#7051) (Boshen)
- 76947e2 transform: Refactor Babel Targets (#7026) (Boshen)
- d03e622 transformer: Do not use `AstBuilder::*_from_*` methods (#7070) (overlookmotel)
- 9d384ad transformer: Use `identifier_reference_with_reference_id` builder method (#7056) (overlookmotel)
- 4688a06 transformer: Use `*_with_scope_id` builder methods where possible (#7055) (overlookmotel)
- 7122e00 transformer: Use `ctx.alloc` over `ctx.ast.alloc` where possible (#7066) (overlookmotel)
- a3b68b4 transformer: Flatten dir structure of options/babel/env (#7049) (Boshen)
- 6d92f36 transformer: Deserialize `BabelOptions::compiler_assumptions` (#7048) (Boshen)
- f83a760 transformer: Deserialize `BabelOptions::presets` (#7047) (Boshen)
- 52c20d6 transformer: Deserialize `BabelOptions::plugins` (#7045) (Boshen)
- e921df6 transformer: Rename `EnvOptions` to `BabelEnvOptions` (#7036) (Boshen)
- af5140f transformer: Isolate babel options logic (#7034) (Boshen)
- 12aa910 transformer: Clean up `env/targets/query.rs` (#7033) (Boshen)
- 3d174bb transformer: Clean up `BabelOptions` (#7029) (Boshen)
- 6284f84 transformer: Use `Browserslist::Version` (#7028) (Boshen)
- 5b11cdf transformer: Clean up TransformerOptions (#7005) (Boshen)
- f0c87d4 transformer: Mark all EnvOptions as not implemented (#7004) (Boshen)
- d9edef6 transformer: Combine ObjectRestSpread into a single file (#7002) (Boshen)
- c945fe7 transformer: Import `oxc_allocator::Box` as `ArenaBox` (#6999) (overlookmotel)
- fc1af2e transformer: Import `oxc_allocator::Vec` as `ArenaVec` (#6998) (overlookmotel)
- 63e8bfe transformer: Rename `AString` to `ArenaString` (#6997) (overlookmotel)
- 562bb9a transformer/async-to-generator: Move transform methods to `AsyncGeneratorExecutor` and make it public (#6992) (Dunqing)
- e23f7e6 transformer/common: `VarDeclarations` insert either `var` or `let` statements (#7043) (overlookmotel)
- e5ecbb9 transformer/jsx: Return `&mut T` not `&mut ArenaBox<T>` (#7001) (overlookmotel)
- 9e66c29 transformer/react-refresh: Small refactor (#6973) (overlookmotel)
- 1ca8cd2 transformer/react-refresh: Avoid panic for `init` of `VariableDeclarator` isn't a `BindingIdentifier` (#6937) (Dunqing)
- 5f153ac transformer/react-refresh: Use `VarDeclarations` to insert declarators (#6884) (Dunqing)
- df3b089 transformer/react-refresh: Use `StatementInjector` to insert statements (#6881) (Dunqing)
- ae22671 transformer/typescript: Pass `&mut T` not `&mut ArenaBox<T>` (#7000) (overlookmotel)

### Styling

- 86ab091 transformer/common: Split up `StatementInjectorStore` methods into blocks (#7042) (overlookmotel)

### Testing

- 6133a50 transformer: Use a single integration test for faster compilation (#7099) (Boshen)

## [0.34.0] - 2024-10-26

- 4618aa2 transformer: [**BREAKING**] Rename `TransformerOptions::react` to `jsx` (#6888) (Boshen)

### Features

- 0d0bb17 transformer: Complete the async-to-generator plugin (#6658) (Dunqing)

### Bug Fixes

- 4dc5e51 transformer: Only run typescript plugin for typescript source (#6889) (Boshen)
- 076f5c3 transformer/typescript: Retain ExportNamedDeclaration without specifiers and declaration (#6848) (Dunqing)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)
- 2d95009 transformer: Implement `Debug` on `StatementInjector` internal types (#6886) (overlookmotel)
- c383c34 transformer: Make `StatementInjectorStore` methods generic over `GetAddress` (#6885) (overlookmotel)
- 1f29523 transformer: Rename ReactJsx to Jsx (#6883) (Boshen)
- 333b758 transformer: `StatementInjectorStore` methods take `&Statement` as target (#6858) (overlookmotel)
- c19996c transformer: Add `StatementInjectorStore::insert_many_before` method (#6857) (overlookmotel)
- 7339dde transformer: `StatementInjectorStore::insert_many_after` take an iterator (#6856) (overlookmotel)
- 4348eae transformer/typescript: Re-order visitor methods (#6864) (overlookmotel)
- 3a56d59 transformer/typescript: Insert assignments after super by `StatementInjector` (#6654) (Dunqing)
- 60f487a traverse: `TraverseCtx::generate_binding` take an `Atom` (#6830) (overlookmotel)

## [0.33.0] - 2024-10-24

- 4d2d214 ast, transformer: [**BREAKING**] Remove `StringLiteral::new` method (#6788) (overlookmotel)

- aeaa27a ast, parser, transformer, traverse: [**BREAKING**] Remove `BindingIdentifier::new` methods (#6786) (overlookmotel)

- ecc9151 ast, parser, transformer, traverse: [**BREAKING**] Remove `IdentifierReference::new` methods (#6785) (overlookmotel)

- c91ffbc ast, transformer: [**BREAKING**] Remove `IdentifierName::new` method (#6784) (overlookmotel)

- 2bee4e2 ast, transformer: [**BREAKING**] Remove `BlockStatement::new` methods (#6783) (overlookmotel)

- 8032813 regular_expression: [**BREAKING**] Migrate to new regexp parser API (#6741) (leaysgur)

### Features

- 10484cd transformer: Class static block transform (#6733) (overlookmotel)
- 7fbca9d transformer: Introduce `StatementInjector` helper (#6653) (Dunqing)

### Bug Fixes

- 1107770 coverage: Inject babel helpers for transform (#6818) (Boshen)
- b711ee1 transformer: After using StatementInjector, some statements disappeared (#6778) (Dunqing)

### Documentation

- ab03535 transformer: Correct typos and reformat doc comments (#6758) (overlookmotel)

### Refactor

- ab8aa2f allocator: Move `GetAddress` trait into `oxc_allocator` (#6738) (overlookmotel)
- 0e9b695 ast: Change `plain_function` to accept `FunctionBody` as a required parameter (#6709) (Dunqing)
- b8dfa19 transformer: Shorten code (#6809) (overlookmotel)
- 759710a transformer: Methods only take `&TraverseCtx` where possible (#6812) (overlookmotel)
- 06e06e3 transformer: Rename `OxcVec` to `AVec` (#6737) (overlookmotel)
- e5f4b4a transformer/react-refresh: Dereference `ScopeId` as soon as possible (#6820) (overlookmotel)
- 57685b2 transformer/react-refresh: Unwrap `BindingIdentifier::symbol_id` (#6817) (overlookmotel)
- 4f6dc22 transformer/react-refresh: Avoid re-creating `Atom`s (#6816) (overlookmotel)
- 8316069 transformer/react-refresh: Shorten code by using `BoundIdentifier` (#6815) (overlookmotel)
- fdd69e4 transformer/typescript: Use `TraverseCtx::generate_binding` to create a symbol (#6806) (Dunqing)

### Styling

- 871b9f5 transformer/react-refresh: Fix whitespace (#6813) (overlookmotel)

## [0.32.0] - 2024-10-19

- 5200960 oxc: [**BREAKING**] Remove passing `Trivias` around (#6446) (Boshen)

### Features

- a01a5df transformer: Pass TransformerCtx to async-to-generator plugin (#6633) (Dunqing)
- a9260cf transformer: `async-to-generator` plugin. (#5590) (Ethan Goh)
- 8fe1b0a transformer: Support helper loader (#6162) (Dunqing)
- ab51c2a transformer: Support `DefaultImport` in `ModuleImports` (#6434) (Dunqing)
- a3dea9c transformer/async-to-generator: Handle arrow-function correctly (#6640) (Dunqing)
- 41c8675 transformer/object-rest-spread: Using helper loader (#6449) (Dunqing)

### Bug Fixes

- 1d3d256 transformer: Correctly trim JSX (#6639) (magic-akari)
- c6f2b5f transformer: `HelperLoader` common transform: do not assume `babelHelpers` is global (#6569) (overlookmotel)
- 85d93ed transformer: Arrow function transform: correctly resolve `this` in class accessor properties (#6386) (overlookmotel)

### Performance

- f70a413 transformer: Object spread transform: do not lookup `Object` binding if not needed (#6570) (overlookmotel)

### Documentation

- f3451d7 transformer/async-to-generator: Remove empty lines from doc comment (#6642) (overlookmotel)
- 448388a transformer/module_imports: Update outdated comments (#6574) (Dunqing)

### Refactor

- 856cab5 ecmascript: Move ToInt32 from `oxc_syntax` to `oxc_ecmascript` (#6471) (Boshen)
- 1ba2a24 ecmascript: Remove `HasProto` which is not part of the spec (#6470) (Boshen)
- 435a89c oxc: Remove useless `allocator.alloc(program)` calls (#6571) (Boshen)
- 9281234 transformer: Shorten imports (#6643) (overlookmotel)
- 3af0840 transformer: `HelperLoader`: add import immediately (#6601) (overlookmotel)
- f81aa7f transformer: `HelperLoader` common transform: comments (#6599) (overlookmotel)
- 679cc68 transformer: `HelperLoader` common transform: construct string directly in arena (#6596) (overlookmotel)
- c346ebb transformer: `HelperLoader` common transform: `Helper` enum (#6595) (overlookmotel)
- 7a028b3 transformer: Remove unnecessary `#![warn]` attr (#6585) (overlookmotel)
- 8c6afe0 transformer: Reorder imports (#6582) (overlookmotel)
- 779ff46 transformer: `HelperLoader` common transform: `Helper` struct (#6568) (overlookmotel)
- bc24a24 transformer: `HelperLoader` common transform: use hashmap `Entry` API (#6567) (overlookmotel)
- 9f02fc7 transformer: `HelperLoader` common transform: re-order fields (#6565) (overlookmotel)
- 50ecade transformer: `HelperLoader` common transform: remove `Rc`s (#6564) (overlookmotel)
- 1c1e9fc transformer: `HelperLoader` common transform: reorder methods (#6563) (overlookmotel)
- c9054c8 transformer: Rename `ImportKind` to `Import` (#6561) (overlookmotel)
- 9542c4e transformer: Add more specific methods to `ModuleImportsStore` (#6560) (overlookmotel)
- 7e57a1d transformer: `ImportKind` use `BoundIdentifier` (#6559) (overlookmotel)
- 602df9d transformer: Re-order fields of `Common` and `TransformCtx` (#6562) (overlookmotel)
- 390abca transformer/async-to-generator: Use `helper_call_expr` (#6634) (Dunqing)
- 2ff917f transformer/async-to-generator: Move internal methods below entry points (#6632) (Dunqing)

### Styling

- 9d43a11 transformer: Re-order dependencies (#6659) (overlookmotel)

## [0.31.0] - 2024-10-08

- 01b878e parser: [**BREAKING**] Use `BindingIdentifier` for `namespace` declaration names (#6003) (DonIsaac)

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

- 409dffc traverse: [**BREAKING**] `generate_uid` return a `BoundIdentifier` (#6294) (overlookmotel)

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

- 4f6bc79 transformer: [**BREAKING**] Remove `source_type` param from `Transformer::new` (#6251) (overlookmotel)

- 82ab689 transformer,minifier: [**BREAKING**] Move define and inject plugin from minifier to transformer (#6199) (Boshen)

### Features

- c3c3447 data_structures: Add `oxc_data_structures` crate; add stack (#6206) (Boshen)
- 51a78d5 napi/transform: Rename all mention of React to Jsx; remove mention of `Binding` (#6198) (Boshen)
- 9e62396 syntax_operations: Add crate `oxc_ecmascript` (#6202) (Boshen)
- cf20f3a transformer: Exponentiation transform: support private fields (#6345) (overlookmotel)

### Bug Fixes

- 9736aa0 oxc_transformer: Define `import.meta` and `import.meta.*` (#6277) (IWANABETHATGUY)
- 2bcd12a transformer: Exponentiation transform: fix reference flags (#6330) (overlookmotel)
- 28cbfa7 transformer: Exponentiation transform: fix temp var names (#6329) (overlookmotel)
- 3a4bcc7 transformer: Exponentiation transform: fix temp var names (#6318) (overlookmotel)
- ccb7bdc transformer: Exponentiation transform: do not replace object when private property (#6313) (overlookmotel)
- 56d50cf transformer: Exponentiation transform: do not assume `Math` is not a local var (#6302) (overlookmotel)
- bd81c51 transformer: Exponentiation transform: fix duplicate symbols (#6300) (overlookmotel)
- 06797b6 transformer: Logical assignment operator transform: fix reference IDs (#6289) (overlookmotel)
- 4b42047 transformer: Fix memory leak in `ReplaceGlobalDefines` (#6224) (overlookmotel)
- a28926f transformer: Fix inserting `require` with `front` option (#6188) (overlookmotel)
- b92fe84 transformer: `NonEmptyStack::push` write value before updating cursor (#6169) (overlookmotel)

### Performance

- 788e444 transformer: Parse options from comments only once (#6152) (overlookmotel)
- da2b2a4 transformer: Look up `SymbolId` for `require` only once (#6192) (overlookmotel)
- 40bd919 transformer: Faster parsing JSX pragmas from comments (#6151) (overlookmotel)

### Documentation

- eb1d0b8 transformer: Exponentiation transform: update doc comments (#6315) (overlookmotel)

### Refactor

- bd5fb5a transformer: Exponentiation transform: rename methods (#6344) (overlookmotel)
- 4aa4e6b transformer: Exponentiation transform: do not wrap in `SequenceExpression` if not needed (#6343) (overlookmotel)
- a15235a transformer: Exponentiation transform: no cloning (#6338) (overlookmotel)
- 7d93b25 transformer: Exponentiation transform: split into 2 paths (#6316) (overlookmotel)
- 15cc8af transformer: Exponentiation transform: break up into functions (#6301) (overlookmotel)
- 7f5a94b transformer: Use `Option::get_or_insert_with` (#6299) (overlookmotel)
- 0cea6e9 transformer: Exponentiation transform: reduce identifier cloning (#6297) (overlookmotel)
- ac7a3ed transformer: Logical assignment transform: reduce identifier cloning (#6296) (overlookmotel)
- 527f7c8 transformer: Nullish coalescing transform: no cloning identifier references (#6295) (overlookmotel)
- 7b62966 transformer: Move `BoundIdentifier` into `oxc_traverse` crate (#6293) (overlookmotel)
- c7fbf68 transformer: Logical assignment operator transform: no cloning identifier references (#6290) (overlookmotel)
- f0a74ca transformer: Prefer `create_bound_reference_id` to `create_reference_id` (#6282) (overlookmotel)
- ba3e85b transformer: Fix spelling (#6279) (overlookmotel)
- bc757c8 transformer: Move functionality of common transforms into stores (#6243) (overlookmotel)
- 1c31932 transformer: Rename var in `VarDeclarations` common transform (#6242) (overlookmotel)
- 0400ff9 transformer: `VarDeclarations` common transform: check if at top level with `ctx.parent()` (#6231) (overlookmotel)
- 235cdba transformer: Use AstBuilder instance from TraverseCtx (#6209) (overlookmotel)
- a7ed29e transformer: Insert `import` statement or `require` depending on source type (#6191) (overlookmotel)
- 4c63f0e transformer: Rename methods (#6190) (overlookmotel)
- 900cb46 transformer: Convert `ModuleImports` into common transform (#6186) (overlookmotel)
- 00e2802 transformer: Introduce `TopLevelStatements` common transform (#6185) (overlookmotel)
- 70d4c56 transformer: Rename `VarDeclarationsStore` methods (#6184) (overlookmotel)
- 81be545 transformer: Export `var_declarations` module from `common` module (#6183) (overlookmotel)
- 02fedf5 transformer: Shorten import (#6180) (overlookmotel)
- f2ac584 transformer: Use TraverseCtx's ast in ModuleImports (#6175) (Dunqing)
- 21b08ba transformer: Shared `VarDeclarations` (#6170) (overlookmotel)
- 0dd9a2e traverse: Add helper methods to `BoundIdentifier` (#6341) (overlookmotel)

## [0.30.5] - 2024-09-29

### Bug Fixes

- bfd1988 transformer/react: Should not collect use-hooks if it's a nested member expression (#6143) (Dunqing)

### Refactor

- 375bebe transformer: Improve parsing React pragmas (#6138) (overlookmotel)
- 0836f6b transformer: Move parsing pragmas into TS transform (#6137) (overlookmotel)
- 30424fa transformer: TS transforms only store options they need (#6135) (overlookmotel)

## [0.30.4] - 2024-09-28

### Bug Fixes

- 64d4756 transformer: Fix debug assertion in `Stack` (#6106) (overlookmotel)

### Refactor

- 7bc3988 transformer: Remove dead code (#6124) (overlookmotel)
- 07fe45b transformer: Exponentiation operator: convert to match (#6123) (overlookmotel)
- 4387845 transformer: Share `TypeScriptOptions` with ref not `Rc` (#6121) (overlookmotel)
- 09e41c2 transformer: Share `TransformCtx` with ref not `Rc` (#6118) (overlookmotel)
- 58fd6eb transformer: Pre-allocate more stack space (#6095) (overlookmotel)
- 9ac80bd transformer: Add wrapper around `NonNull` (#6115) (overlookmotel)
- c50500e transformer: Move common stack functionality into `StackCommon` trait (#6114) (overlookmotel)
- 9839059 transformer: Simplify `StackCapacity` trait (#6113) (overlookmotel)

## [0.30.2] - 2024-09-27

### Features

- 60c52ba ast: Allow passing span to `void_0` method (#6065) (Dunqing)
- 28da771 transformer: Do not transform `**` with bigint literals (#6023) (Boshen)

### Bug Fixes

- c8682e9 semantic,codegen,transformer: Handle definite `!` operator in variable declarator (#6019) (Boshen)

### Performance

- 85aff19 transformer: Introduce `Stack` (#6093) (overlookmotel)
- ad4ef31 transformer: Introduce `NonEmptyStack` (#6092) (overlookmotel)

### Refactor

- e60ce50 transformer: Add `SparseStack::with_capacity` method (#6094) (overlookmotel)
- 1399d2c transformer: Move `SparseStack` definition into folder (#6091) (overlookmotel)
- 6bd29dd transformer: Add more debug assertions (#6090) (overlookmotel)
- c90b9bf transformer: Rename `SparseStack` methods (#6089) (overlookmotel)
- 2b380c8 transformer: Remove unsued `self.ctx` (#6022) (Boshen)

### Testing

- a4cec75 transformer: Enable tests (#6032) (overlookmotel)

## [0.30.1] - 2024-09-24

### Performance

- 7b90d79 transformer: `SparseStack` always keep minimum 1 entry (#5962) (overlookmotel)
- 28fe80a transformer: Logical assignment operator transform use `SparseStack` (#5960) (overlookmotel)
- 9f7d4b7 transformer: Exponentiation operator transform use `SparseStack` (#5959) (overlookmotel)
- 5dc0154 transformer: Nullish coalescing operator transform use `SparseStack` (#5942) (overlookmotel)
- 618e89e transformer: Arrow function transform: reduce stack memory usage (#5940) (overlookmotel)

### Documentation

- 860f108 transformer: Add to arrow functions transform docs (#5989) (overlookmotel)

### Refactor

- f02bf51 transformer: Arrow function transform: remove unnecessary assertion (#6002) (overlookmotel)

## [0.30.0] - 2024-09-23

- c96b712 syntax: [**BREAKING**] Remove `SymbolFlags::ArrowFunction` (#5857) (overlookmotel)

### Features

- 3230ae5 semantic: Add `SemanticBuilder::with_excess_capacity` (#5762) (overlookmotel)
- a07f03a transformer: Sync `Program::source_type` after transform (#5887) (Boshen)

### Bug Fixes

- 87323c6 transformer: Arrow function transform: prevent stack getting out of sync (#5941) (overlookmotel)
- 4e9e838 transformer: Fix arrow function transform (#5933) (overlookmotel)
- 4d5c4f6 transformer: Fix reference flags in logical assignment operator transform (#5903) (overlookmotel)
- d335a67 transformer: Fix references in logical assignment operator transform (#5896) (overlookmotel)
- 9758c1a transformer: JSX source: add `var _jsxFileName` statement (#5894) (overlookmotel)
- 49ee1dc transformer: Arrow function transform handle `this` in arrow function in class static block (#5848) (overlookmotel)
- 172fa03 transformer: Fix stacks in arrow function transform (#5828) (overlookmotel)
- d74c7fa transformer: Remove `AstBuilder::copy` from arrow functions transform (#5825) (overlookmotel)
- 3cc38df transformer/react: React refresh panics when encounter `use` hook (#5768) (Dunqing)

### Performance

- ff7d9c1 transformer: Arrow function transform: calculate whether `this` is in arrow function lazily (#5850) (Dunqing)
- fd70c4b transformer: Arrow function transform more efficient scope search (#5842) (overlookmotel)
- 56703a3 transformer: Make branch more predictable in arrow function transform (#5833) (overlookmotel)
- 36e698b transformer: Call `transform_jsx` in `exit_expression` rather than `enter_expression` (#5751) (Dunqing)
- aac8316 transformer/react: Improve `is_componentish_name`'s implementation (#5769) (Dunqing)

### Documentation

- 7085829 transformer: Arrow function transform: comment about incomplete implementation (#5945) (overlookmotel)
- 66b4688 transformer: React: convert docs to standard format (#5891) (overlookmotel)
- 7f05eed transformer: Add comment about missing features in arrow function transform (#5855) (overlookmotel)
- 8770647 transformer: Correct docs for arrow function transform (#5854) (overlookmotel)

### Refactor

- 155d7fc transformer: Arrow function transform: ignore type fields when finding enclosing arrow function (#5944) (overlookmotel)
- 2cf5607 transformer: Split up logical assignment operator transform into functions (#5902) (overlookmotel)
- 41fbe15 transformer: Internal functions not `pub` in logical assignment operator transform (#5898) (overlookmotel)
- b11d91c transformer: Remove nested match in logical assignment operator transform (#5897) (overlookmotel)
- 52c9903 transformer: JSX: use `AstBuilder::vec_from_iter` (#5862) (overlookmotel)
- 74364ad transformer: JSX: merge `transform_jsx_attribute_item` into `transform_jsx` (#5861) (overlookmotel)
- d2eaa7d transformer: Reorder match arms in JSX transform (#5860) (overlookmotel)
- 58a8327 transformer: Simplify match in JSX transform (#5859) (overlookmotel)
- b9c4564 transformer: Transformer example output semantic + transformer errors (#5852) (overlookmotel)
- 03e02a0 transformer: Comment about potential improvement to arrow function transform (#5841) (overlookmotel)
- 40cdad5 transformer: Remove repeat code in arrow function transform (#5837) (overlookmotel)
- 3dd188c transformer: Deref `SymbolId` immediately (#5836) (overlookmotel)
- 03a9e1a transformer: Reorder methods in arrow function transform (#5830) (overlookmotel)
- 4d97184 transformer: Rename vars in arrow function transform (#5827) (overlookmotel)
- 01c5b7c transformer: Shorten code in arrow functions transform (#5826) (overlookmotel)
- 85ac3f7 transformer: Arrow functions transform do not wrap function expressions in parentheses (#5824) (overlookmotel)
- 1c1353b transformer: Use AstBuilder instead of using struct constructor (#5778) (Boshen)

## [0.29.0] - 2024-09-13

### Features

- 953fe17 ast: Provide `NONE` type for AST builder calls (#5737) (overlookmotel)

### Bug Fixes

- 77d9170 transformer/react: IsStaticChildren should be false when there is only one child (#5745) (Dunqing)

### Refactor

- 4bdc202 rust: Remove some #[allow(unused)] (#5716) (Boshen)
- cc0408b semantic: S/AstNodeId/NodeId (#5740) (Boshen)

## [0.28.0] - 2024-09-11

- ee4fb42 ast: [**BREAKING**] Reduce size of `WithClause` by `Box`ing it (#5677) (Boshen)

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Features

- 95a6d99 transformer: Enable the react refresh plugin in enable_all (#5630) (Dunqing)
- 7b543df transformer/react: Handle `refresh_sig` and `refresh_reg` options correctly (#5638) (Dunqing)

### Bug Fixes

- 1bc08e2 coverage: Parse babel unambiguously (#5579) (Boshen)
- 919d17f transform_conformance: Only print semantic mismatch errors when output is correct (#5589) (Boshen)
- 505d064 transformer: JSX transform delete references for `JSXClosingElement`s (#5560) (overlookmotel)
- 9b7ecc7 transformer: RegExp transform only set span on final expression (#5508) (overlookmotel)
- d1ece19 transformer: RegExp transform handle `Term::Quantifier` (#5501) (overlookmotel)
- a1afd48 transformer/react: Incorrect scope_id for var hoisted in fast refresh plugin (#5695) (Dunqing)
- f2f5e5a transformer/react: Missing scope_id for function in fast refresh plugin (#5693) (Dunqing)
- a891c31 transformer/react: Refresh plugin has incorrect reference flags (#5656) (Dunqing)
- 3e8b96f transformer/react: The refresh plugin cannot handle member expressions with React hooks (#5655) (Dunqing)
- 0739b5f transformer/react: Don't transform declaration of function overloads (#5642) (Dunqing)
- 3bf6aaf transformer/react: Support `emit_full_signatures` option in refresh plugin (#5629) (Dunqing)
- 36d864a transformer/react: Don't transform if the variable does not have a value reference (#5528) (Dunqing)

### Performance


### Documentation

- 9282647 transformer: Comment on RegExp transform for potential improvement (#5514) (overlookmotel)

### Refactor

- 0ac420d linter: Use meaningful names for diagnostic parameters (#5564) (Don Isaac)
- ce71982 transformer: Shorten code in JSX transform (#5554) (overlookmotel)
- 758a10c transformer: RegExp transform reuse var (#5527) (overlookmotel)
- fad0a05 transformer: RegExp transform unbox early (#5504) (overlookmotel)

## [0.27.0] - 2024-09-06

- cba93f5 ast: [**BREAKING**] Add `ThisExpression` variants to `JSXElementName` and `JSXMemberExpressionObject` (#5466) (overlookmotel)

### Features

- 32d4bbb transformer: Add `TransformOptions::enable_all` method (#5495) (Boshen)
- c59d8b3 transformer: Support all /regex/ to `new RegExp` transforms (#5387) (Dunqing)

### Bug Fixes

- 8f9627d transformer: RegExp transform do not transform invalid regexps (#5494) (overlookmotel)
- 2060efc transformer: RegExp transform don't transform all RegExps (#5486) (overlookmotel)
- cfe5497 transformer: Do not create double reference in JSX transform (#5414) (overlookmotel)
- 0617249 transformer/nullish-coalescing-operator: Incorrect reference flags (#5408) (Dunqing)

### Performance

- ed8937e transformer: Memoize rope instance (#5518) (Dunqing)
- bfab091 transformer: Store needed options only on `RegExp` (#5484) (overlookmotel)
- b4765af transformer: Pre-calculate if unsupported patterns in RegExp transform (#5483) (overlookmotel)
- 182ab91 transformer: Pre-calculate unsupported flags in RegExp transform (#5482) (overlookmotel)

### Refactor

- a96866d transformer: Re-order imports (#5499) (overlookmotel)
- 6abde0a transformer: Clarify match in RegExp transform (#5498) (overlookmotel)
- 09c522a transformer: RegExp transform report pattern parsing errors (#5496) (overlookmotel)
- dd19823 transformer: RegExp transform do not take ownership of `Pattern` then reallocate it (#5492) (overlookmotel)
- 2514cc9 transformer/react: Move all entry points to implementation of Traverse trait (#5473) (Dunqing)
- c984219 transformer/typescript: Move all entry points to implementation of Traverse trait (#5422) (Dunqing)

## [0.26.0] - 2024-09-03

- 1aa49af ast: [**BREAKING**] Remove `JSXMemberExpressionObject::Identifier` variant (#5358) (Dunqing)

- 32f7300 ast: [**BREAKING**] Add `JSXElementName::IdentifierReference` and `JSXMemberExpressionObject::IdentifierReference` (#5223) (Dunqing)

- 23e8456 traverse: [**BREAKING**] `TraverseCtx::ancestor` with level 0 = equivalent to `parent` (#5294) (overlookmotel)

- 582ce9e traverse: [**BREAKING**] `TraverseCtx::ancestor` return `Ancestor::None` if out of bounds (#5286) (overlookmotel)

### Features

- f81e8a1 linter: Add `oxc/no-async-endpoint-handlers` (#5364) (DonIsaac)
- d04857b transformer: Support `Targets::from_query` method (#5336) (Dunqing)
- 3d4a64c transformer: Make `Targets` public (#5335) (Dunqing)
- 0eb7602 transformer: Support `TransformOptions::from_preset_env` API (#5323) (Dunqing)
- 08dc0ad transformer: Add `object-spread` plugin (#3133) (magic-akari)
- 01c0c3e transformer: Add remaining options to transformer options (#5169) (Boshen)
- 056c667 transformer/arrow-functions: The output that uses `this` inside blocks doesn't match Babel (#5188) (Dunqing)
- 0abfc50 transformer/typescript: Support `rewrite_import_extensions` option (#5399) (Dunqing)

### Bug Fixes

- 35f03db transformer: `ArrowfunctionExpression`'s expression is true but has more than one body statement (#5366) (Dunqing)
- 8d6b05c transformer: Class property with typescript value should not be removed (#5298) (Boshen)
- 47e69a8 transformer-optional-catch-binding: The `unused` binding is not in the correct scope (#5066) (Dunqing)
- 94ff94c transformer/arrow-functions: Reaches `unreachable` when `<this.foo>` is inside an arrow function (#5356) (Dunqing)
- f8bb022 transformer/arrow-functions: Remove `SymbolFlags::ArrowFunction` (#5190) (Dunqing)
- d9ba5ad transformer/arrow-functions: Correct scope for `_this` (#5189) (Dunqing)
- 3acb3f6 transformer/react: Mismatch output caused by incorrect transformation ordering (#5255) (Dunqing)
- 5754c89 transformer/typescript: Remove accessibility from `AccessorProperty` (#5292) (Dunqing)

### Performance

- a1523c6 transformer: Remove an allocation from arrow functions transform (#5412) (overlookmotel)

### Documentation

- 8334bd4 transformer: Add documentation for `Targets::get_targets` (#5337) (Dunqing)
- d51a954 transformer: Add documentation for arrow-functions plugin (#5186) (Dunqing)

### Refactor

- 960e1d5 ast: Rename `IdentifierReference::new_with_reference_id` (#5157) (overlookmotel)
- 0de844d transformer: Remove unnecessary code from JSX transform (#5339) (overlookmotel)
- 5136f01 transformer: Remove unnecessary type annotation (#5131) (overlookmotel)
- 260c9d2 transformer/es2015: Move all entry points to implementation of Traverse trait (#5187) (Dunqing)
- 1645115 transformer/object-reset-spread: Make plugin initialization unconditional (#5319) (Dunqing)
- d2666fe transformer/object-rest-spread: Move plugin-relates files to `object_rest_spread` mod (#5320) (Dunqing)
- 7e2a7af transformer/react: Remove `CalculateSignatureKey` implementation from refresh (#5289) (Dunqing)

## [0.25.0] - 2024-08-23

- 78f135d ast: [**BREAKING**] Remove `ReferenceFlag` from `IdentifierReference` (#5077) (Boshen)

- 5f4c9ab semantic: [**BREAKING**] Rename `SymbolTable::get_flag` to `get_flags` (#5030) (overlookmotel)

- 58bf215 semantic: [**BREAKING**] Rename `Reference::flag` and `flag_mut` methods to plural (#5025) (overlookmotel)

- c4c08a7 ast: [**BREAKING**] Rename `IdentifierReference::reference_flags` field (#5024) (overlookmotel)

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

- c30e2e9 semantic: [**BREAKING**] `Reference::flag` method return `ReferenceFlag` (#5019) (overlookmotel)

- f88970b ast: [**BREAKING**] Change order of fields in CallExpression (#4859) (Burlin)

### Features

- 4b49cf8 transformer: Always pass in symbols and scopes (#5087) (Boshen)
- f51d3f9 transformer/nullish-coalescing-operator: Handles nullish coalescing expression in the FormalParamter (#4975) (Dunqing)
- f794870 transformer/nullish-coalescing-operator: Generate the correct binding name (#4974) (Dunqing)
- 72ff2c6 transformer/nullish-coalescing-operator: Add comments in top of file (#4972) (Dunqing)

### Bug Fixes

- 6ffbd78 transformer: Remove an `AstBuilder::copy` call from TS namespace transform (#4987) (overlookmotel)
- a8dfdda transformer: Remove an `AstBuilder::copy` call from TS module transform (#4986) (overlookmotel)
- 1467eb3 transformer: Remove an `AstBuilder::copy` call from TS enum transform (#4985) (overlookmotel)
- 1365feb transformer: Remove an `AstBuilder::copy` call for TS `AssignmentTarget` transform (#4984) (overlookmotel)
- edacf93 transformer: Remove an `AstBuilder::copy` call (#4983) (overlookmotel)
- 3b35332 transformer/logical-assignment-operators: Fix semantic errors (#5047) (Dunqing)

### Documentation

- 178d1bd transformer: Add documentation for exponentiation-operator plugin (#5084) (Dunqing)
- d50eb72 transformer: Add documentation for `optional-catch-binding` plugin (#5064) (Dunqing)
- 4425b17 transformer: Add documentation for `logical-assignment-operators` plugin (#5012) (Dunqing)
- 1bd5853 transformer: Updated README re: order of methods (#4993) (overlookmotel)

### Refactor

- cca7440 ast: Replace `AstBuilder::move_statement_vec` with `move_vec` (#4988) (overlookmotel)
- 96422b6 ast: Make AstBuilder non-exhaustive (#4925) (DonIsaac)
- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)
- 8d15e65 transformer: Use `into_member_expression` (#5006) (overlookmotel)
- 4796ece transformer: TS annotations transform use `move_expression` (#4982) (overlookmotel)
- a9fcf29 transformer/es2016: Move all entry points to implementation of Traverse trait (#5085) (Dunqing)
- deda6ac transformer/es2019: Move all entry points to implementation of Traverse trait (#5065) (Dunqing)
- 9df2f80 transformer/es2020: Move all entry points to implementation of Traverse trait (#4973) (Dunqing)
- 3f9433c transformer/es2021: Move all entry points to implementation of Traverse trait (#5013) (Dunqing)
- c60a50d transformer/exponentiation-operator: Use built-in `ctx.clone_identifier_reference` (#5086) (Dunqing)
- bcc8da9 transformer/logical-assignment-operator: Use `ctx.clone_identifier_reference` (#5014) (Dunqing)
- 38d4434 transformer/nullish-coalescing-operator: Move internal methods to bottom of the file (#4996) (Dunqing)

## [0.24.3] - 2024-08-18

### Features

- d49fb16 oxc_codegen: Support generate range leading comments (#4898) (IWANABETHATGUY)
- f1fcdde transformer: Support react fast refresh (#4587) (Dunqing)
- 0d79122 transformer: Support logical-assignment-operators plugin (#4890) (Dunqing)
- ab1d08c transformer: Support `optional-catch-binding` plugin (#4885) (Dunqing)
- 69da9fd transformer: Support nullish-coalescing-operator plugin (#4884) (Dunqing)
- 3a66e58 transformer: Support exponentiation operator plugin (#4876) (Dunqing)
- f88cbcd transformer: Add `BoundIdentifier::new_uid_in_current_scope` method (#4903) (overlookmotel)
- 1e6d0fe transformer: Add methods to `BoundIdentifier` (#4897) (overlookmotel)

### Bug Fixes

- 2476dce transformer: Remove an `ast.copy` from `NullishCoalescingOperator` transform (#4913) (overlookmotel)
- 248a757 transformer/typescript: Typescript syntax within `SimpleAssignmentTarget` with `MemberExpressions` is not stripped (#4920) (Dunqing)

### Documentation

- 9c700ed transformer: Add README including style guide (#4899) (overlookmotel)

### Refactor

- 1eb59d2 ast, isolated_declarations, transformer: Mark `AstBuilder::copy` as an unsafe function (#4907) (overlookmotel)
- 452187a transformer: Rename `BoundIdentifier::new_uid_in_root_scope` (#4902) (overlookmotel)
- 707a01f transformer: Re-order `BoundIdentifier` methods (#4896) (overlookmotel)
- 117dff2 transformer: Improve comments for `BoundIdentifier` helper (#4895) (overlookmotel)

## [0.24.2] - 2024-08-12

### Bug Fixes

- 62f759c transformer/typescript: Generated assignment for constructor arguments with access modifiers should be injected to the top of the constructor (#4808) (Dunqing)

## [0.24.0] - 2024-08-08

- 75f2207 traverse: [**BREAKING**] Replace `find_scope` with `ancestor_scopes` returning iterator (#4693) (overlookmotel)

- 506709f traverse: [**BREAKING**] Replace `find_ancestor` with `ancestors` returning iterator (#4692) (overlookmotel)

### Bug Fixes

- 4797eaa transformer: Strip TS statements from for in/of statement bodies (#4686) (overlookmotel)
- 5327acd transformer/react: The `require` IdentifierReference does not have a `reference_id` (#4658) (Dunqing)
- 3987665 transformer/typescript: Incorrect enum-related `symbol_id`/`reference_id` (#4660) (Dunqing)
- 4efd54b transformer/typescript: Incorrect `SymbolFlags` for jsx imports (#4549) (Dunqing)

### Refactor

- 83546d3 traverse: Enter node before entering scope (#4684) (overlookmotel)

## [0.23.1] - 2024-08-06

### Bug Fixes

- 5327acd transformer/react: The `require` IdentifierReference does not have a `reference_id` (#4658) (Dunqing)
- 3987665 transformer/typescript: Incorrect enum-related `symbol_id`/`reference_id` (#4660) (Dunqing)
- 4efd54b transformer/typescript: Incorrect `SymbolFlags` for jsx imports (#4549) (Dunqing)

## [0.23.0] - 2024-08-01

### Bug Fixes

- d5c4b19 parser: Fix enum member parsing (#4543) (DonIsaac)

### Refactor

- 96602bf transformer/typescript: Determine whether to remove `ExportSpeicifer` by `ReferenceFlags` (#4513) (Dunqing)

## [0.22.1] - 2024-07-27

### Bug Fixes

- c04b9aa transformer: Add to `SymbolTable::declarations` for all symbols (#4460) (overlookmotel)
- ecdee88 transformer/typescript: Incorrect eliminate exports when the referenced symbol is both value and type (#4507) (Dunqing)

### Refactor

- f17254a semantic: Populate `declarations` field in `SymbolTable::create_symbol` (#4461) (overlookmotel)

## [0.22.0] - 2024-07-23

- 85a7cea semantic: [**BREAKING**] Remove name from `reference` (#4329) (Dunqing)

### Refactor


## [0.21.0] - 2024-07-18

### Features

- 7eb960d transformer: Decode xml character entity `&#xhhhh` and `&#nnnn;` (#4235) (Boshen)

### Refactor

- a197e01 transformer/typescript: Remove unnecessary code (#4321) (Dunqing)

## [0.20.0] - 2024-07-11

- 5731e39 ast: [**BREAKING**] Store span details inside comment struct (#4132) (Luca Bruno)

### Bug Fixes

- 48947a2 ast: Put `decorators` before everything else. (#4143) (rzvxa)

### Refactor


## [0.19.0] - 2024-07-09

- b936162 ast/ast_builder: [**BREAKING**] Shorter allocator utility method names. (#4122) (rzvxa)

### Refactor


## [0.18.0] - 2024-07-09

- d347aed ast: [**BREAKING**] Generate `ast_builder.rs`. (#3890) (rzvxa)

### Features


## [0.17.2] - 2024-07-08

### Bug Fixes

- 4413e2d transformer: Missing initializer for readonly consructor properties (#4103) (Don Isaac)

## [0.17.0] - 2024-07-05

### Bug Fixes

- aaac2d8 codegen: Preserve parentheses from AST instead calculating from  operator precedence (#4055) (Boshen)

## [0.16.3] - 2024-07-02

### Bug Fixes

- bdee156 transformer/typescript: `declare class` incorrectly preserved as runtime class (#3997) (Dunqing)
- a50ce3d transformer/typescript: Missing initializer for class constructor arguments with `private` and `protected` modifier (#3996) (Dunqing)

## [0.16.2] - 2024-06-30

### Performance

- 1eac3d2 semantic: Use `Atom<'a>` for `Reference`s (#3972) (Don Isaac)

### Refactor

- 5845057 transformer: Pass in symbols and scopes (#3978) (Boshen)

## [0.16.1] - 2024-06-29

### Refactor

- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.16.0] - 2024-06-26

- 1f85f1a ast: [**BREAKING**] Revert adding `span` field to the `BindingPattern` type. (#3899) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- cfcef24 ast: [**BREAKING**] Add `directives` field to `TSModuleBlock` (#3830) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- 5501d5c transformer/typescript: Transform `import {} from "mod"` to import `"mod"` (#3866) (Dunqing)

### Bug Fixes

- 08fcfb3 transformer: Fix spans and scopes in TS enum transform (#3911) (overlookmotel)
- 17ad8f7 transformer: Create new scopes for new blocks in TS transform (#3908) (overlookmotel)
- d76f34b transformer: TODO comments for missing scopes (#3837) (overlookmotel)
- e470731 transformer: TS transform handle when type exports first (#3833) (overlookmotel)
- d774e54 transformer: TS transform generate do not copy statements (#3832) (overlookmotel)
- ff1da27 transformer: Correct comment in example (#3831) (overlookmotel)
- 6dcc3f4 transformer: Fix TS annotation transform scopes (#3816) (overlookmotel)
- aea3e9a transformer: Correct spans for TS annotations transform (#3782) (overlookmotel)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 363d3d5 ast: Add span field to the `BindingPattern` type. (#3855) (rzvxa)
- 5ef28b7 transformer: Shorten code (#3912) (overlookmotel)
- d9f268d transformer: Shorten TS transform code (#3836) (overlookmotel)
- 21b0d01 transformer: Pass ref to function (#3781) (overlookmotel)
- 7c44703 transformer: Remove needless `pub` on TS enum transform methods (#3774) (overlookmotel)
- 22c56d7 transformer: Move TSImportEqualsDeclaration transform code (#3764) (overlookmotel)
- cd56aa9 transformer: Simplify TS export assignment transform (#3762) (overlookmotel)
- 512740d transformer: Move and simplify TS enum transform entry point (#3760) (overlookmotel)

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

### Features

- 750a534 coverage: Transformer idempotency test (#3691) (Boshen)
- 4f2db46 transformer-dts: `--isolatedDeclarations` dts transform (#3664) (Dunqing)

### Bug Fixes

- 59666e0 transformer: Do not rename accessible identifier references (#3623) (Dunqing)

## [0.14.0] - 2024-06-12

### Bug Fixes

- 35e267b transformer: Arrow function transform use UIDs for `_this` vars (#3634) (overlookmotel)
- 39bdebc transformer: Arrow func transform maintain scope ID (#3633) (overlookmotel)
- 5cb7e6a transformer: Arrow func transform use correct spans (#3630) (overlookmotel)
- 0c4ccb4 transformer: Arrow function transform alter `</this>` (#3627) (overlookmotel)
- 8d237c4 transformer: JSX source calculate correct column when Unicode chars (#3615) (overlookmotel)
- 9e8f4d6 transformer: Do not add `__source` for generated nodes (#3614) (overlookmotel)
- 0fb4c35 transformer: Use UID for JSX source filename var (#3612) (overlookmotel)

### Performance

- 3a59294 transformer: React display name transform reduce Atom allocations (#3616) (overlookmotel)
- f4c1389 transformer: Create `Vec` with capacity (#3613) (overlookmotel)

### Refactor

- 08f1010 ast: Make `AstBuilder` `Copy` (#3602) (overlookmotel)
- 89bcbd5 transformer: Move `BoundIdentifier` into helpers (#3610) (overlookmotel)
- 5793ff1 transformer: Replace `&’a Trivias` with `Rc<Trivias>` (#3580) (Dunqing)
- 509871f transformer: Comment for unimplemented `spec` option in arrow fns transform (#3618) (overlookmotel)
- 4b2e3a7 transformer: Fix indentation (#3617) (overlookmotel)
- 3467e3d transformer: Remove outdated comment (#3606) (overlookmotel)
- a799225 transformer: Flatten file structure for React transform (#3604) (overlookmotel)
- 70f31a8 transformer: Reduce branching in JSX transform (#3596) (overlookmotel)
- 3ae567d transformer: Remove dead code (#3588) (overlookmotel)
- 60cbdec traverse: `generate_uid_in_root_scope` method (#3611) (overlookmotel)

## [0.13.4] - 2024-06-07

### Features

- 646b993 coverage/transformer: Handle @jsx option (#3553) (Dunqing)
- a939ddd transformer/typescript: Remove more typescript ast nodes (#3563) (Dunqing)
- e8a20f8 transformer/typescript: Remove typescript ast nodes (#3559) (Dunqing)
- ee9a215 transformer/typescript: Handle namespace directive correctly (#3532) (Dunqing)

### Bug Fixes

- f6939cb transformer: Store `react_importer` in `Bindings` in JSX transform (#3551) (overlookmotel)
- 7982b93 transformer: Correct spans for JSX transform (#3549) (overlookmotel)
- c00598b transformer: JSX set `reference_id` on refs to imports (#3524) (overlookmotel)

### Performance

- 37cdc13 transformer: Faster checks if JSX plugin enabled (#3577) (overlookmotel)
- 9f467b8 transformer: Avoid fragment update where possible (#3535) (overlookmotel)
- ac394f0 transformer: JSX parse pragma only once (#3534) (overlookmotel)

### Refactor

- f2113ae transformer: Reduce cloning and referencing `Rc`s (#3576) (overlookmotel)
- 0948124 transformer: Pass `Rc`s by value (#3550) (overlookmotel)
- e4d74ac transformer: Remove `update_fragment` from JSX transform (#3541) (overlookmotel)
- 73b7864 transformer: Combine import and usage in JSX transform (#3540) (overlookmotel)
- 6978269 transformer/typescript: Replace reference collector with symbols references (#3533) (Dunqing)

## [0.13.3] - 2024-06-04

### Bug Fixes

- 591c54b transformer: JSX set `symbol_id` on imports (#3523) (overlookmotel)
- 837776e transformer: TS namespace transform do not track var decl names (#3501) (overlookmotel)
- 8d2beff transformer: Use correct scope for TS namespaces (#3489) (overlookmotel)
- 8e4f335 transformer: Output empty file for TS definition files (#3500) (overlookmotel)

### Performance

- 91519d9 transformer: React JSX reduce allocations (#3522) (overlookmotel)
- f3a755c transformer: React JSX reuse same `Atom`s (#3521) (overlookmotel)

### Refactor

- 7bbd3da traverse: `generate_uid` return `SymbolId` (#3520) (overlookmotel)

## [0.13.2] - 2024-06-03

### Features

- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)
- 574629e tasks/coverage: Turn on idempotency testing for transformer (#3470) (Dunqing)
- 816a782 transformer: Support `targets` option of preset-env (#3371) (Dunqing)
- 92df98b transformer/typescript: Report error that do not allow namespaces (#3448) (Dunqing)
- a6b073a transformer/typescript: Report error for namespace exporting non-const (#3447) (Dunqing)
- 150255c transformer/typescript: If within a block scope, use let to declare enum name (#3446) (Dunqing)
- e80552c transformer/typescript: If binding exists, variable declarations are not created for namespace name (#3445) (Dunqing)
- 241e8d1 transformer/typescript: If the binding exists, the identifier reference is not renamed (#3387) (Dunqing)

### Bug Fixes

- 90b0f6d transformer: Use UIDs for React imports (#3431) (overlookmotel)
- d4371e8 transformer: Use UIDs in TS namespace transforms (#3395) (overlookmotel)
- baed1ca transformer/jsx-source: Add filename statement only after inserting the source object (#3469) (Dunqing)
- b4fd1ad transformer/typescript: Variable declarations are not created when a function has a binding with the same name (#3460) (Dunqing)

### Refactor

- 55bbde2 ast: Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488) (overlookmotel)
- 84feceb transformer: Explicit skip TS statements in TS namespace transform (#3479) (overlookmotel)
- 7f7b5ea transformer: Shorter code in TS namespace transform (#3478) (overlookmotel)
- 7e7b452 transformer: Panic on illegal cases in TS namespace transform (#3477) (overlookmotel)
- 8e089a9 transformer: Rename var (#3476) (overlookmotel)
- 0f69ffd transformer: Shorten code in TS namespace transform (#3468) (overlookmotel)
- deef86a transformer: Remove unreachable code from TS namespace transform (#3475) (overlookmotel)
- 9dc58d5 transformer/typescript: Use a memory-safe implementation instead (#3481) (Dunqing)
- 1a50b86 typescript/namespace: Reuse TSModuleBlock's scope id (#3459) (Dunqing)

## [0.13.1] - 2024-05-22

### Features

- e2c6fe0 transformer: Report errors when options have unknown fields (#3322) (Dunqing)
- 9ee962a transformer: Support `from_babel_options` in TransformOptions (#3301) (Dunqing)
- b9d69ad transformer: Do not add self attribute in react/jsx plugin (#3287) (Dunqing)
- 421107a traverse: Pass `&mut TraverseCtx` to visitors (#3312) (overlookmotel)

### Bug Fixes

- b4fa27a transformer: Do no add __self when the jsx is inside constructor (#3258) (Dunqing)

### Refactor

- c9d84af diagnostics: S/warning/warn (Boshen)
- e7a6595 transformer: Correct spelling of var name (#3369) (overlookmotel)
- dad47a5 transformer: Improve indentation (#3282) (overlookmotel)
- 05c71d2 traverse: `Traverse` produce scopes tree using `Semantic` (#3304) (overlookmotel)

## [0.13.0] - 2024-05-14

### Features

- f1ccbd4 syntax: Add `ToInt32` trait for f64 (#3132) (Boshen)
- 870d11f syntax: Add `ToJsString` trait for f64 (#3131) (Boshen)
- 34dd53c transformer: Report ambient module cannot be nested error (#3253) (Dunqing)
- 1b29e63 transformer: Do not elide jsx imports if a jsx element appears somewhere (#3237) (Dunqing)
- 905ee3f transformer: Add arrow-functions plugin (#3083) (Dunqing)
- 78875b7 transformer: Implement typescript namespace (#3025) (Boshen)
- a52e321 transformer/jsx-source: Get the correct lineNumber and columnNumber from the span. (#3142) (Dunqing)
- 18d853b transformer/react: Support development mode (#3143) (Dunqing)
- be8fabe transformer/react: Enable jsx plugin when development is true (#3141) (Dunqing)

### Bug Fixes

- 9590eb0 transform: Implement `transform-react-display-name` with bottom-up lookup (#3183) (overlookmotel)
- 6ac8a84 transformer: Correctly jsx-self inside arrow-function (#3224) (Dunqing)
- b589496 transformer/arrow-functions: Should not transform `this` in class (#3129) (Dunqing)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- be958ce transform: Transformer use `Traverse` (#3182) (overlookmotel)
- 7067f9c transformer: Clean up more diagnostics (Boshen)
- d351f2d transformer: Unify diagnostics (Boshen)
- 9525653 transformer: Remove no-op scopes code (#3210) (overlookmotel)
- a63a45d transformer: Remove the requirement of `Semantic` (#3140) (Boshen)
- 843318c transformer/typescript: Reimplementation of Enum conversion based on Babel (#3102) (Dunqing)- d8173e1 Remove all usages of `Into<Error>` (Boshen)

## [0.12.5] - 2024-04-22

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)
- 383b449 ast: Box `ImportDeclarationSpecifier` enum variants (#3061) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- b6b63ac transform_conformance: Skip tests with plugin.js (#2978) (Boshen)
- ef602af transform_conformance: Skip plugins we don't support yet (#2967) (Boshen)
- 85a3653 transformer: Add "_jsxFileName" variable in jsx source plugin (#3000) (Boshen)
- e43c245 transformer: Add import helpers to manage module imports (#2996) (Boshen)
- c211f1e transformer: Add diagnostics to react transform (#2974) (Boshen)
- 3a6eae1 transformer: Apply jsx self and source plugin inside jsx transform (#2966) (Boshen)
- bd9fc6d transformer: React jsx transform (#2961) (Boshen)
- e673550 transformer: Start on TypeScript annotation removal (#2951) (Miles Johnson)
- e651e50 transformer: Add the most basic plugin toggles (#2950) (Boshen)
- 1475477 transformer: Implement react-jsx-source (#2948) (Boshen)
- f903a22 transformer: Implement react-jsx-self (#2946) (Boshen)
- 0c04bf7 transformer: Transform TypeScript namespace (#2942) (Boshen)
- 3419306 transformer: Add filename (#2941) (Boshen)
- b72bdca transformer/react: Reports duplicate __self/__source prop error (#3009) (Dunqing)
- 3831147 transformer/typescript: Report error for export = <value> (#3021) (Dunqing)
- 7416de2 transformer/typescript: Reports error for import lib = require(...); (#3020) (Dunqing)
- e14ac17 transformer/typescript: Insert this assignment after the super call (#3018) (Dunqing)
- afb1dd4 transformer/typescript: Support for transform TSImportEqualsDeclaration (#2998) (Dunqing)
- 6732e8b transformer/typescript: Support for transform enum (#2997) (Dunqing)
- 6a53fa3 transformer/typescript: Correct elide imports/exports statements (#2995) (Dunqing)

### Bug Fixes

- 722d4c2 transformer: `TypeScriptOptions` deserialize should fallback to default (#3012) (Boshen)
- 6704546 transformer: React `development` default value should be false (#3002) (Boshen)
- c7e70c8 transformer: Deserialize ReactJsxRuntime with camelCase (#2972) (Boshen)
- 10814d5 transformer: Turn on react preset by default (#2968) (Boshen)
- 35e3b0f transformer: Fix incorrect jsx whitespace text handling (#2969) (Boshen)
- 99e038c transformer/typescript: Modifiers should not be removed (#3005) (Dunqing)

### Refactor

- 82e00bc transformer: Remove boilerplate code around decorators to reduce noise (#2991) (Boshen)
- 60ccbb1 transformer: Clean up some code (#2949) (Boshen)

## [0.12.3] - 2024-04-11

### Features

- 02adc76 transformer: Implement plugin-transform-react-display-name top-down (#2937) (Boshen)
- 255c74c transformer: Add transform context to all plugins (#2931) (Boshen)
- 79ca6fe transformer: Add transform callback methods (#2929) (Boshen)
- d65eab3 transformer: Add react preset (#2921) (Boshen)

## [0.12.1] - 2024-04-03

### Features

- 7710d8c transformer: Add compiler assumptions (#2872) (Boshen)
- 7034bcc transformer: Add proposal-decorators (#2868) (Boshen)
- ffadcb0 transformer: Add react plugins (#2867) (Boshen)
- 293b9f4 transformer: Add `transform-typescript` boilerplate (#2866) (Boshen)

### Bug Fixes

- 21a5e44 transformer: Add serde "derive" feature to fix compile error (Boshen)

## [0.11.0] - 2024-03-30

### Features

- 243131d transformer: Numeric separator plugin. (#2795) (Ali Rezvani)
- 56493bd transformer: Add transform literal for numeric literals. (#2797) (Ali Rezvani)
- 398a034 transformer/typescript: Remove `verbatim_module_syntax` option (#2796) (Dunqing)

### Bug Fixes

- b76b02d parser: Add support for empty module declaration (#2834) (Ali Rezvani)
- 528744c transformer: Optional-catch-binding unused variable side effect (#2822) (Ali Rezvani)

### Refactor

- fc38783 ast: Add walk_mut functions (#2776) (Ali Rezvani)
- 813226b ast: Get rid of unsafe transmutation in VisitMut trait. (#2764) (Ali Rezvani)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)
- fe12617 transformer: Pass options via context. (#2794) (Ali Rezvani)

## [0.10.0] - 2024-03-14

### Features

- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)
- 308b780 transformer/decorators: Handling the coexistence of class decorators and member decorators (#2636) (Dunqing)

### Bug Fixes

- 2a235d3 ast: Parse `with_clause` in re-export declaration (#2634) (magic-akari)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Features

- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- f760108 transformer: Call build module record (#2529) (Dunqing)
- 6d43e85 transformer/typescript: Support transform constructor method (#2551) (Dunqing)

### Bug Fixes

- 258b9b1 ast: Support FormalParameter.override (#2577) (Arnaud Barré)
- 7a12514 transformer/decorators: Missing check private function (#2607) (Dunqing)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 2c2256a transformer/typescript: Improve implementation of remove import/export (#2530) (Dunqing)

## [0.8.0] - 2024-02-26

### Features

- 70295a5 ast: Update arrow_expression to arrow_function_expression (#2496) (Dunqing)
- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)
- cd75c1c transformer/decorators: Insert only one private in expression (#2486) (Dunqing)
- 3d008ab transformer/decorators: Insert instanceBrand function (#2480) (Dunqing)
- 2628c97 transformer/decorators: Transform getter function (#2473) (Dunqing)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)
- 27b2c21 transformer/decorators: If it is a private method definition, transform it (#2427) (Dunqing)
- 4b11183 transformer/decorators: Move get_decorator_info inside the decorators (#2426) (Dunqing)

## [0.6.0] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- 165f948 ast: Remove expression property from Function (#2247) (Dunqing)
- 9e598ff transformer: Add decorators plugin (#2139) (Dunqing)
- 02c18d8 transformer/decorators: Support for static and private member decorators (#2246) (Dunqing)
- ba85b09 transformer/decorators: Support method decorator and is not static (#2238) (Dunqing)
- a79988d transformer/decorators: Support static member (#2235) (Dunqing)
- 3b85e18 transformer/decorators: Ensure property key consistency (#2233) (Dunqing)
- e5719e9 transformer/decorators: Support transform member decorators (#2171) (Dunqing)
- 7f89bfe transformer/decorators: Support version 2023-05 (#2152) (Dunqing)
- 04b401c transformer/decorators: Support transform the class decorators in export declaration (#2145) (Dunqing)
- b5b2ef3 transformer/typescript: Improve function parameters name (#2079) (Dunqing)
- 7711f7a transformer/typescript: Support only_remove_type_imports option (#2077) (Dunqing)
- f5bf5de transformer/typescript: Support transform exported TSModuleBlock (#2076) (Dunqing)
- 56ca8b6 transformer/typescript: Support transform namespace (#2075) (Dunqing)
- b89e84c transformer/typescript: Keep imports if import specifiers is empty (#2058) (Dunqing)
- 3413bb3 transformer/typescript: Remove type-related exports (#2056) (Dunqing)
- 95d741a transformer/typescript: Remove type only imports/exports correctly (#2055) (Dunqing)
- 6c7f983 transformer/typescript: Remove export specifier that import_kind is type (#2015) (Dunqing)
- ead4e8d transformer/typescript: Remove import if only have type reference (#2001) (Dunqing)
- 2794064 transfrom: Transform-json-strings (#2168) (underfin)

### Bug Fixes

- 777352e transformer: Always create valid identifiers (#2131) (overlookmotel)

### Refactor

- b261e86 ast: Improve simple_assignment_target_identifier and simple_assignment_target_member_expression method (#2153) (Dunqing)
- ee949fc transformer: Use `is_identifier_part` (overlookmotel)
- 040ee19 transformer: Use `is_identifier_name` from `oxc_syntax` (overlookmotel)
- de6d2f5 transformer/decorators: Optimizing code with ast.private_field (#2249) (Dunqing)
- 51cecbb transformer/decorators: Align the implementation of all versions (#2159) (Dunqing)
- 2e78b91 transformer/typescript: Move the ExportNamedDeclaration logic to its function (#2074) (Dunqing)

## [0.5.0] - 2024-01-12

### Features

- 78b427b transform: Support es2015 new target (#1967) (underfin)
- 6a7e4be transformer: Call enter_node/leave_node in visit_xxx (#1990) (Dunqing)
- afb2c50 transformer: Support for transform TSImportEqualsDeclaration (#1994) (Dunqing)
- ae27a8d transformer: Add partial support for babel-plugin-transform-instanceof (#1802) (秦宇航)
- f58b627 transformer: Add arrow_functions plugin (#1663) (Dunqing)
- e331cc2 transformer: Duplicate keys (#1649) (Ken-HH24)
- 864176a transformer/react-jsx: Returns ThisExpression when identifier is this (#1661) (Dunqing)

### Refactor

- a2858ed ast: Introduce `ThisParameter` (#1728) (magic-akari)

## [0.4.0] - 2023-12-08

### Features

- c6ad660 semantic: Support scope descendents starting from a certain scope. (#1629) (Miles Johnson)
- 92c1d9d transform: TypeScript Enum (#1173) (magic-akari)
- 6cbc5dd transformer: Start on `function_name` transform. (#1510) (Miles Johnson)
- c034eee transformer: Handle invalid react jsx  runtime (#1502) (IWANABETHATGUY)
- f66e4d8 transformer: Add transform property-literal plugin (#1458) (IWANABETHATGUY)
- f0e452a transformer: Support importSource option in react_jsx (#1115) (Dunqing)
- b6393f0 transformer/react: Handle babel 8 breaking removed-options (#1489) (IWANABETHATGUY)
- 7f01d48 transformer/react-jsx: Set `automatic` to the default value for `runtime` (#1270) (Dunqing)
- 1eef241 transformer/react-jsx: Support for throwing SpreadChildrenAreNotSupported error (#1234) (Dunqing)
- 39e6087 transformer/react-jsx: Support for throwing ImportSourceCannotBeSet error (#1224) (Dunqing)
- b7e8feb transformer/react-jsx: Support throw valueless-key error (#1221) (Dunqing)
- a22ced7 transformer/react-jsx: Implement `throwIfNamespace` option (#1220) (Dunqing)
- d9b4504 transformer/react-jsx: When the source type is a script, use require to import the react (#1207) (Dunqing)
- 8c624ab transformer/react-jsx: Throw the `pragma and pragmaFrag cannot be set when runtime is automatic` error (#1196) (Dunqing)
- 7d85492 transformer/react-jsx: Support the `sourceType` is a `script` (#1192) (Dunqing)
- 28c0b85 transformer/react-jsx: Support `@jsxFrag` annotation (#1189) (Dunqing)
- 633c469 transformer/react-jsx: Support `@jsx` annotation (#1182) (Dunqing)
- 3cb7c0b transformer/react-jsx: Support `pragmaFrag` option (#1181) (Dunqing)
- 4ed0813 transformer/react-jsx: Support `pragma` option (#1180) (Dunqing)
- bf23d87 transformer/react-jsx: Support `@jsxImportSource` annotation (#1179) (Dunqing)

### Bug Fixes

- 4824236 transformer/react-jsx: Missing import jsxs in nested fragment (#1218) (Dunqing)
- a0f40cb transformer/react-jsx: Missing default options when plugin without config (#1219) (Dunqing)
- 3e15fa6 transformer/react-jsx: Undetectable comments in multiline comments (#1211) (Dunqing)
- b65094b transformer/react-jsx: No need to wrap the Array when there is only one correct child element (#1205) (Dunqing)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)
- d62631e transformer/react-jsx: Use extend instead of for-in with push (#1236) (Dunqing)
- 47ba874 transformer/react-jsx: Improve SpreadChildrenAreNotSupported error implementation (#1235) (Dunqing)

## [0.3.0] - 2023-11-06

### Features

- e0ca09b codegen: Implement the basics of non-minifying codegen (#987) (Boshen)
- 2e2b758 playground: Add transform and minify (#993) (Boshen)
- f60fd65 transfomer: Implement react has_key_after_props_spread (#1075) (Boshen)
- f71cb9f transform: Support TemplateLiteral of babel/plugin-transform-template-literals (#1132) (Wenzhe Wang)
- b5bfc36 transform: Transform jsx element name (#1070) (Wenzhe Wang)
- 09df8e6 transform: Sticky-regex (#968) (Wenzhe Wang)
- ce79bc1 transform_conformance: Move Formatter to codegen (#986) (Boshen)
- 46d2623 transform_conformance: Add jsx and ts tests (Boshen)
- e8a4e81 transformer: Implement some of jsx decode entities (#1086) (Boshen)
- 0856111 transformer: Implement more of react transform attributes (#1081) (Boshen)
- 96332c8 transformer: Import jsxs when children is static (#1080) (Boshen)
- d411258 transformer: Finish transform jsx attribute value (#1078) (Boshen)
- 5fb27fb transformer: Implement key extraction for react automatic (#1077) (Boshen)
- 394ed35 transformer: Implement react get_attribute_name (#1076) (Boshen)
- d6ba891 transformer: Add props `null` to React.createElement (#1074) (Boshen)
- e16e7e4 transformer: Implement react transform attributes (#1071) (Boshen)
- d8f1a7f transformer: Start implementing react jsx transform (#1057) (Boshen)
- 1b64e48 transformer: Strip implicit type import for typescript (#1058) (magic-akari)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- d31a667 transformer: Drop `this` parameter from typescript functions (#1019) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- c060621 transformer: Add unit tests and test coverage (#1001) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)
- dc08c94 transformer: RegexpFlags (#977) (magic-akari)
- 9ad2634 transformer: Class Static Block (#962) (magic-akari)
- 21066a9 transformer: Shorthand Properties (#960) (magic-akari)
- 5973e5a transformer: Setup typescript and react transformers (#930) (Boshen)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)
- f4cea34 transformer: Add babel conformance test suite (#920) (Boshen)
- 419d5aa transformer: Transformer prototype (#918) (Boshen)
- 1051f15 transformer/jsx: Escape xhtml in jsx attributes (#1088) (Boshen)
- 203cf37 transformer/react: Read comment pragma @jsxRuntime classic / automatic (#1133) (Boshen)
- 262631d transformer/react: Implement fixup_whitespace_and_decode_entities (#1091) (Boshen)
- 1b3b100 transformer_conformance: Read plugins options from babel `options.json` (#1006) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)
- fe4a5ed transformer: Fix position of inserted react import statement (#1082) (Boshen)
- 1ad2dca transformer/react_jsx: Add imports to the top body (#1087) (Boshen)

### Refactor

- 4787220 ast: Clean up some methods (Boshen)
- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- 70189f9 ast: Change the arguments order for some `new` functions (Boshen)
- 801d78a minifier: Make the minifier api only accept an ast (#990) (Boshen)
- 052661d transform_conformance: Improve report format (Boshen)
- 69150d8 transformer: Move Semantic into Transformer (#1130) (Boshen)
- c7a04f4 transformer: Remove returning None from transform functions (#1079) (Boshen)
- d9ba532 transformer: Add an empty SPAN utility for creating AST nodes (#1067) (Boshen)
- 46a5c42 transformer: Add TransformerCtx struct for easier access to symbols and scopes (Boshen)
- 1e1050f transformer: Clean up the transformer constructor code (Boshen)

