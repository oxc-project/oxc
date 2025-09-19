# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

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



