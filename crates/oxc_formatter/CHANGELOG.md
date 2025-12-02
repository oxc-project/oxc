# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.16.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 74cf572 ast: [**BREAKING**] Make `source` field of `TSImportType` a `StringLiteral` (#16114) (copilot-swe-agent)
- 43156ae ast: [**BREAKING**] Rename `TSImportType` `argument` field to `source` (#16110) (overlookmotel)

### 🚀 Features

- 862bdf7 oxfmt: Detect unsupported experimental options (take2) (#16088) (leaysgur)

### 🐛 Bug Fixes

- 75ac90c formatter: Comments in call arguments should be printed as-is (#16327) (Dunqing)
- fd77568 formatter: Don't wrap parenthesis for yield expression if there is no leading comment (#16326) (Dunqing)
- 8ccfb06 formatter: Should indent class extends and interface heritage when it is a member expression without type arguments (#16323) (Dunqing)
- 2b8f982 formatter: JSX text wrapping incorrect (#16318) (Dunqing)
- f3ffebe formatter: Should indent variable declarator if there is a trailing comment (#16243) (Dunqing)
- 31d3186 formatter: Incorrect handling of directives with comments (#16235) (Dunqing)
- ac8fcaf formatter: Add parens for new: private field expr (#16312) (leaysgur)
- 380a0af formatter: Incorrect printing of class binding trailing comments (#16234) (Dunqing)
- 0ca8154 formatter: Incorrect printing of trailing comments of callee when the call arguments are empty (#16232) (Dunqing)
- ac3a92e formatter: Print comment in ternary jsx (#16224) (leaysgur)
- e3a7388 formatter: Fix parens for static member chain (#16229) (leaysgur)
- 55334c3 formatter: Incorrect printing of dangling comments in the if statement (#16228) (Dunqing)
- 9096a63 formatter: Correct printing of trailing comments after the semicolon for class properties (#16225) (Dunqing)
- 75fd568 formatter: Inconsistent union type output between two runs (#16222) (Dunqing)
- cd70484 formatter: Should not add a hard space before function body (#16221) (Dunqing)
- 9097167 formatter: Incorrect printing of union types with comments (#16205) (Dunqing)
- 79b78b3 formatter: Template literal element should not be indented (#16189) (Dunqing)
- 48d6ed2 formatter: Nested assignment pattern should not expand outer object pattern (#16160) (Dunqing)
- 8f4137d formatter: Output is incorrect when using comments inside JSX which is the right hand-side of `LogicalExpression` (#16156) (Dunqing)
- 85c3a10 formatter/sort_imports: Handle internal prefixes correctly (#16128) (leaysgur)
- 889d2e7 formatter: Handle poor layout for grouped call arguments (#16093) (Dunqing)
- 14b0a6a oxfmt: Fix JS-ish file detection (#16092) (leaysgur)
- 9706a1a oxfmt: Ignore unsupported options (#16085) (leaysgur)

## [0.15.0] - 2025-11-24

### 💥 BREAKING CHANGES

- a937890 formatter: [**BREAKING**] Default to `lineWidth: 100` (#15933) (leaysgur)
- 03d5f5a formatter/sort-imports: [**BREAKING**] Change default order to `natural` with `natord` crate (#15828) (leaysgur)
- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

### 🚀 Features

- 7818e22 formatter/sort-imports: Support `options.groups` (#15831) (leaysgur)

### 🐛 Bug Fixes

- 4817486 formatter: Revert  `FormatElement::BestFitting` printing logic (#16028) (Dunqing)
- 5562dd6 formatter: Incorrect formatting method chain with trailing comments (#16027) (Dunqing)
- 6d14c8b formatter: Comments in export class decorators are printing incorrectly (#15897) (Dunqing)
- 683c764 formatter: Correct a few minor mismatched typescript tests (#15894) (Dunqing)
- c11cc07 formatter: Improve formatting for default type on type parameters (#15893) (Dunqing)
- 0bff596 formatter: Handle JSX expresssion dangling comment (#15890) (leaysgur)
- 16a9dc8 formatter: Inconsistent printing of class extends and interface extends (#15892) (Dunqing)
- 300b496 formatter: Inconsistent CallExpression and NewExpression around member chain and logical expression (#15858) (Dunqing)

### ⚡ Performance

- 65174cc formatter: Reduce the size of `TextWidth` to 4 byte (#15827) (Dunqing)
- 4fe3aac formatter: Use `ArenaVec` and `ArenaBox` (#15420) (Dunqing)

## [0.14.0] - 2025-11-17

### 🚀 Features

- 84de1ca oxlint,oxfmt: Allow comments and also commas for vscode-json-ls (#15612) (leaysgur)
- 25a0163 formatter/sort_imports: Sort imports by `Array<Array<string>>` groups (#15578) (leaysgur)

### 🐛 Bug Fixes

- bf20cf5 formatter: `CRLF` issue in the member chain (#15764) (Dunqing)
- 5d688a0 formatter: Measuring multiline text in `fits_text` is incorrect (#15762) (Dunqing)
- e306958 formatter: Regression case for test call (#15760) (Dunqing)
- c42d983 formatter: Re-fix all cases that fail after `AstNode::Argument` was removed (#15676) (Dunqing)

### ⚡ Performance

- 128e186 formatter/sort_imports: Precompute import metadata (#15580) (leaysgur)
- cd31cc1 formatter/sort_imports: Use `Vec::with_capacity` for `next_elements` (#15579) (leaysgur)

## [0.12.0] - 2025-11-10

### 🚀 Features

- 33ad374 oxfmt: Disable embedded formatting by default for alpha (#15402) (leaysgur)
- 5708126 formatter/sort_imports: Add `options.newlinesBetween` (#15369) (leaysgur)
- 2dfc3bd formatter: Remove `Tag::StartVerbatim` and `Tag::EndVerbatim` (#15370) (Dunqing)
- 88c7530 formatter: Remove `FormatElement::LocatedTokenText` (#15367) (Dunqing)

### 🐛 Bug Fixes

- d32d22e formatter: Correct `FormatElement` size check (#15461) (Dunqing)
- b0f43f9 formatter: Test call difference (#15356) (Dunqing)
- 01f20f3 formatter: Incorrect comment checking logic for grouping argument (#15354) (Dunqing)

### ⚡ Performance

- f4b75b6 formatter: Pre-allocate enough space for the FormatElement buffer (#15422) (Dunqing)
- 5a61189 formatter: Avoid unnecessary allocation for `BinaryLikeExpression` (#15467) (Dunqing)
- 064f835 formatter: Optimize printing call arguments (#15464) (Dunqing)
- 29f35b2 formatter: Reuse previous indent stack in `FitsMeasurer` (#15416) (Dunqing)
- 2515045 formatter: Use CodeBuffer's built-in print_indent to print indentation (#15406) (Dunqing)
- 681607b formatter: Check the `Text` to see whether it has multiple lines based on its width (#15405) (Dunqing)
- b92deb4 formatter: Replace String buffer with byte-oriented CodeBuffer (#14752) (Boshen)
- 963b87f formatter: Add `text_without_whitespace` for text that can never have whitespace (#15403) (Dunqing)
- f30ce4b formatter: Optimize formatting literal string (#15380) (Dunqing)
- 8f25a0e formatter: Memorize text width for `FormatElement::Text` (#15372) (Dunqing)
- f913543 formatter: Avoid allocation for `SyntaxTokenCowSlice` (#15366) (Dunqing)
- 98c9234 formatter: Optimize `FormatElement::Token` printing (#15365) (Dunqing)


## [0.10.0] - 2025-11-04

### 🚀 Features

- 505252c formatter: Wrap parenthesis for AssignmentExpression that is a key of `PropertyDefinition` (#15243) (Dunqing)
- 880b259 formatter: Align import-like formatting the same as Prettier (#15238) (Dunqing)
- b77f254 oxfmt,formatter: Support `embeddedLanguageFormatting` option (#15216) (leaysgur)
- 898d6fe oxfmt: Add embedded language formatting with Prettier integration (#14820) (Boshen)
- e77a48e formatter: Detect code removal feature (#15059) (leaysgur)

### 🐛 Bug Fixes

- 46793d7 formatter: Correct printing comments for `LabeledStatement` (#15260) (Dunqing)
- 831ae99 formatter: Multiple comments in `LogicalExpression` and `TSIntersectionType` (#15253) (Dunqing)
- 5fa9b1e formatter: Should not indent `BinaryLikeExpression` when it is an argument of `Boolean` (#15250) (Dunqing)
- 99e520f formatter: Handle chain expression for `JSXExpressionContainer` (#15242) (Dunqing)
- a600bf5 formatter: Correct printing comments for `TaggedTemplateExpression` (#15241) (Dunqing)
- a7289e7 formatter: Handle member chain for the call's parent is a chain expression (#15237) (Dunqing)

### 🚜 Refactor

- 36ae721 formatter: Simplify the use of `indent` with `soft_line_break_or_space` (#15254) (Dunqing)
- cdd8e2f formatter/sort-imports: Split sort_imports modules (#15189) (leaysgur)
- 85fb8e8 formatter/sort-imports: Pass options to is_ignored() (#15181) (leaysgur)

### 🧪 Testing

- 9d5b34b formatter/sort-imports: Refactor sort_imports tests (#15188) (leaysgur)


## [0.9.0] - 2025-10-30

### 🚀 Features

- 8fe7e85 formatter: Support printing Formatter IR (#14855) (Dunqing)

### 🐛 Bug Fixes

- a6b6ef8 formatter: Correct calculating layout for `TSNonNullExpression` in `StaticMemberExpression` (#15065) (Dunqing)
- 99bd995 formatter: Print parenthesis for sequence expression in `ReturnStatement` and `ExpressionStatement` (#15062) (Dunqing)
- f3fb998 formatter: Correct printing comments for `TSAsExpression` (#15061) (Dunqing)
- 1e4a018 formatter: Correct checking of the short argument for `CallArguments` (#15055) (Dunqing)
- c0dfd9b formatter: Print comments before fat arrow as-is for `ArrowFunctionExpression` (#15050) (Dunqing)
- 206b519 formatter: Should hug parameter with `TSMappedType` type annotation (#15049) (Dunqing)
- e48c604 formatter: Incorrect formatting of a function with `this` parameter (#15031) (Dunqing)
- a9f0c45 formatter: Decorators and class method on the same line shouldn't be broken by a leading comment of the method (#15029) (Dunqing)
- 43d74e4 formatter: Handle `<CRLF>` for `SourceText` (#15016) (leaysgur)
- 34fab40 formatter: Correct calculating layout for `ChainExpression` in `StaticMemberExpression` (#14986) (Dunqing)
- 68dc101 formatter: Should not break when the parent of root of `StaticMemberExpression` is used as the `Argument` of `NewExpression` (#14985) (Dunqing)
- 071b739 formatter: Align the short argument handling for UnaryExpression with Prettier (#14984) (Dunqing)
- 3940f3a formatter: `BestFitting` doesn't exactly matches the `conditinalGroup` behavior in Prettier (#14983) (Dunqing)
- 4a84e44 formatter: Align the logic of printing type parameters, parameters, and return type for functions with Prettier (#14942) (Dunqing)
- 68c1f2a formatter: Non-nested static member expressions will never break (#14929) (Dunqing)
- 42adc47 formatter: Check whether a type alias is complex when its right hand side never break (#14928) (Dunqing)
- e501f13 formatter: Should not add a trailing comma for long curried calls when `trailingComma` is es5 (#14913) (Dunqing)

### 🚜 Refactor

- 7d64291 formatter: Simplify printing ClassElement with a semicolon (#15030) (Dunqing)
- 5de99c2 formatter: Export unified way to get_parse_options (#15027) (leaysgur)
- f6f22e2 formatter: Clean up unneeded implementations for printing comments (#14935) (Dunqing)
- 7a2b9d1 formatter: Improve printing trailing comments (#14934) (Dunqing)
- ba10caa formatter: Align printing trailing comments with Prettier (#14927) (Dunqing)
- 597c9e8 formatter: Remove redundunt public API (#14915) (leaysgur)

### ⚡ Performance

- 467b3a1 formatter: Optimize grouping logic for call arguments (#15033) (Dunqing)

### 💼 Other

- aceff66 oxfmt: V0.9.0 (#15088) (Boshen)



## [0.8.0] - 2025-10-22

### 🚀 Features

- 381e08c oxfmt: More friendly JSON schema (#14879) (leaysgur)
- 006708d oxfmt: Support `ignorePatterns` in oxfmtrc (#14875) (leaysgur)

### 🐛 Bug Fixes

- 64b8226 formatter: Corrct printing leading own line comments before method body (#14886) (Dunqing)
- 6ce1162 formatter: Remove a redundant space for TSMappedType (#14885) (Dunqing)
- 5b962a7 formatter: Remove redundant leading space when only the rest part of the array exists (#14884) (Dunqing)
- 8301d8f formatter: No need to wrap parenthesis for ObjectExpression when it is an expression of a template literal (#14883) (Dunqing)
- 9397472 formatter: Should not wrap parenthesis for ComputedMemberExpression when it is not an option or it doesn't contain a call expression (#14882) (Dunqing)
- 3e62277 formatter: Should not add a soft line for the arrow function inside ExpressionContainer with a trailing comment (#14878) (Dunqing)
- 990916a formatter: Correct handling of leading own line before arrow function body (#14877) (Dunqing)
- 4a499b5 formatter: Correct printing trailing comments for if statement with non-block consequent (#14857) (Dunqing)

### 🧪 Testing

- 868ff99 formatter: Fix tests using TS syntax with `.js` (#14880) (leaysgur)


## [0.7.0] - 2025-10-21

### 🚀 Features

- aa024d9 formatter: Wrap parenthesis for `AssignmentExpression` that are inside `ComputedMemberExpression` (#14834) (Dunqing)

### 🐛 Bug Fixes

- 88fb768 formatter: Correct handling of ignore comment for `TSUnionType` and `TSMappedType` (#14824) (Dunqing)
- f7727c7 formatter: Ignore comment doesn't work for the expression statement (#14817) (Dunqing)
- 7a420a1 oxfmt: Handle `.d.ts` file correctly (#14835) (leaysgur)

### 🚜 Refactor

- 9d914a3 formatter: Improve comments handling (#14816) (Dunqing)
- f52863d formatter: Improve handling of type cast node (#14815) (Dunqing)


## [0.6.0] - 2025-10-20

### 🚀 Features

- 6bf8bac formatter: Reimplement formatting for `ImportExpression` (#14712) (Dunqing)
- 3f2e036 formatter: Introduce `AstNode<ExpressionStatement>::is_arrow_function_body` (#14709) (Dunqing)
- df225e9 formatter: Add `AstNode::ancestor` and `AstNode::grand_parent` methods (#14700) (Dunqing)
- fec2ed9 oxfmt: Use Prettier style config key and value (#14612) (leaysgur)
- 1b58521 oxfmt,language_server: Enable JSX for all JS source type (#14605) (leaysgur)

### 🐛 Bug Fixes

- 21c4285 formatter: Correct printing remaining trailing comments for `TSMappedType` (#14761) (Dunqing)
- 1d1573e formatter: Correct adding semicolons for TypeScript left-hand side nodes (#14760) (Dunqing)
- 4cc3b10 formatter: Improve handling of new lines between comments in `MemberChain` (#14759) (Dunqing)
- e6bce8e formatter: Break the left hand side of AssignmentLike node if it is an `ObjectPattern` with three properties (#14756) (Dunqing)
- dc57a2b formatter: Incorrect handling of `VariableDeclarator` with an `ArrowFunctionExpression` initializer (#14731) (Dunqing)
- 537185d formatter: Should always group the left side of `AssignmentPattern` (#14730) (Dunqing)
- 4283fd8 formatter: Correct printing comments for `JSXAttributeValue` (#14719) (Dunqing)
- 59c9e1b formatter: Avoid conditional being broken in arguments by trailing comments (#14718) (Dunqing)
- 7d64b96 formatter: Should wrap parentheses with JSX arguments of `NewExpression` (#14717) (Dunqing)
- 2068a63 formatter: Should indent TemplateExpression if it is a member expression that is part of `ChainExpression` (#14714) (Dunqing)
- 5ea3bb6 formatter: Incorrect handling of `ObjectPattern` as an `AssignmentPattern` of a parameter (#14711) (Dunqing)
- eb52529 formatter: Incorrect handling of `ObjectPattern` as a parameter (#14670) (Dunqing)
- 8ac10da formatter: Correct checking assignment layout for the right side with ownline comment and it is a `PrivateFieldExpression` (#14664) (Dunqing)
- 6cba9b1 formatter: Should not merge tail with head for MemberChain when its parent is ArrowFunctionExpression (#14663) (Dunqing)
- f44d3c0 formatter: Should not indent BinaryLikeExpression when its parent is `NewExpression` (#14662) (Dunqing)
- bf953b8 formatter: Should group nested test for TSConditionalType (#14661) (Dunqing)
- 63dc57b formatter: Correct handling if a template literal should be printed as a signle line (#14660) (Dunqing)
- 0150ad5 formatter: Should group type parameters and parameters for method-like and function-like nodes (#14659) (Dunqing)
- 392bf74 formatter: Improve handling of dangling comments in if statements (#14658) (Dunqing)
- fd52b10 formatter: Don't print CallExpression as MemberChain style when its only has one argument and it is a TemplateLiteral on its own line (#14657) (Dunqing)
- 29c3c60 formatter: Don't group nested await expression when its is the leftmost expression (#14656) (Dunqing)
- 72c7503 formatter: Correct checking function composition when the arguments have an call expression before function (#14655) (Dunqing)
- 2b645e2 formatter: Don't wrap parenthesis for `type` when its grandparent isn't a `ExpressionStatement` (#14654) (Dunqing)
- e0eb966 formatter: Skip the leading semicolon when calculating leading lines (#14653) (Dunqing)

### 🚜 Refactor

- 83e783a formatter: Organize utils structure (#14710) (Dunqing)
- 58dd74a formatter: Remove all `without_parentheses` usages (#14707) (Dunqing)
- 75dfcad formatter: Simplify `ExpressionLeftSide` (#14706) (Dunqing)
- 273f0fe formatter: Remove unnecessary lifetimes for the implementations of `NeedsParentheses` (#14703) (Dunqing)
- bae5f11 formatter: Improve `AstNode` and `AstNodes` (#14686) (Dunqing)

### ⚡ Performance

- c6395c7 formatter: Optimize string reservation in print_text (#14751) (Boshen)

### 🧪 Testing

- 42d8c62 formatter: Add tests for sort-imports (#14685) (leaysgur)


## [0.5.0] - 2025-10-14

### 🚀 Features

- 8077f9b oxfmt: Provide JSON schema for `.oxfmtrc.json` (#14399) (leaysgur)
- 51ddfa8 oxfmt: Support `.oxfmtrc.json(c)` config file (#14398) (leaysgur)

### 🐛 Bug Fixes

- c4e12df formatter: Correct checking comments around the return argument (#14585) (Dunqing)
- 454303b formatter: Check empty lines between arguments incorrectly (#14584) (Dunqing)
- a5554a8 formatter: Print a line break incorrectly for directives (#14577) (Dunqing)
- 5db9774 formatter: Correct printing comments that are around the `StaticMemberExpression` (#14543) (Dunqing)
- 620dbac formatter: No need to wrap a parenthesis for `TSConditionalType` and `TSFunctionType` when its parent is `TSUnionType` and it only has one element (#14540) (Dunqing)
- 1aec74f formatter: Missing parenthesis for `ComputedMemberExpression` when its parent is an `NewExpression` (#14530) (Dunqing)
- 59f1d8f formatter: Missing parenthesis for `PrivateInExpression` when its parent is an `UnaryExpression` (#14529) (Dunqing)
- b06059e formatter: Correct printing comments for logical expression (#14500) (Dunqing)
- be38095 formatter: Should break even if the right side of the assignment is a require when it has a leading own line comment (#14499) (Dunqing)
- 4d55654 formatter: Correct printing comments for the end of union type (#14498) (Dunqing)
- 4994872 formatter: Correct printing comments for `try` statement (#14497) (Dunqing)
- 9a4da3d formatter: Should expand the parent if the member chain will break (#14487) (Dunqing)
- bba9689 formatter: Correct printing comments around the expression of `ComputedMemberExpression` (#14486) (Dunqing)
- dc5e08a formatter: Correct printing yield argument (#14485) (Dunqing)
- b8a0905 formatter: No need to wrap with parentheses for a type cast node (#14484) (Dunqing)
- b159215 formatter: Ignore the leading line break for the first argument of the call expression (#14483) (Dunqing)
- ea8f9ed formatter: Correct checking comments between the operator and the right side for assignment like nodes (#14482) (Dunqing)
- 4f19504 formatter: Block comments without a leading linebreak that are around the operator of the conditional expression should be printed as-is (#14474) (Dunqing)
- ef77997 formatter: Correct printing comments for `for` statements (#14473) (Dunqing)
- 31595c3 formatter: Correct printing comments for assignment pattern (#14469) (Dunqing)
- 0f19be0 oxfmt: Normalize path delimiter on Windows (#14463) (leaysgur)
- 5856bc9 oxc_formatter: Fix arrow_parentheses: 'avoid' > 'as-needed' (#14462) (leaysgur)

### 🚜 Refactor

- 7bc86f4 formatter: Simplify foramtting import and export (#14576) (Dunqing)
- 29c3cf2 formatter: Remove `SiblingNode` (#14551) (Dunqing)
- 893bdac formatter: Improve printing comments for special nodes (#14544) (Dunqing)
- 97bb964 formatter: Improve line break detaching logic in SourceText (#14539) (Dunqing)

### 🧪 Testing

- bb43dc5 formatter: Add snapshot-based test infrastructure (#14400) (Dunqing)


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



