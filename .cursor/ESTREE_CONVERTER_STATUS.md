# ESTree Converter Implementation Status

## Current Work
Implementing missing features in the ESTree to oxc AST converter (`crates/oxc_linter/src/estree_converter.rs`).

## Last Completed
- Added `type_annotation` and `accessibility` support for `AccessorProperty`
- Added `accessibility` support for `FormalParameter`
- Implemented `WithClause` conversion for `ImportDeclaration` (import attributes/assertions)
- Implemented `TSModuleDeclaration` body conversion (TSModuleBlock and nested declarations)
- Implemented `TSImportEqualsDeclaration` `moduleReference` TSTypeName variants (IdentifierReference, QualifiedName, ThisExpression)
- Implemented directive conversion for `TSModuleBlock` (added `convert_directive` helper function)
- Added support for getter and setter properties in object literals (Property kind 'get' and 'set')
- Added test for getter/setter properties (test_object_expression_with_getter_setter)
- Added BigInt literal support in `convert_literal_to_expression` (all bases: decimal, hex, octal, binary)
- Fixed `convert_literal` to check for BigInt before String (BigInt is string ending with 'n')
- Fixed duplicate match arms in EstreeNodeType (ReturnStatement, NewExpression, ThisExpression)
- Added test for BigInt literals (test_bigint_literal)
- Added RegExp literal support in `convert_literal_to_expression` (pattern and flags parsing)
- Fixed RegExp detection to check top-level `regex` property (not in value)
- Added test for RegExp literals (test_regexp_literal)

## Current Issue
✅ **RESOLVED** - Fixed duplicate `type_annotation` definition in `convert_accessor_property` function and type mismatch in `convert_ts_module_declaration`.
✅ **RESOLVED** - Implemented directive conversion for TSModuleBlock.

## Files Modified
- `crates/oxc_linter/src/estree_converter.rs` - Main converter implementation

## Key Functions Modified/Added
- `convert_accessor_property` - Added type_annotation and accessibility (fixed duplicate type_annotation definition)
- `convert_import_declaration` - Added WithClause conversion
- `convert_import_attribute` - Already exists, used by WithClause conversion
- `convert_ts_module_declaration` - Added body conversion (fixed type mismatch in body array handling)
- `convert_ts_import_equals_declaration` - Added TSTypeName variant support for moduleReference
- `convert_directive` - New helper function to convert ESTree directives (ExpressionStatement with StringLiteral)
- `convert_object_property` - Added support for getter/setter properties (PropertyKind::Get, PropertyKind::Set)
- `convert_literal_to_expression` - Added BigInt and RegExp literal support
- `convert_literal` (oxc_estree) - Fixed to check for BigInt before String, removed RegExp detection (handled at caller level)

## Completed Steps
1. ✅ **Fixed compilation errors**:
   - Removed duplicate `type_annotation` definition in `convert_accessor_property` (was using wrong context "PropertyDefinition" instead of "AccessorProperty")
   - Fixed type mismatch in `convert_ts_module_declaration` body array handling (changed from `unwrap_or_else(|| &[])` to proper `if let` pattern)
2. ✅ **Verified compilation**: `cargo check -p oxc_linter` passes
3. ✅ **Ran tests**: All 64 tests in `estree_converter_test` pass
4. ⏳ **Ready for commit**: All changes are complete and tested

## Context
This is part of the ESLint custom parser support implementation. The converter transforms ESTree AST nodes (from JavaScript parsers like espree, @typescript-eslint/parser) into oxc's native AST format. The missing features being implemented are TypeScript-specific fields and import attributes support.

