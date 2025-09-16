# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

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



