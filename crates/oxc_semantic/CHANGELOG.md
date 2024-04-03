# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.1] - 2024-04-03

### Bug Fixes

- Flag function expressions with `SymbolFlags::Function` (#2891)

## [0.11.0] - 2024-03-30

### Features

- Add `Span` for JSDoc, JSDocTag (#2815)
- Distinguish type imports in ModuleRecord (#2785)

### Bug Fixes

- Ignore export declaration in no-duplicates (#2863)
- Missing SymbolFlags::Export when identifier used in ExportDefaultDeclaration (#2837)
- Incorrect ExportEntry span for ExportAllDeclaration in ModuleRecord (#2793)
- ModuleRecord's indirect_export_entires missing reexported imports (#2792)

### Refactor

- Distinguish whether requested_modules is type imports/exports (#2848)
- JSDocTag parser rework (#2765)

## [0.10.0] - 2024-03-14

### Features

- Merge features `serde` and `wasm` to `serialize` (#2716)
- Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)
- Move redeclare varaibles to symbol table (#2614)

### Bug Fixes

- Support multibyte chars (#2694)
- Fix up builder (#2623)

### Refactor

- Remove unused dependencies (#2718)
- Refactor `Trivias` API - have less noise around it (#2692)
- Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669)
- Import `Tsify` to shorten code (#2665)
- "wasm" feature enable "serde" feature (#2639)
- Shorten manual TS defs (#2638)
- Make `CompactStr` immutable (#2620)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Features

- Remove all commonjs logic for import plugin (#2537)
- Call build module record (#2529)

### Bug Fixes

- Jsx reference with an incorrect node id (#2546)
- Incorrect scope for switch statement (#2513)

### Refactor

- Misc fixes for JSDoc related things (#2531)
- Replace InlinableString with CompactString for `Atom` (#2517)

## [0.8.0] - 2024-02-26

### Features

- Handle cjs `module.exports = {} as default export (#2493)
- Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
- Handle top-level `require` for import plugin (#2491)
- Add check for duplicate class elements in checker (#2455)
- Add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445)

### Bug Fixes

- Add export symbol flag to identifiers in export declarations (#2508)
- Improve import/no-named-as-default (#2494)
- Should return nearest JSDoc (#2490)
- Refactor jsdoc finding (#2437)
- Incorrect reference flag for MemberExpression assign (#2433)

### Performance

- Reduce visit parent nodes in resolve_reference_usages (#2419)

### Refactor

- S/NumberLiteral/NumericLiteral to align with estree
- S/ArrowExpression/ArrowFunctionExpression to align estree
- Remove `panic!` from examples (#2454)
- Delete the redundant code in binder (#2423)
- Reduce allocation in resolve_references_for_current_scope (#2414)
- Check directive by current_scope_id (#2411)

## [0.7.0] - 2024-02-09

### Features

- Add export binding for ExportDefaultDeclarations in module record (#2329)
- Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317)
- Report parameter related errors for setter/getter (#2316)
- Report type parameter list cannot be empty (#2315)
- Report unexpected type annotation in ArrayPattern (#2309)
- Apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307)
- Fix memory leak by implementing inlineable string for oxc_allocator (#2294)

### Bug Fixes

- Remove unnecessary SymbolFlags::Import (#2311)
- Remove ignore cases (#2300)

## [0.6.0] - 2024-02-03

### Features

- Report no class name error (#2273)
- Check parameters property (#2264)
- Check optional parameters (#2263)
- Report error on optional variable declaration in TypeScript (#2261)
- Improve sample visualization (#2251)
- Track cfg index per ast node (#2210)
- Remove serde skip for symbol_id and reference_id (#2220)
- TypeScript definition for wasm target (#2158)
- Cfg prototype (#2019)
- Improve no_redeclare rule implementation (#2084)
- Remove import if only have type reference (#2001)

### Bug Fixes

- Handle short-circuiting operators in CFG (#2252)
- Proper traversal of try statements (#2250)
- Fix incorrect semantic example (#2198)
- Replace ClassStatickBlockAwait with ClassStaticBlockAwait (#2179)
- Print `Directive` original string (#2157)
- Incorrect reference flag (#2057)

### Refactor

- Adding binder for ImportSpecifier replaces the ModuleDeclaration's binder (#2230)
- Get function by scope_id in set_function_node_flag (#2208)
- Checking label in ContinueStatement based on LabelBuilder (#2202)
- Use LabelBuilder instead of UnusedLabeled (#2184)
- Move all miette usages to `oxc_diagnostics`
- Remove all #[dead_code[ from tester
- Rename RestElement to BindingRestElement (#2116)
- Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114)
- Improve declare symbol logic in FormalParameters (#2088)

## [0.5.0] - 2024-01-12

### Features

- Support es2015 new target (#1967)
- Allow reserved keyword defined in ts module block (#1907)
- Add current_scope_flags function in SemanticBuilder (#1906)
- Visualize symbol (#1886)
- Visualize scope (#1882)
- Improve check super implementation, reduce access nodes (#1827)
- Support get node id by scope id (#1826)
- Support eslint/no-unused-private-class-members rule (#1820)
- Add ClassTable (#1793)
- Add SymbolFlags::Function for FunctionDeclaration (#1713)

### Bug Fixes

- Remove duplicate errors in ModuleDeclaration::ImportDeclaration (#1846)

### Performance

- Check duplicate parameters in Binder of FormalParameters (#1840)
- Just need to find the AstKind::FormalParameter in is_in_formal_parameters (#1852)
- Reduce calls to span() (#1851)
- Find class node by symbols in get_parent_es6_component (#1657)

### Refactor

- Improve ClassTable implmention and merge properties and methods to elements (#1902)
- Improve check function declaration implementation (#1854)
- Rename `add_node_id` to `add_current_node_id_to_current_scope` (#1847)
- Improve check private identifier implementation (#1794)
- Remove unused methods from `AstNode`

## [0.4.0] - 2023-12-08

### Features

- Support scope descendents starting from a certain scope. (#1629)
- Parse jsdoc on `PropertyDefinition` (#1517)
- Add the basics of comment printing (#1313)
- Add to_string function to VariableDelcartionKind (#1303)

### Refactor

- Move to workspace lint table (#1444)
- VariableDeclarationKind::to_string -> as_str (#1321)

## [0.3.0] - 2023-11-06

### Features

- Read comment pragma @jsxRuntime classic / automatic (#1133)
- Implement some of needs_explicit_esm for typescript (#1047)
- Bind function expression name (#1049)
- Add utils to make logical_assignment_operators pass (#1017)
- ES2020 Nullish Coalescing Operator (#1004)
- Finish 2016 exponentiation operator (#996)
- Check non-simple lhs expression of assignment expression (#994)
- Re-enable mangler (#972)
- Partially re-enable minifier (#963)
- Add no-redeclare rule. (#683)

### Bug Fixes

- Make ExportDeclaration span accurate (#928)

### Refactor

- Move Semantic into Transformer (#1130)
- Fix the lifetime annotations around Vist and VisitMut (#973)

### Testing

- Add scoping test cases (#954)

## [0.2.0] - 2023-09-14

### Features

- Implement re-exports (#877)
- Add loaded_modules to ModuleRecord
- Add runner for import-plugin (#858)
- Add `SymbolId` and `ReferenceId` (#755)
- Add `node_id` to `Reference` (#689)

### Bug Fixes

- Symbol of identifier of top level function declaration should be in the root scope (#843)
- Make semantic own `Trivias` (#711)
- Nested references (#661)

### Performance

- Lazily build trivia map instead of build in-place (#903)

### Testing

- Test harness (#679)

