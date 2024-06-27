# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.16.0] - 2024-06-26

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- 0673677 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `Function` (#3844) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 9b38119 ast: [**BREAKING**] Replace `Modifiers` with `declare` on `VariableDeclaration` (#3839) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

### Features

- d5f6aeb semantic: Check for illegal symbol modifiers (#3838) (Don Isaac)

### Bug Fixes

- 8c9fc63 semantic: Apply strict mode scope flag for strict mode TS Modules (#3861) (overlookmotel)
- 99a40ce semantic: `export default foo` should have `ExportLocalName::Default(NameSpan)` entry (#3823) (Boshen)
- 17ad8f7 transformer: Create new scopes for new blocks in TS transform (#3908) (overlookmotel)

### Performance

- 10d1de5 semantic: Remove uneccessary allocation in builder (#3867) (DonIsaac)- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 187f078 parser: Improve parsing of `parse_function_or_constructor_type` (#3892) (Boshen)- d6437fe Clean up some usages of `with_labels` (#3854) (Boshen)

## [0.15.0] - 2024-06-18

- 0537d29 cfg: [**BREAKING**] Move control flow to its own crate. (#3728) (rzvxa)

- 4bce59d semantic/cfg: [**BREAKING**] Re-export `petgraph` as `control_flow::graph`. (#3722) (rzvxa)

- 0578ece ast: [**BREAKING**] Remove `ExportDefaultDeclarationKind::TSEnumDeclaration` (#3666) (Dunqing)

### Features

- 046ff3f linter/eslint: Add `no_unreachable` rule. (#3238) (rzvxa)
- 9c31ed9 semantic/cfg: Propagate unreachable edges through subgraphs. (#3648) (rzvxa)
- d9c5b33 semantic/cfg: Add `Condition` instruction. (#3567) (Ali Rezvani)
- f2dfd66 semantic/cfg: Add iteration instructions. (#3566) (rzvxa)

### Bug Fixes

- 70fc69b semantic: Add Eq to CtxFlags (#3651) (Yuji Sugiura)
- 7a58fec semantic/cfg: Issue in unlabeled `Ctx`s. (#3678) (rzvxa)
- abd6ac8 semantic/cfg: Discrete finalization path after `NewFunction`s. (#3671) (rzvxa)
- e148a32 semantic/cfg: Correct unreachability propagation in try-finally. (#3667) (Ali Rezvani)

### Performance

- 2717a1a semantic/cfg: Lower the visits in `neighbors_filtered_by_edge_weight`. (#3676) (rzvxa)

### Refactor

- 7ec44f8 semantic: Rename `cfg` macro to `control_flow`. (#3742) (rzvxa)
- d8ad321 semantic: Make control flow generation optional. (#3737) (rzvxa)
- a94a72d semantic: Expose 1 checker function instead of 2 (#3694) (Boshen)
- bd8d115 semantic/cfg: Remove unused types. (#3677) (rzvxa)
- f702fb9 semantic/cfg: Cleanup control flow and it's builder. (#3650) (rzvxa)

## [0.14.0] - 2024-06-12

### Refactor

- 84304b4 linter: Add a `ctx.module_record()` method (#3637) (Boshen)
- 7d61832 semantic: Pass `Rc` by value (#3586) (overlookmotel)
- 5793ff1 transformer: Replace `&’a Trivias` with `Rc<Trivias>` (#3580) (Dunqing)
- 60cbdec traverse: `generate_uid_in_root_scope` method (#3611) (overlookmotel)

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

- 3a5f088 linter/jsdoc: Implement require-returns rule (#3218) (Yuji Sugiura)
- bcdc658 transformer: Add `TraverseCtx::generate_uid` (#3394) (overlookmotel)

### Bug Fixes

- 5e06298 linter: Memorize visited block id in `neighbors_filtered_by_edge_weight` (#3407) (mysteryven)
- 6f71541 semantic: Set program scope_id for TS definition files (#3496) (overlookmotel)
- d4371e8 transformer: Use UIDs in TS namespace transforms (#3395) (overlookmotel)

### Refactor

- 55bbde2 ast: Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488) (overlookmotel)
- 9c58231 semantic: Use a simpler way to resolve reference for ReferenceFlag::Type (#3430) (Dunqing)- de75fb2 Compile less test binaries to speed up CI (#3414) (Boshen)

## [0.13.1] - 2024-05-22

### Refactor

- 6f3b1c8 semantic: Semantic populate `scope_id` fields in AST (#3303) (overlookmotel)
- 78e6326 semantic/cfg: Alias petgraph's `NodeIndex` as `BasicBlockId`. (#3380) (rzvxa)

## [0.13.0] - 2024-05-14

### Features

- 44b16ef linter/eslint: Implement max-classes-per-file (#3241) (Jelle van der Waa)
- 5866086 linter/jsdoc: Implement no-defaults rule (#3098) (Yuji Sugiura)
- 1f135ce linter/react: Add the `rules_of_hooks` rule. (#3071) (rzvxa)
- c3d8a85 semantic: Report that enum member must have initializer (#3113) (Dunqing)
- 2dd96df semantic: Report namespace related errors (#3093) (Dunqing)

### Bug Fixes

- 5e36e0d semantic: Add `cfg` nodes for `ConditionalExpression`s. (#3127) (Ali Rezvani)
- c91d261 semantic: Connect `test` expression of `for` statements to the cfg. (#3122) (Ali Rezvani)
- dcb2528 semantic: Revert test code pushed to the main by accident. (#3085) (Ali Rezvani)
- 8d17ab3 semantic: Allow `root_node` to be empty for empty trees. (#3084) (Ali Rezvani)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- dbde5b3 diagnostics: Remove export of `miette` (Boshen)
- 312f74b diagnostics: S/OxcDiagnostic::new/OxcDiagnostic::error (Boshen)
- c5588c9 semantic: Clean up redeclaration diagnostic (Boshen)
- 09f34fc semantic: Unify diagnostic in checker (Boshen)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)- 893af23 Clean up more diagnostics usages (Boshen)- d8173e1 Remove all usages of `Into<Error>` (Boshen)

## [0.12.5] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)
- 57ad6c4 semantic: Add root node to the `AstNodes` structure. (#3032) (Ali Rezvani)

### Bug Fixes

- 84c43c8 semantic: Correctly resolve identifiers inside catch parameter initializers (#3050) (Boshen)
- 1f7033e semantic: Correctly resolve identifiers inside parameter initializers (#3046) (Boshen)

## [0.12.4] - 2024-04-19

### Features

- 40af2b1 semantic/jsdoc: Handle optional type syntax for type name part (#2960) (Yuji Sugiura)

### Bug Fixes

- 2c325ef semantic/jsdoc: Skip parsing `@` inside of backticks (#3017) (Yuji Sugiura)

## [0.12.3] - 2024-04-11

### Refactor

- 0a77d62 semantic/jsdoc: Rework JSDoc struct for better Span handling (#2917) (Yuji Sugiura)

## [0.12.2] - 2024-04-08

### Features

- aa63b64 linter: Implement jsdoc/check-access (#2642) (Yuji Sugiura)

### Bug Fixes

- 1ea24ea semantic: Symbols inside functions and classes incorrectly flagged as exported (#2896) (Don Isaac)

## [0.12.1] - 2024-04-03

### Bug Fixes

- d3eb1c3 semantic: Flag function expressions with `SymbolFlags::Function` (#2891) (Don Isaac)

## [0.11.0] - 2024-03-30

### Features

- 712b3d2 semantic: Distinguish type imports in ModuleRecord (#2785) (Dunqing)
- df744b2 semantic/jsdoc: Add `Span` for JSDoc, JSDocTag (#2815) (Yuji Sugiura)

### Bug Fixes

- df62828 linter/import: Ignore export declaration in no-duplicates (#2863) (Dunqing)
- 947a9f0 semantic: Missing SymbolFlags::Export when identifier used in ExportDefaultDeclaration (#2837) (Dunqing)
- b28b617 semantic: Incorrect ExportEntry span for ExportAllDeclaration in ModuleRecord (#2793) (Dunqing)
- b6e493b semantic: ModuleRecord's indirect_export_entires missing reexported imports (#2792) (Dunqing)

### Refactor

- 1b5e544 semantic: Distinguish whether requested_modules is type imports/exports (#2848) (Dunqing)
- 4a42c5f semantic/jsdoc: JSDocTag parser rework (#2765) (Yuji Sugiura)

## [0.10.0] - 2024-03-14

### Features

- 57ce737 semantic: Move redeclare varaibles to symbol table (#2614) (Dunqing)
- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

### Bug Fixes

- b00d4b8 semantic/jsdoc: Support multibyte chars (#2694) (Yuji Sugiura)
- 2609e90 semantic/jsdoc: Fix up builder (#2623) (Yuji Sugiura)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)
- cba1e2f ast: Import `Tsify` to shorten code (#2665) (overlookmotel)
- 6b5723c ast: Shorten manual TS defs (#2638) (overlookmotel)- cbc2f5f Remove unused dependencies (#2718) (Boshen)- 3c1e0db Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) (overlookmotel)- d76ee6b "wasm" feature enable "serde" feature (#2639) (overlookmotel)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Features

- d41dcc3 linter: Remove all commonjs logic for import plugin (#2537) (Boshen)
- f760108 transformer: Call build module record (#2529) (Dunqing)

### Bug Fixes

- 37de80d semantic: Jsx reference with an incorrect node id (#2546) (Dunqing)
- 1519b90 semantic: Incorrect scope for switch statement (#2513) (Dunqing)

### Refactor

- 1391e4a semantic/jsdoc: Misc fixes for JSDoc related things (#2531) (Yuji Sugiura)- c56b6cb Replace InlinableString with CompactString for `Atom` (#2517) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- f5aadc7 linter: Handle cjs `module.exports = {} as default export (#2493) (Boshen)
- f64c7e0 linter: Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492) (Boshen)
- d0a9c46 linter: Handle top-level `require` for import plugin (#2491) (Boshen)
- 197fa16 semantic: Add check for duplicate class elements in checker (#2455) (Dunqing)
- 950298d semantic: Add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445) (Dunqing)

### Bug Fixes

- fba66dc linter: Improve import/no-named-as-default (#2494) (Boshen)
- 4c2e2bd semantic: Add export symbol flag to identifiers in export declarations (#2508) (Dunqing)
- 04f4621 semantic: Should return nearest JSDoc (#2490) (Yuji Sugiura)
- bc22ae5 semantic: Refactor jsdoc finding (#2437) (Yuji Sugiura)
- 5bd2ce6 semantic: Incorrect reference flag for MemberExpression assign (#2433) (Dunqing)

### Performance

- 8110288 semantic: Reduce visit parent nodes in resolve_reference_usages (#2419) (Dunqing)

### Refactor

- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)
- 7c2d868 semantic: Delete the redundant code in binder (#2423) (Dunqing)
- c6767fa semantic: Reduce allocation in resolve_references_for_current_scope (#2414) (Dunqing)
- 28ba28f semantic: Check directive by current_scope_id (#2411) (Dunqing)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.7.0] - 2024-02-09

### Features

- d571839 ast: Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317) (Dunqing)
- 40e9541 semantic: Add export binding for ExportDefaultDeclarations in module record (#2329) (Dunqing)
- a3570d4 semantic: Report parameter related errors for setter/getter (#2316) (Dunqing)
- 9ca13d0 semantic: Report type parameter list cannot be empty (#2315) (Dunqing)
- f53c54c semantic: Report unexpected type annotation in ArrayPattern (#2309) (Dunqing)
- f3035f1 semantic: Apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307) (Dunqing)
- 6002560 span: Fix memory leak by implementing inlineable string for oxc_allocator (#2294) (Boshen)

### Bug Fixes

- 540b2a0 semantic: Remove unnecessary SymbolFlags::Import (#2311) (Dunqing)
- cb17a83 semantic: Remove ignore cases (#2300) (Dunqing)

## [0.6.0] - 2024-02-03

### Features

- f673e41 ast: Remove serde skip for symbol_id and reference_id (#2220) (Dunqing)
- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)
- 721a869 linter: Improve no_redeclare rule implementation (#2084) (Dunqing)
- 2768195 oxc_semantic: Improve sample visualization (#2251) (Tzvi Melamed)
- 28daf83 semantic: Report no class name error (#2273) (Boshen)
- da2ffdf semantic: Check parameters property (#2264) (Dunqing)
- d71175e semantic: Check optional parameters (#2263) (Dunqing)
- 8d99a15 semantic: Report error on optional variable declaration in TypeScript (#2261) (Boshen)
- e561457 semantic: Track cfg index per ast node (#2210) (Tzvi Melamed)
- 8898377 semantic: Cfg prototype (#2019) (Boshen)
- ead4e8d transformer/typescript: Remove import if only have type reference (#2001) (Dunqing)

### Bug Fixes

- 989ab88 codegen: Print `Directive` original string (#2157) (underfin)
- f4674f3 oxc_semantic: Handle short-circuiting operators in CFG (#2252) (Tzvi Melamed)
- 73ccf8a oxc_semantic: Proper traversal of try statements (#2250) (Tzvi Melamed)
- 972be83 semantic: Fix incorrect semantic example (#2198) (Dunqing)
- 122abd5 semantic: Replace ClassStatickBlockAwait with ClassStaticBlockAwait (#2179) (Dunqing)
- 24ac957 semantic: Incorrect reference flag (#2057) (Dunqing)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)
- 2924258 semantic: Adding binder for ImportSpecifier replaces the ModuleDeclaration's binder (#2230) (Dunqing)
- c62495d semantic: Get function by scope_id in set_function_node_flag (#2208) (Dunqing)
- f59e87f semantic: Checking label in ContinueStatement based on LabelBuilder (#2202) (Dunqing)
- 56adfb1 semantic: Use LabelBuilder instead of UnusedLabeled (#2184) (Dunqing)
- fc1592b semantic: Remove all #[dead_code[ from tester (Boshen)
- 8bccdab semantic: Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114) (Dunqing)
- 8e43eef semantic: Improve declare symbol logic in FormalParameters (#2088) (Dunqing)- 87b9978 Move all miette usages to `oxc_diagnostics` (Boshen)

## [0.5.0] - 2024-01-12

### Features

- f45a3cc linter: Support eslint/no-unused-private-class-members rule (#1820) (Dunqing)
- f1b433b playground: Visualize symbol (#1886) (Dunqing)
- 45a7985 playground: Visualize scope (#1882) (Dunqing)
- 3b4fe0e semantic: Allow reserved keyword defined in ts module block (#1907) (Dunqing)
- b0569bc semantic: Add current_scope_flags function in SemanticBuilder (#1906) (Dunqing)
- b9bdf36 semantic: Improve check super implementation, reduce access nodes (#1827) (Dunqing)
- f7b7f0a semantic: Support get node id by scope id (#1826) (Dunqing)
- ca04312 semantic: Add ClassTable (#1793) (Dunqing)
- edc6fa4 semantic: Add SymbolFlags::Function for FunctionDeclaration (#1713) (Dunqing)
- 78b427b transform: Support es2015 new target (#1967) (underfin)

### Bug Fixes

- 9c9d882 semantic: Remove duplicate errors in ModuleDeclaration::ImportDeclaration (#1846) (Dunqing)

### Performance

- 0080638 linter/react: Find class node by symbols in get_parent_es6_component (#1657) (Dunqing)
- dae5f62 semantic: Check duplicate parameters in Binder of FormalParameters (#1840) (Dunqing)
- a743d06 semantic: Just need to find the AstKind::FormalParameter in is_in_formal_parameters (#1852) (Dunqing)
- 0e0f258 semantic: Reduce calls to span() (#1851) (Dunqing)

### Refactor

- 6c5b22f semantic: Improve ClassTable implmention and merge properties and methods to elements (#1902) (Dunqing)
- bfd5cd9 semantic: Improve check function declaration implementation (#1854) (Dunqing)
- 497a0b8 semantic: Rename `add_node_id` to `add_current_node_id_to_current_scope` (#1847) (Dunqing)
- d63c50a semantic: Improve check private identifier implementation (#1794) (Dunqing)
- da67fe1 semantic: Remove unused methods from `AstNode` (Boshen)

## [0.4.0] - 2023-12-08

### Features

- 446ba16 ast: Add to_string function to VariableDelcartionKind (#1303) (Dunqing)
- 0115314 ast/semantic: Parse jsdoc on `PropertyDefinition` (#1517) (Shannon Rothe)
- 5f31662 prettier: Add the basics of comment printing (#1313) (Boshen)
- c6ad660 semantic: Support scope descendents starting from a certain scope. (#1629) (Miles Johnson)

### Refactor

- be043c3 ast: VariableDeclarationKind::to_string -> as_str (#1321) (Boshen)
- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 2453954 linter: Add no-redeclare rule. (#683) (cin)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- a442fad semantic: Bind function expression name (#1049) (Boshen)
- 1661385 semantic: Check non-simple lhs expression of assignment expression (#994) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)
- dfee853 transformer: Add utils to make logical_assignment_operators pass (#1017) (Boshen)
- 678db1d transformer: ES2020 Nullish Coalescing Operator (#1004) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)
- 203cf37 transformer/react: Read comment pragma @jsxRuntime classic / automatic (#1133) (Boshen)

### Bug Fixes

- 0f02d37 semantic: Make ExportDeclaration span accurate (#928) (Wenzhe Wang)

### Refactor

- 903854d ast: Fix the lifetime annotations around Vist and VisitMut (#973) (Boshen)
- 69150d8 transformer: Move Semantic into Transformer (#1130) (Boshen)

### Testing

- b4b39b8 semantic: Add scoping test cases (#954) (Don Isaac)

## [0.2.0] - 2023-09-14

### Features

- e7c2313 ast: Add `SymbolId` and `ReferenceId` (#755) (Yunfei He)
- 4e5f63a linter: Implement re-exports (#877) (Boshen)
- ee54575 linter: Add runner for import-plugin (#858) (Boshen)
- c5ff534 semantic: Add `node_id` to `Reference` (#689) (Makoto Tateno)
- 75d928a syntax: Add loaded_modules to ModuleRecord (Boshen)

### Bug Fixes

- 2f48bdf parser,semantic: Make semantic own `Trivias` (#711) (Boshen)
- 815db57 semantic: Symbol of identifier of top level function declaration should be in the root scope (#843) (Yunfei He)
- d3accc1 semantic: Nested references (#661) (Don Isaac)

### Performance

- babbc47 parser: Lazily build trivia map instead of build in-place (#903) (Boshen)

### Testing

- 38fb4c2 semantic: Test harness (#679) (Don Isaac)

