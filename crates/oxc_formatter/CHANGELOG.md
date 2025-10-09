# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.4.0] - 2025-10-09

### 🚀 Features

- 142e7ac formatter/sort-imports: Implement options.ignoreCase: bool (#14367) (leaysgur)
- 5c8bd31 formatter/sort-imports: Implement options.sortSideEffects: bool (#14293) (leaysgur)
- 593c416 formatter/sort-imports: Add options.order: asc|desc (#14292) (leaysgur)
- f1a1f89 formatter/sort-imports: Implement basic sorting with tests (#14291) (leaysgur)
- f75b8f7 formatter/sort-imports: Wrap `ImportDeclaration` with `JsLabels` (#14109) (leaysgur)
- 6be4ae5 formatter/sort-imports: Experimental sort-imports base (#14105) (leaysgur)
- cb29117 formatter: Correct printing parameters with `return_type` for function-like node (#14084) (Dunqing)
- 90fd46f formatter: Normalize key of `TSPropertySignature` (#14083) (Dunqing)
- 6cfce80 formatter: Implement formatting for `TSTypeAliasDeclaration` (#14040) (Dunqing)
- 3097b60 formatter: Implement formatting for `TSMappedType` (#14025) (Dunqing)
- cd620bd formatter: Correct printing for `Class` (#14024) (Dunqing)
- 03244f1 formatter: Correct printing for `TSConditionalType` (#14023) (Dunqing)
- f6dc981 formatter: Implement formatting for `TSTupletype` (#14019) (Dunqing)
- 10a41ab formatter: Export doc() API to inspect IR in example (#14068) (leaysgur)
- 06a1df6 formatter: Implement formatting for `TSTypeParameters` and `TSTypeParameterInstantiation` (#13919) (Dunqing)
- 9b46dd7 formatter: Implement formatting for `TSTypeAssertion` (#13911) (Dunqing)
- 5710b13 formatter: Implement formatting for `TSIntersectiontype` (#13910) (Dunqing)
- 2d18144 formatter: Implement formatting for `TSUnionType` (#13893) (Dunqing)
- 0f15ed3 formatter: Implement formatting for `TSAsExpression` and `TSSatisfiesExpression` (#13892) (Dunqing)

### 🐛 Bug Fixes

- ad5c18a formatter: Correct parentheses in `TSIntersectionType` (#14098) (Noel Kim (김민혁))
- 7c09b20 formatter: Print comments incorrectly if the node is without following a node (#14110) (Dunqing)
- ed33fad formatter: Merge the right side of `LogicalExpression` if it's a `LogicalExpression` and both have the same `operator` (#14097) (Dunqing)
- 1b0519c formatter: Correct printing comments within the type annotation of ArrayPattern and `ObjectPattern` (#14077) (Dunqing)
- e299ab0 formatter: Correct printing comments around decorators (#14076) (Dunqing)
- 7d11047 formatter: Correct a bunch of implementations for TypeScript (#14069) (Dunqing)
- 57cbf84 formatter: Correct preserving parentheses for `TSXXXType` nodes (#14022) (Dunqing)
- 134f255 formatter: Missing parenthesis for `NewExpression` whose callee is a `TSNonNullExpression` with `TaggedTemplateExpression` (#14021) (Dunqing)
- 1e9ce4e formatter: Skip the parent node if it is a `TSNonNullExpression` or `AstNodes::ChainExpression` for `StaticMemberExpression` (#14020) (Dunqing)
- 3ce0775 formatter: Missing semicolon for `declare` function (#13928) (Dunqing)

### 🚜 Refactor

- 70bd141 formatter: Improve formatting of `Function` and `ArrowFunctionExpression` with types (#14070) (Dunqing)


## [0.3.0] - 2025-09-19

### 🚀 Features

- 2cead8b formatter: Keep parser options consistent for all formatter usages (#13884) (Dunqing)

### 🐛 Bug Fixes

- c96f7e9 formatter: Add parentheses for `await` and `yield` inside `PrivateInExpression` (#13863) (Noel Kim (김민혁))
- eae4845 formatter: Add parentheses for mixed types (#13862) (Noel Kim (김민혁))
- 57108c0 formatter: Keep computed name in enum (#13848) (Noel Kim (김민혁))
- 5c3645b formatter: Handle decorators correctly for class expressions in export (#13845) (Dunqing)
- 3cf1a41 formatter: Missing parenthesis for `TSAsExpression` (#13842) (Dunqing)
- 25edd03 formatter: Missing parenthesis for `TSTypeAssertion` (#13841) (Dunqing)
- 72144e9 formatter: Missing trailing semicolon in `TSSignature` (#13823) (Dunqing)
- f643093 formatter: Missing parenthesis for expression of `decorator` (#13813) (Dunqing)
- b43ad49 formatter: Add parentheses for `PrivateInExpression` in super class (#13806) (Noel Kim (김민혁))
- 7879f85 formatter: Add parentheses inside `UpdateExpression` (#13825) (Noel Kim (김민혁))
- 7371bad formatter: Add parentheses inside `TSIntersectionType` (#13821) (Noel Kim (김민혁))


## [0.2.0] - 2025-09-16

### 🚀 Features

- 7cbd06e formatter: Support `TSTypePredicate` (#13742) (Sysix)

### 🐛 Bug Fixes

- 9882dce formatter: Add parentheses for `TSFunctionType` and `TSConstructorType` inside `TSConditionalType` (#13804) (Noel Kim (김민혁))
- f56c8a3 formatter: Add parentheses for nested `TSConditionalType` (#13800) (Noel Kim (김민혁))
- a1ad9c5 formatter: Add parentheses for `TSUnionType` inside `TSArrayType` (#13792) (Sysix)
- 34e7000 formatter: Add parentheses for `TSConstructorType` inside `TSUnionType` (#13791) (Sysix)
- d515114 formatter: Add `declare` for `FunctionDeclaration` (#13790) (Sysix)
- 8659498 formatter: Should parenthesize `TSInferType` when wrapped with `TSArrayType` (#13756) (Noel Kim (김민혁))
- 0b48186 formatter: Add space after `readonly` in `TSPropertySignature` (#13747) (Sysix)
- 52d365b formatter: Add `declare` for `VariableDeclaration` (#13749) (Sysix)
- 0b047e8 formatter: Add parentheses for `TSFunctionType` inside `TSUnionType` (#13746) (Sysix)
- f5f37c4 formatter: Add space after `extends` in `TSInterfaceDeclaration` (#13741) (Sysix)


## [0.1.0] - 2025-09-12

### 🚀 Features

- 265d6a6 formatter: Support `TSEnumDeclaration` (#13704) (leaysgur)
- 34b7255 formatter: Consolidate comments checking API (#13656) (Dunqing)
- 8c072dc formatter: Print type cast comments (#13597) (Dunqing)

### 🐛 Bug Fixes

- bda5fc1 formatter: Correct comments printing for import and export (#13707) (Dunqing)
- 966e395 formatter: Incorrectly wrap a parenthesis for `ArrowFunctionExpression` when it has a leading type cast comment (#13683) (Dunqing)
- 239d4cb formatter: Improve AssignmentExpression parentheses handling (#13668) (leaysgur)

### 🚜 Refactor

- d7ff3d9 formatter: Introduce `SourceText` with utility methods (#13650) (Dunqing)
- 6b74078 formatter: Move `is_supported_source_type` to `oxc_formatter` crate (#13702) (Sysix)



