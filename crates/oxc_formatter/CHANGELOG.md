# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

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



