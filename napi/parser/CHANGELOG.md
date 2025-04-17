# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)

### Bug Fixes

- f3ddefb ast/estree: Add missing fields to `AssignmentTargetRest` in TS-ESTree AST (#10456) (overlookmotel)
- 77b6f7e ast/estree: Fix start span of `Program` in TS-ESTree AST where first statement is `@dec export class C {}` (#10448) (overlookmotel)
- 4817c7e ast/estree: Add fields to `AssignmentTargetPattern` in TS-ESTree AST (#10423) (overlookmotel)
- b3094b3 ast/estree: Add `optional` field to `AssignmentTargetProperty` in TS-ESTree AST (#10412) (overlookmotel)
- a7fd30f ast/estree: Add fields to `BindingRestElement` in TS-ESTree AST (#10411) (overlookmotel)
- cc07efd ast/estree: Fix `JSXOpeningFragment` (#10208) (therewillbecode)
- 48ed6a1 ast/estree: Fix span for `TemplateElement` in TS AST (#10315) (overlookmotel)
- 2520b25 estree: Align `TSMappedType` fields (#10392) (Yuji Sugiura)
- 3ed3669 estree: Rename `JSDocXxxType` to `TSJSDocXxxType` (#10358) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)

### Bug Fixes

- f3ddefb ast/estree: Add missing fields to `AssignmentTargetRest` in TS-ESTree AST (#10456) (overlookmotel)
- 77b6f7e ast/estree: Fix start span of `Program` in TS-ESTree AST where first statement is `@dec export class C {}` (#10448) (overlookmotel)
- 4817c7e ast/estree: Add fields to `AssignmentTargetPattern` in TS-ESTree AST (#10423) (overlookmotel)
- b3094b3 ast/estree: Add `optional` field to `AssignmentTargetProperty` in TS-ESTree AST (#10412) (overlookmotel)
- a7fd30f ast/estree: Add fields to `BindingRestElement` in TS-ESTree AST (#10411) (overlookmotel)
- cc07efd ast/estree: Fix `JSXOpeningFragment` (#10208) (therewillbecode)
- 48ed6a1 ast/estree: Fix span for `TemplateElement` in TS AST (#10315) (overlookmotel)
- 2520b25 estree: Align `TSMappedType` fields (#10392) (Yuji Sugiura)
- 3ed3669 estree: Rename `JSDocXxxType` to `TSJSDocXxxType` (#10358) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- e42c040 ast/estree: Add TS fields to `LabelIdentifier` (#10295) (overlookmotel)
- 06fc07c ast/estree: Fix `TSImportType` (#10200) (therewillbecode)
- 760188e ast/estree: Fix `BindingProperty` (#10193) (therewillbecode)
- f547d76 ast/estree: Add `TSEnumBody` to `TSEnumDeclaration.body` (#10017) (Yuji Sugiura)
- 34d5c00 ast/estree: Fix `ExportDefaultDeclaration` node (#10165) (therewillbecode)
- 498b479 ast/estree: Fix `AccessorProperty` node (#10067) (therewillbecode)
- bf90072 ast/estree: Fix `ObjectProperty` node (#10018) (therewillbecode)
- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)
- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)
- 52f2a40 span/estree: Skip `ModuleKind::Unambiguous` varient for `estree` (#10146) (Dunqing)

### Refactor

- b662df4 ast/estree: Alter `Program` start span with converter (#10195) (overlookmotel)

### Testing

- bdded7e ast/estree: Add tests for JSX via raw transfer (#10241) (overlookmotel)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- e42c040 ast/estree: Add TS fields to `LabelIdentifier` (#10295) (overlookmotel)
- 06fc07c ast/estree: Fix `TSImportType` (#10200) (therewillbecode)
- 760188e ast/estree: Fix `BindingProperty` (#10193) (therewillbecode)
- f547d76 ast/estree: Add `TSEnumBody` to `TSEnumDeclaration.body` (#10017) (Yuji Sugiura)
- 34d5c00 ast/estree: Fix `ExportDefaultDeclaration` node (#10165) (therewillbecode)
- 498b479 ast/estree: Fix `AccessorProperty` node (#10067) (therewillbecode)
- bf90072 ast/estree: Fix `ObjectProperty` node (#10018) (therewillbecode)
- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)
- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)
- 52f2a40 span/estree: Skip `ModuleKind::Unambiguous` varient for `estree` (#10146) (Dunqing)

### Refactor

- b662df4 ast/estree: Alter `Program` start span with converter (#10195) (overlookmotel)

### Testing

- bdded7e ast/estree: Add tests for JSX via raw transfer (#10241) (overlookmotel)

## [0.62.0] - 2025-04-01

### Features

- 1ab8871 napi/parser: Auto download wasm binding on webcontainer (#10049) (Hiroshi Ogawa)

### Bug Fixes

- 95e69f6 ast/estree: Fix `StringLiteral`s containing lone surrogates (#10036) (overlookmotel)
- 8408606 ast/estree: Fix `TSMethodSignature` (#10032) (therewillbecode)
- 1a0bd7c ast/estree: Fix `TSPropertySignature` (#10031) (therewillbecode)
- 707a776 ast/estree: Fix TS type defs for `TSIndexSignature` and `TSIndexSignatureName` (#10003) (overlookmotel)
- c98d3f4 ast/estree: Add custom serializer for extends field of TSInterfaceDeclaration (#9996) (therewillbecode)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Testing

- ab1a796 napi: Disable NAPI parser tests for TS files (#10028) (overlookmotel)

## [0.62.0] - 2025-04-01

### Features

- 1ab8871 napi/parser: Auto download wasm binding on webcontainer (#10049) (Hiroshi Ogawa)

### Bug Fixes

- 95e69f6 ast/estree: Fix `StringLiteral`s containing lone surrogates (#10036) (overlookmotel)
- 8408606 ast/estree: Fix `TSMethodSignature` (#10032) (therewillbecode)
- 1a0bd7c ast/estree: Fix `TSPropertySignature` (#10031) (therewillbecode)
- 707a776 ast/estree: Fix TS type defs for `TSIndexSignature` and `TSIndexSignatureName` (#10003) (overlookmotel)
- c98d3f4 ast/estree: Add custom serializer for extends field of TSInterfaceDeclaration (#9996) (therewillbecode)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Testing

- ab1a796 napi: Disable NAPI parser tests for TS files (#10028) (overlookmotel)

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

- db642eb ast/estree: Shorten raw deser code (#9944) (overlookmotel)

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

- db642eb ast/estree: Shorten raw deser code (#9944) (overlookmotel)

## [0.61.1] - 2025-03-21

### Features

- 8e3b20d napi/parser: Add portable wasm browser build (#9901) (Hiroshi Ogawa)

## [0.61.1] - 2025-03-21

### Features

- 8e3b20d napi/parser: Add portable wasm browser build (#9901) (Hiroshi Ogawa)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)
- 2cedfe4 napi: Add codeframe to napi error (#9893) (Boshen)
- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)
- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

### Bug Fixes

- 28a2ed3 estree/ast: Fix `IdentifierName` and `IdentifierReference` (#9863) (hi-ogawa)

### Performance

- 5f97f28 ast/estree: Speed up raw deser for `JSXElement` (#9895) (overlookmotel)

### Documentation

- 590a258 napi/parser: Add stackblitz link for wasm build (Boshen)

### Refactor

- 961b95d napi: Move common code to `oxc_napi` (#9875) (Boshen)
- 233c1fc napi/playground: Add JSON.parse wrapper (#9880) (Hiroshi Ogawa)

### Testing

- 040e993 napi: Refactor NAPI parser benchmarks (#9911) (overlookmotel)
- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)
- 2cedfe4 napi: Add codeframe to napi error (#9893) (Boshen)
- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)
- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

### Bug Fixes

- 28a2ed3 estree/ast: Fix `IdentifierName` and `IdentifierReference` (#9863) (hi-ogawa)

### Performance

- 5f97f28 ast/estree: Speed up raw deser for `JSXElement` (#9895) (overlookmotel)

### Documentation

- 590a258 napi/parser: Add stackblitz link for wasm build (Boshen)

### Refactor

- 961b95d napi: Move common code to `oxc_napi` (#9875) (Boshen)
- 233c1fc napi/playground: Add JSON.parse wrapper (#9880) (Hiroshi Ogawa)

### Testing

- 040e993 napi: Refactor NAPI parser benchmarks (#9911) (overlookmotel)
- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)
- 06518ae napi/parser: `JSON.parse` the returned AST in wasm (#9630) (Boshen)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)

### Refactor


### Testing

- 48bac92 napi/parser: Test wasi browser (#9793) (Hiroshi Ogawa)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)
- 06518ae napi/parser: `JSON.parse` the returned AST in wasm (#9630) (Boshen)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)

### Refactor


### Testing

- 48bac92 napi/parser: Test wasi browser (#9793) (Hiroshi Ogawa)

## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features

- 446d11e ast/estree: Export `Node` union type (#9574) (hi-ogawa)

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features

- 446d11e ast/estree: Export `Node` union type (#9574) (hi-ogawa)

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 638007b parser: Apply `preserveParens` to `TSParenthesizedType` (#9653) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)
- 6ac3635 napi/parser: Disable raw transfer on unsupported platforms (#9651) (overlookmotel)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 638007b parser: Apply `preserveParens` to `TSParenthesizedType` (#9653) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)
- 6ac3635 napi/parser: Disable raw transfer on unsupported platforms (#9651) (overlookmotel)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

## [0.56.4] - 2025-03-07

### Bug Fixes

- c08b7fc napi: Commit wasi files (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

### Bug Fixes

- 91c9932 napi: Do not support raw transfer on WASM32 (#9566) (overlookmotel)

## [0.55.0] - 2025-03-05

- 4056560 ast/estree: [**BREAKING**] Option to return JS-only AST (#9520) (overlookmotel)

### Features

- af02a87 ast/estree: `Property` have consistent field order (#9547) (overlookmotel)
- 3e4f909 ast/estree: ESTree AST `ExportNamedDeclaration` always have `attributes` field (#9546) (overlookmotel)
- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

### Bug Fixes

- a0f6f37 ast/estree: Raw transfer support `showSemanticErrors` option (#9522) (overlookmotel)

### Refactor

- c1a8cea ast/estree: Simplify serializing `RegExpLiteral`s (#9551) (overlookmotel)

### Testing

- 4378a66 ast/estree: Speed up raw transfer tests (#9521) (overlookmotel)

## [0.54.0] - 2025-03-04

- 355a4db napi/parser: [**BREAKING**] Remove `parse_without_return` API (#9455) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 68c77c8 napi/parser: Return semantic errors (#9460) (Boshen)

### Testing

- d129055 napi: Add tests for worker threads (#9408) (Boshen)
- 48d51e3 napi: Add tests for `hashbang` field (#9386) (overlookmotel)

## [0.53.0] - 2025-02-26

- 4a5a7cf napi/parser: [**BREAKING**] Remove magic string; enable utf16 span converter by default (#9291) (Boshen)

### Features


### Performance

- 61939ca ast/estree: Faster UTF-8 to UTF-16 span conversion (#9349) (overlookmotel)

### Refactor

- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


### Bug Fixes

- b9c8a10 wasm: Transfer AST to JS as JSON string in `oxc-wasm` (#9269) (overlookmotel)
- 5acc6ec wasm: Transfer AST to JS as JSON string (#9259) (overlookmotel)

## [0.51.0] - 2025-02-15

### Bug Fixes

- 0937a55 napi/parser: Utf16 span for errors (#9112) (hi-ogawa)
- 15f23f1 napi/parser: Utf16 span for module record (#9093) (hi-ogawa)
- 9edfb1d napi/parser: Fix unicode comment panic (#9084) (hi-ogawa)

### Performance

- af59945 napi/parser: Do not convert comment spans twice (#9087) (overlookmotel)

### Testing

- eaff3d9 napi/parser: Split tests for `convertSpanUtf16` (#9113) (hi-ogawa)

## [0.50.0] - 2025-02-12

### Features

- 81c81a7 napi/parser: Add `convert_span_utf16` option (#8983) (hi-ogawa)

### Bug Fixes

- 41dba62 ast/estree: Set `value` for `BigIntLiteral`s and `RegExpLiteral`s on JS side (#9044) (overlookmotel)

### Testing

- ef553b9 napi: Add NAPI parser benchmark (#9045) (overlookmotel)

## [0.49.0] - 2025-02-10

### Bug Fixes

- a520986 ast: Estree compat `Program.sourceType` (#8919) (Hiroshi Ogawa)
- e30cf6a ast: Estree compat `MemberExpression` (#8921) (Hiroshi Ogawa)
- 0c55dd6 ast: Serialize `Function.params` like estree (#8772) (Hiroshi Ogawa)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- 4803059 ast: Remove old ast snapshot tests (#8976) (hi-ogawa)

## [0.47.1] - 2025-01-19

### Features

- ee8ee55 napi/parser: Add `.hasChanged()` to `MagicString` (#8586) (Boshen)
- 1bef911 napi/parser: Add source map API (#8584) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- c479a58 napi/parser: Expose dynamic import expressions (#8540) (Boshen)

