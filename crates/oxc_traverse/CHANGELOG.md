# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

