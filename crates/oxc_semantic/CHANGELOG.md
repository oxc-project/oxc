# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.14.0] - 2024-06-05

### Bug Fixes

* transformer: JSX set `reference_id` on refs to imports (#3524)

### Refactor

* transformer/typescript: replace reference collector with symbols references (#3533)

## [0.13.3] - 2024-06-04

### Refactor

* traverse: `generate_uid` return `SymbolId` (#3520)

## [0.13.2] - 2024-06-03

### Bug Fixes

* linter: memorize visited block id in `neighbors_filtered_by_edge_weight` (#3407)
* semantic: set program scope_id for TS definition files (#3496)
* transformer: use UIDs in TS namespace transforms (#3395)

### Refactor

* ast: move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488)
* semantic: use a simpler way to resolve reference for ReferenceFlag::Type (#3430)- compile less test binaries to speed up CI (#3414) |

### Features

* linter/jsdoc: Implement require-returns rule (#3218)
* transformer: add `TraverseCtx::generate_uid` (#3394)

## [0.13.1] - 2024-05-22

### Refactor

* semantic: semantic populate `scope_id` fields in AST (#3303)
* semantic/cfg: alias petgraph's `NodeIndex` as `BasicBlockId`. (#3380)

## [0.13.0] - 2024-05-14

### Features

* linter/eslint: Implement max-classes-per-file (#3241)
* linter/jsdoc: Implement no-defaults rule (#3098)
* linter/react: add the `rules_of_hooks` rule. (#3071)
* semantic: report that enum member must have initializer (#3113)
* semantic: report namespace related errors (#3093)

### Refactor

* ast: squash nested enums (#3115)
* diagnostics: remove export of `miette`
* diagnostics: s/OxcDiagnostic::new/OxcDiagnostic::error
* semantic: clean up redeclaration diagnostic
* semantic: unify diagnostic in checker
* syntax: move number related functions to number module (#3130)- clean up more diagnostics usages |- remove all usages of `Into<Error>` |

### Bug Fixes

* semantic: add `cfg` nodes for `ConditionalExpression`s. (#3127)
* semantic: connect `test` expression of `for` statements to the cfg. (#3122)
* semantic: revert test code pushed to the main by accident. (#3085)
* semantic: allow `root_node` to be empty for empty trees. (#3084)

## [0.12.5] - 2024-04-22

### Bug Fixes

* semantic: correctly resolve identifiers inside catch parameter initializers (#3050)
* semantic: correctly resolve identifiers inside parameter initializers (#3046)

### Features

* ast: add `CatchParameter` node (#3049)
* semantic: add root node to the `AstNodes` structure. (#3032)

## [0.12.4] - 2024-04-19

### Bug Fixes

* semantic/jsdoc: Skip parsing `@` inside of backticks (#3017)

### Features

* semantic/jsdoc: Handle optional type syntax for type name part (#2960)

## [0.12.3] - 2024-04-11

### Refactor

* semantic/jsdoc: Rework JSDoc struct for better Span handling (#2917)

## [0.12.2] - 2024-04-08

### Bug Fixes

* semantic: symbols inside functions and classes incorrectly flagged as exported (#2896)

### Features

* linter: Implement jsdoc/check-access (#2642)

## [0.12.1] - 2024-04-03

### Bug Fixes

* semantic: flag function expressions with `SymbolFlags::Function` (#2891)

## [0.11.0] - 2024-03-30

### Bug Fixes

* linter/import: ignore export declaration in no-duplicates (#2863)
* semantic: missing SymbolFlags::Export when identifier used in ExportDefaultDeclaration (#2837)
* semantic: incorrect ExportEntry span for ExportAllDeclaration in ModuleRecord (#2793)
* semantic: ModuleRecord's indirect_export_entires missing reexported imports (#2792)

### Refactor

* semantic: distinguish whether requested_modules is type imports/exports (#2848)
* semantic/jsdoc: JSDocTag parser rework (#2765)

### Features

* semantic: distinguish type imports in ModuleRecord (#2785)
* semantic/jsdoc: Add `Span` for JSDoc, JSDocTag (#2815)

## [0.10.0] - 2024-03-14

### Refactor

* ast: refactor `Trivias` API - have less noise around it (#2692)
* ast: import `Tsify` to shorten code (#2665)
* ast: shorten manual TS defs (#2638)- remove unused dependencies (#2718) |- reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) |- "wasm" feature enable "serde" feature (#2639) |- make `CompactStr` immutable (#2620) |- rename `CompactString` to `CompactStr` (#2619) |

### Features

* semantic: move redeclare varaibles to symbol table (#2614)
* span: remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)- merge features `serde` and `wasm` to `serialize` (#2716) |

### Bug Fixes

* semantic/jsdoc: Support multibyte chars (#2694)
* semantic/jsdoc: Fix up builder (#2623)

## [0.9.0] - 2024-03-05

### Bug Fixes

* semantic: jsx reference with an incorrect node id (#2546)
* semantic: incorrect scope for switch statement (#2513)

### Refactor

* semantic/jsdoc: Misc fixes for JSDoc related things (#2531)- replace InlinableString with CompactString for `Atom` (#2517) |

### Features

* linter: remove all commonjs logic for import plugin (#2537)
* transformer: call build module record (#2529)

## [0.8.0] - 2024-02-26

### Bug Fixes

* linter: improve import/no-named-as-default (#2494)
* semantic: add export symbol flag to identifiers in export declarations (#2508)
* semantic: Should return nearest JSDoc (#2490)
* semantic: Refactor jsdoc finding (#2437)
* semantic: incorrect reference flag for MemberExpression assign (#2433)

### Features

* linter: handle cjs `module.exports = {} as default export (#2493)
* linter: handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
* linter: handle top-level `require` for import plugin (#2491)
* semantic: add check for duplicate class elements in checker (#2455)
* semantic: add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445)

### Refactor

* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: s/ArrowExpression/ArrowFunctionExpression to align estree
* semantic: delete the redundant code in binder (#2423)
* semantic: reduce allocation in resolve_references_for_current_scope (#2414)
* semantic: check directive by current_scope_id (#2411)- remove `panic!` from examples (#2454) |

### Performance

* semantic: reduce visit parent nodes in resolve_reference_usages (#2419)

## [0.7.0] - 2024-02-09

### Features

* ast: enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317)
* semantic: add export binding for ExportDefaultDeclarations in module record (#2329)
* semantic: report parameter related errors for setter/getter (#2316)
* semantic: report type parameter list cannot be empty (#2315)
* semantic: report unexpected type annotation in ArrayPattern (#2309)
* semantic: apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307)
* span: fix memory leak by implementing inlineable string for oxc_allocator (#2294)

### Bug Fixes

* semantic: remove unnecessary SymbolFlags::Import (#2311)
* semantic: remove ignore cases (#2300)

## [0.6.0] - 2024-02-03

### Features

* ast: remove serde skip for symbol_id and reference_id (#2220)
* ast: TypeScript definition for wasm target (#2158)
* linter: improve no_redeclare rule implementation (#2084)
* oxc_semantic: Improve sample visualization (#2251)
* semantic: report no class name error (#2273)
* semantic: check parameters property (#2264)
* semantic: check optional parameters (#2263)
* semantic: report error on optional variable declaration in TypeScript (#2261)
* semantic: track cfg index per ast node (#2210)
* semantic: cfg prototype (#2019)
* transformer/typescript: remove import if only have type reference (#2001)

### Bug Fixes

* codegen: print `Directive` original string (#2157)
* oxc_semantic: Handle short-circuiting operators in CFG (#2252)
* oxc_semantic: proper traversal of try statements (#2250)
* semantic: fix incorrect semantic example (#2198)
* semantic: replace ClassStatickBlockAwait with ClassStaticBlockAwait (#2179)
* semantic: incorrect reference flag (#2057)

### Refactor

* ast: rename RestElement to BindingRestElement (#2116)
* semantic: adding binder for ImportSpecifier replaces the ModuleDeclaration's binder (#2230)
* semantic: get function by scope_id in set_function_node_flag (#2208)
* semantic: checking label in ContinueStatement based on LabelBuilder (#2202)
* semantic: use LabelBuilder instead of UnusedLabeled (#2184)
* semantic: remove all #[dead_code[ from tester
* semantic: add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114)
* semantic: improve declare symbol logic in FormalParameters (#2088)- move all miette usages to `oxc_diagnostics` |

## [0.5.0] - 2024-01-12

### Features

* linter: support eslint/no-unused-private-class-members rule (#1820)
* playground: visualize symbol (#1886)
* playground: visualize scope (#1882)
* semantic: allow reserved keyword defined in ts module block (#1907)
* semantic: add current_scope_flags function in SemanticBuilder (#1906)
* semantic: improve check super implementation, reduce access nodes (#1827)
* semantic: support get node id by scope id (#1826)
* semantic: add ClassTable (#1793)
* semantic: add SymbolFlags::Function for FunctionDeclaration (#1713)
* transform: support es2015 new target (#1967)

### Refactor

* semantic: improve ClassTable implmention and merge properties and methods to elements (#1902)
* semantic: improve check function declaration implementation (#1854)
* semantic: rename `add_node_id` to `add_current_node_id_to_current_scope` (#1847)
* semantic: improve check private identifier implementation (#1794)
* semantic: remove unused methods from `AstNode`

### Performance

* linter/react: find class node by symbols in get_parent_es6_component (#1657)
* semantic: check duplicate parameters in Binder of FormalParameters (#1840)
* semantic: just need to find the AstKind::FormalParameter in is_in_formal_parameters (#1852)
* semantic: reduce calls to span() (#1851)

### Bug Fixes

* semantic: remove duplicate errors in ModuleDeclaration::ImportDeclaration (#1846)

## [0.4.0] - 2023-12-08

### Features

* ast: add to_string function to VariableDelcartionKind (#1303)
* ast/semantic: parse jsdoc on `PropertyDefinition` (#1517)
* prettier: add the basics of comment printing (#1313)
* semantic: support scope descendents starting from a certain scope. (#1629)

### Refactor

* ast: VariableDeclarationKind::to_string -> as_str (#1321)
* rust: move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

* linter: add no-redeclare rule. (#683)
* minifier: re-enable mangler (#972)
* minifier: partially re-enable minifier (#963)
* semantic: bind function expression name (#1049)
* semantic: check non-simple lhs expression of assignment expression (#994)
* transformer: implement some of needs_explicit_esm for typescript (#1047)
* transformer: add utils to make logical_assignment_operators pass (#1017)
* transformer: ES2020 Nullish Coalescing Operator (#1004)
* transformer: finish 2016 exponentiation operator (#996)
* transformer/react: read comment pragma @jsxRuntime classic / automatic (#1133)

### Refactor

* ast: fix the lifetime annotations around Vist and VisitMut (#973)
* transformer: move Semantic into Transformer (#1130)

### Testing

* semantic: add scoping test cases (#954)

### Bug Fixes

* semantic: make ExportDeclaration span accurate (#928)

## [0.2.0] - 2023-09-14

### Performance

* parser: lazily build trivia map instead of build in-place (#903)

### Features

* ast: add `SymbolId` and `ReferenceId` (#755)
* linter: implement re-exports (#877)
* linter: add runner for import-plugin (#858)
* semantic: add `node_id` to `Reference` (#689)
* syntax: add loaded_modules to ModuleRecord

### Bug Fixes

* parser,semantic: make semantic own `Trivias` (#711)
* semantic: symbol of identifier of top level function declaration should be in the root scope (#843)
* semantic: nested references (#661)

### Testing

* semantic: test harness (#679)

