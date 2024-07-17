# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.21.0] - 2024-07-17

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

