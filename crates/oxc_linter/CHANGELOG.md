# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.4.2] - 2024-05-28

### Features

* lint/eslint: implement require-await (#3406)
* linter: add `oxc/no-rest-spread-properties` rule (#3432)
* linter: add `oxc/no-const-enum` rule (#3435)
* linter: add `oxc/no-async-await` rule (#3438)
* linter: eslint-plugin-jest/require-top-level-describe (#3439)
* linter: eslint-plugin-jest/prefer-hooks-on-top (#3437)
* linter: @typescript-eslint/consistent-indexed-object-style (#3126)
* linter/eslint: Implement no-div-regex (#3442)
* linter/eslint: Implement no-useless-concat (#3363)

### Bug Fixes

* linter: memorize visited block id in `neighbors_filtered_by_edge_weight` (#3407)
* linter: Accept more valid regex (#3408)
* website: hack `schemars` to render code snippet in markdown (#3417)

### Documentation

* linter: add docs for consistent-indexed-object-style (#3409)

## [0.4.0] - 2024-05-24

### Features

* cli,linter: add `--disable-oxc-plugin` (#3328)
* cli,linter: add `--disable`-react/unicorn/typescript-`plugin` (#3305)
* linter: temporary move react/require-render-return to nursery
* linter: eslint/no-restricted-globals (#3390)
* linter: change jsdoc/require-returns from correctness to pedantic
* linter: change jsdoc/require-render-return from correctness to pedantic
* linter: start adding json schema for configuration file (#3375)
* linter: eslint-plugin-jest/no-duplicate-hooks (#3358)
* linter: backward compability for `react-hooks` and `deepscan` plugins (#3334)
* linter/eslint: Implement default_case rule (#3379)
* linter/eslint: Implement no-new (#3368)
* linter/eslint: Implement prefer-exponentiation-operator (#3365)
* linter/eslint: Implement symbol-description (#3364)
* linter/jsdoc: Implement require-returns-description (#3397)
* linter/jsdoc: Implement require-returns rule (#3218)
* tasks/website: code generate the linter rules
* tasks/website: start generating linter config markdown from json schema (#3386)
* website: generate linter configuration page

### Refactor

* diagnostics: s/warning/warn
* linter: rename variable names prefix `ESLint` to `Oxlint`
* linter: remove unnecessary check in `eslint/no-global-assign` (#3391)
* linter: find return statement by using CFG in `react/require-render-return` (#3353)
* linter: remove `with_rule_name` from the tight loop (#3335)
* linter: merge deepscan rules into oxc rules (#3327)
* semantic/cfg: alias petgraph's `NodeIndex` as `BasicBlockId`. (#3380)

### Bug Fixes

* linter: `no-new` false positive when return from arrow expression (#3393)
* linter: only report issues on top-level fragment (#3389)
* linter: avoid infinite loop in `jest/expect-expect` (#3332)
* linter: avoid infinite loop when traverse ancestors in `jest/no_conditional_expect` (#3330)
* linter: fix panic in jest/expect-expect (#3324)
* linter/jsx-no-undef: check for globals when an identifier is undefined (#3331)
* linter/next: false positives for non-custom font link (#3383)
* linter/react: fix false positives for async components in `rules_of_hooks` (#3307)
* linter/react: better detection for hooks in the `rules_of_hooks`. (#3306)
* linter/react: `rules_of_hooks` add support for property hooks/components. (#3300)
* linter/react: `rules_of_hooks` resolve false positives with conditional hooks. (#3299)
* linter/react: fix loop hooks false positives. (#3297)

### Performance

* linter: use `usize` for `RuleEnum` hash (#3336)

## [0.3.5] - 2024-05-15

### Bug Fixes

* linter/no-direct-mutation-state: false positive when class is declared inside a `CallExpression` (#3294)

### Refactor

* linter: rewrite react/require-render-return (#3276)

### Features

* linter: add use-isnan fixer for (in)equality operations (#3284)
* linter/eslint: Implement fixer for unicode-bom rule (#3259)

## [0.3.4] - 2024-05-13

### Features

* linter: move react/rules_of_hooks to nursery
* linter/eslint: Implement max-classes-per-file (#3241)

## [0.3.3] - 2024-05-13

### Bug Fixes

* linter: handle `import { default as foo }` in import/named (#3255)
* linter/default: ignore unsupported files (e.g. .vue)
* parser: correctly parse cls.fn<C> = x (#3208)

### Features

* linter: demote `no-inner-declarations` from correctness to pedantic (eslint v9)
* linter: demote `react/jsx-no-useless-fragment` from correctness to pedantic
* linter: unicorn/no-anonymous-default-export (#3220)
* linter: add `radix` rule (#3167)
* linter: eslint-plugin-next/no-page-custom-font (#3185)
* linter: remove deprecated eslint v9 rules `no-return-await` and `no-mixed-operators` (#3188)
* linter: eslint/no-new-native-nonconstructor (#3187)
* linter: eslint-plugin-next/no-styled-jsx-in-document (#3184)
* linter: eslint-plugin-next/no-duplicate-head (#3174)
* linter/eslint: Implement unicode-bom rule (#3239)
* linter/eslint: Implement no-empty-function rule (#3181)
* linter/import: improve multiple exports error message (#3160)
* linter/react: add the `rules_of_hooks` rule. (#3071)
* linter/tree-shaking: add `isPureFunction` (#3175)

### Refactor

* diagnostics: remove export of `miette`
* diagnostics: remove thiserror
* diagnostics: s/OxcDiagnostic::new/OxcDiagnostic::error
* linter: clean up diagnostics
* linter: clean up diagnostics in fixer
* linter: remove unnecessary usages of `CompactStr`
* linter: reduce llvm lines generated by `RuleEnum::read_json` (#3207)
* linter: clean up prefer_node_protocol and move to restriction (#3171)
* linter,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3235)
* parser,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3214)- clean up more diagnostics usages |

## [0.3.2] - 2024-05-04

### Features

* linter: @typescript-eslint/prefer-literal-enum-member (#3134)
* linter: add more "ban-ts-comment" test cases. (#3107)
* linter: eslint-plugin-jest/require-hook (#3110)
* linter: typescript-eslint/prefer-enum-initializers (#3097)
* linter: eslint/no-await-in-loop (#3070)
* linter/import: move some rules out of nursery (#2841)
* linter/jsdoc: Implement require-yields rule (#3150)
* linter/jsdoc: Support settings.ignore(Private|Internal) (#3147)
* linter/jsdoc: Implement no-defaults rule (#3098)
* linter/jsdoc: Implement `implements-on-classes` rule (#3081)
* linter/jsdoc: Implement check-tag-names rule (#3029)
* linter/tree-shaking: support While/Switch/Yield Statement (#3155)
* linter/tree-shaking: support SequenceExpression (#3154)
* linter/tree-shaking: support UnaryExpression (#3153)
* linter/tree-shaking: support JSX (#3139)
* linter/tree-shaking: support import statement (#3138)
* linter/tree-shaking: support ForStatement (#3078)
* linter/tree-shaking: support ExportNamedDeclaration (#3072)
* linter/tree_shaking: support LogicExpression and MemberExpression (#3148)

### Bug Fixes

* linter: handle named export default in import-plugin/named (#3158)
* linter: fix hang if a file fails to parse while using `--import-plugin`
* semantic: revert test code pushed to the main by accident. (#3085)
* semantic: allow `root_node` to be empty for empty trees. (#3084)

### Refactor

* ast: squash nested enums (#3115)
* ast: add array element `Elision` type (#3074)
* linter: render `--rules` in a table
* linter/jsdoc: Misc improvements (#3109)
* syntax: move number related functions to number module (#3130)
* syntax: use `FxHashMap` for `ModuleRecord::request_modules` (#3124)

## [0.3.1] - 2024-04-22

### Bug Fixes

* linter: fix unwanted plugin rules being enabled

## [0.3.0] - 2024-04-22

### Bug Fixes

* linter: support `-D all -D nursery`
* linter: fix crashing with `unwrap` in import/no-cycle (#3035)

### Features

* ast: add `CatchParameter` node (#3049)
* linter: --deny all should not enable nursery rules
* linter: implement fixer for `typescript-eslint/consistent-type-definitions` (#3045)
* linter: remove all ESLint Stylistic rules
* linter: change no-empty-static-block to `correctness`
* linter: no barrel file. (#3030)
* linter: support eslint globals (#3038)
* linter/tree-shaking: support `ExportDefaultDeclaration` (#3052)

### Refactor

* linter: improve the ergonomics around `ESlintConfig` (#3037)
* linter/import/no_cycle: use ModuleGraphVisitor. (#3064)

### Performance

* ast: box typescript enum variants. (#3065)
* ast: box enum variants (#3058)

## [0.2.18] - 2024-04-19

### Features

* linter: support `oxlint-disable` alongside `eslint-disable` (#3024)
* linter: remove import/no-unresolved (#3023)
* linter: eslint/max-len (#2874)
* linter: Implement plugin-jsdoc/check-property-names (#2989)
* linter: add missing test cases to no-empty-interface and add config (#2973)
* linter: Add --jsdoc-plugin flag (#2935)
* linter/jsdoc: Update settings.jsdoc method (#3016)
* linter/jsdoc: Implement require-property-(type|name|description) rules (#3013)
* linter/jsdoc: Implement require-property rule (#3011)
* linter/tree-shaking: support DoWhileStatement and IfStatement (#2994)
* linter/tree-shaking: support ConditionalExpression (#2965)
* linter/tree-shaking: support Class (#2964)

### Bug Fixes

* linter/no-empty-interface: add missing test (#2979)

## [0.2.17] - 2024-04-11

### Features

* linter: eslint-plugin-jest/prefer-lowercase-title (#2911)
* linter: typescript-eslint/consistent-type-definitions (#2885)
* linter/tree-shaking: support part BinaryExpression (#2922)

### Refactor

* semantic/jsdoc: Rework JSDoc struct for better Span handling (#2917)

### Bug Fixes

* linter: import/no-cycle ignore type-only imports (#2924)

## [0.2.16] - 2024-04-08

### Features

* linter: @typescript-eslint/prefer-for-of (#2789)
* linter: Implement jsdoc/check-access (#2642)
* linter: Implement jsdoc/empty-tags (#2893)
* linter: eslint-plugin-jest/prefer-mock-promise-sorthand (#2864)
* linter/import: Add `ignoreTypes` option for the `import/no-cycle` rule (#2905)
* linter/tree-shaking: support try-catch and AwaitExpression (#2902)
* linter/tree-shaking: check `this` in different environment (#2901)
* linter/tree-shaking: support ThisExpression and NewExpression (#2890)
* linter/tree-shaking: support ArrowFunctionExpression (#2883)
* linter/tree-shaking: support `ArrayExpression` and `ArrayPattern`  (#2882)

### Bug Fixes

* ast: `FinallyClause` won't get visited as `BlockStatement` anymore. (#2881)
* linter: handle self closing script tags in astro partial loader (#2017) (#2907)
* linter: svelte partial loader handle generics (#2875) (#2906)

## [0.2.15] - 2024-03-30

### Bug Fixes

* linter/import: ignore export declaration in no-duplicates (#2863)
* linter/import: false positive for indirect export in namespace (#2862)
* linter/max-lines: only report codes that exceed the line limit (#2778)

### Features

* cli: add tsconfig file validation in LintRunner (#2850)
* linter: fallback to the default tsconfig path (#2842)
* linter: eslint-plugin-jest/prefer-comparison-matcher (#2806)
* linter: eslint-plugin-jest/no-untyped-mock-factory (#2807)
* linter: eslint/no-iterator (#2758)
* linter: eslint-plugin-react checked-requires-onchange-or-readonly (#2754)
* linter: default_param_last (#2756)
* linter: no_script_url (#2761)
* linter/import: ignore type-only imports and exports in no_unresolved (#2849)
* linter/tree-shaking: pass CallExpression cases (#2839)
* linter/tree-shaking: check CallExpression when called (#2809)
* linter/tree-shaking: detect CallExpression in MemberExpression (#2772)

### Refactor

* semantic: distinguish whether requested_modules is type imports/exports (#2848)
* sourcemap: change sourcemap name to take a reference (#2779)

## [0.2.14] - 2024-03-19

- **BREAKING** ast: rename BigintLiteral to BigIntLiteral (#2659)

### Features

* linter: no_template_curly_in_string (#2763)
* linter: eslint/no-proto (#2760)
* linter: no_eq_null (#2757)
* linter: eslint/max-params (#2749)
* linter: eslint/guard-for-in (#2746)
* linter: eslint/no-ternary (#2744)
* linter: eslint/no-continue (#2742)
* linter: eslint/no-with (#2741)
* linter: eslint/max-lines (#2739)
* linter: eslint-plugin-jest: `prefer-to-contain` (#2735)
* linter: eslint-plugin-jest: `prefer-expect-resolves` (#2703)
* linter: Add settings.jsdoc (#2706)
* linter: eslint-plugin-jest: prefer-to-be (#2702)
* linter: eslint-plugin-jest: prefer-spy-on (#2666)
* linter: report side effect for array element in node_side_effects rule (#2683)
* linter: resolve ESM star exports (#2682)
* linter: support check ImportNamespaceSpecifier in no_import_assign (#2617)
* linter: change ban-ts-comment to pedantic
* linter/import: check ObjectPattern syntax in namespace (#2691)
* linter/import: support check reexport binding in namespace (#2678)
* linter/jest: add new property for `parse_jest_fn` (#2715)
* linter/tree-shaking: add cache for checking mutating identifiers (#2743)
* linter/tree_shaking: check assignment of identifier  (#2697)
* semantic: move redeclare varaibles to symbol table (#2614)
* span: `impl<'a> PartialEq<str> for Atom<'a>` (#2649)
* span: remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)
* task: init eslint-plugin-tree-shaking rule (#2662)- miette v7 (#2465) |

### Bug Fixes

* linter: fix guard_for_in span error (#2755)
* linter: correct example for no-obj-calls rule (#2618)
* parser: parse named rest element in type tuple (#2655)

### Refactor

* ast: refactor `Trivias` API - have less noise around it (#2692)
* lint: split files for no_side_effects rule (#2684)
* linter: improve the implementation of no_shadow_restricted_names based on symbols (#2615)
* parser: improve parsing of `BindingPattern` in TypeScript (#2624)
* span: disallow struct expression constructor for `Span` (#2625)- make `CompactStr` immutable (#2620) |- rename `CompactString` to `CompactStr` (#2619) |

## [0.2.13] - 2024-03-05

### Features

* linter: eslint-plugin-jest: prefer-to-have-length (#2580)
* linter: eslint-plugin-jest: prefer-strict-equal (#2581)
* linter/import: partial support namespace check (#2538)

### Bug Fixes

* linter: avoid crash if no members in TSTypeLiteral in typescript/prefer-function-type (#2604)
* linter: exclude typescript syntax function in only_used_in_recursion (#2595)
* linter: fix getter return rule false positives in TypeScript (#2543)
* parser: fix span start for TSModuleDeclaration (#2593)
* semantic: jsx reference with an incorrect node id (#2546)- broken build from codegen API change |

### Refactor

* codegen: clean up API around building sourcemaps (#2602)
* semantic/jsdoc: Misc fixes for JSDoc related things (#2531)

## [0.2.12] - 2024-02-28

### Features

* ast: add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536)
* cli,linter: provide tsconfig path from the cli (#2526)
* linter: remove all commonjs logic for import plugin (#2537)

## [0.2.11] - 2024-02-26

### Bug Fixes

* linter: Correct configuration file parsing for jsx-no-useless-fragment (#2512)
* linter: improve import/no-named-as-default (#2494)
* linter: fix import plugin hanging when ignored modules are imported (#2478)
* linter: Handle cases where createElement is an Identifier in is_create_element_call (#2474)
* semantic: Refactor jsdoc finding (#2437)

### Refactor

* ast: remove `TSEnumBody` (#2509)
* ast: s/TSThisKeyword/TSThisType to align with estree
* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: s/ArrowExpression/ArrowFunctionExpression to align estree

### Features

* linter: handle cjs `module.exports = {} as default export (#2493)
* linter: handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
* linter: handle top-level `require` for import plugin (#2491)
* linter: implement @typescript-eslint/prefer-ts-expect-error (#2435)
* linter: initialize resolver lazily and automatically read tsconfig.json for now (#2482)
* linter: ignore unsupported extensions in import/no_unresolved (#2481)
* linter: handle built-in modules in import/no_unresolved (#2479)
* linter: eslint-plugin-react void-dom-elements-no-children (#2477)
* linter: add boilerplate for eslint-plugin-import/no_duplicates (#2476)
* linter: eslint-plugin-import/no_unresolved (#2475)
* linter: continue working on no_cycle (#2471)
* linter: add boilerplace code for import/namespace,no_deprecated,no_unused_modules (#2470)
* linter: typescript-eslint: prefer-function-type (#2337)

## [0.2.10] - 2024-02-21

### Features

* codegen: configurable typescript codegen (#2443)
* linter: eslint no-nonoctal-decimal-escape (#2428)

### Refactor

* linter: simplify getting ImportDefaultSpecifier (#2453)
* linter: improve implementation of no_dupe_class_members based on ClassTable (#2446)- remove `panic!` from examples (#2454) |

### Bug Fixes

* semantic: incorrect reference flag for MemberExpression assign (#2433)

## [0.2.9] - 2024-02-18

### Refactor

* linter: get arrow expression by scope_id in no_render_return_value (#2424)
* linter/config: Use serde::Deserialize for config parsing (#2325)

### Features

* linter: Implement `unicorn/no-process-exit` rule (#2410)
* linter: detect jest file by default glob pattern (#2408)
* linter: eslint-plugin-jest require-to-throw-message (#2384)
* linter: eslint-plugin-jest: prefer-equality-matcher (#2358)
* semantic: add export binding for ExportDefaultDeclarations in module record (#2329)- add Typescript ban-tslint-comment (#2371) |

### Bug Fixes

* linter: `getter-return` false positive with TypeScript syntax (#2363)
* linter: add missing typescript-eslint(_) prefix for some errors (#2342)
* linter/jsx_a11y: Refactor jsx-a11y related utils and its usage (#2389)
* linter/jsx_a11y: Ensure plugin settings are used (#2359)

### Performance

* lint/no_var_requires: quicker way to check if the `IdentifierReference` point to a global variable (#2376)

## [0.2.8] - 2024-02-06

### Features

* ast: enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317)
* linter: promote `no-this-before-super` to correctness (#2313)
* linter: Implement no_this_before_super with cfg (#2254)
* semantic: apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307)- add typescript-eslint rule array-type (#2292) |

### Refactor

* ast: fix BigInt memory leak by removing it (#2293)

### Bug Fixes

* linter: fix no_dupe_keys false postive on similar key names (#2291)

## [0.2.7] - 2024-02-03

### Bug Fixes

* lexer: correct the span for irregular whitespaces (#2245)
* linter: AllowFunction doesn't support generator (#2277)
* linter: ban `--fix` for variety files(vue, astro, svelte) (#2189)
* linter: jsx no undef match scope should check with ancestors (#2027)
* oxc_semantic: proper traversal of try statements (#2250)

### Refactor

* linter: remove Regex and change error position (#2188)- use our forked version of miette::Reporter for tests (#2266) |- move all miette usages to `oxc_diagnostics` |

### Features

* ast: remove generator property from ArrowFunction (#2260)
* linter: complete custom components setting (#2234)
* linter: implement @next/next/no-before-interactive-script-outsi… (#2203)
* linter: implement @next/next/no-unwanted-polyfillio (#2197)
* semantic: track cfg index per ast node (#2210)

## [0.2.6] - 2024-01-26

### Refactor

* linter: move settings and env to the config module (#2181)

### Bug Fixes

* linter: Rename react_perf/jsx_no_new_function_as_props to jsx_no_new_function_as_prop (#2175)

### Features

* linter: support read env from eslintrc (#2130)
* semantic: cfg prototype (#2019)
* transfrom: transform-json-strings (#2168)

## [0.2.5] - 2024-01-25

### Bug Fixes

* codegen: print `Directive` original string (#2157)
* linter: use correct rule name (#2169)
* linter: explicit-length-check inside ternary (#2165)

### Features

* linter: eslint-plugin-jest: prefer-called-with (#2163)
* linter: eslint: no-void (#2162)

## [0.2.3] - 2024-01-23

### Features

* linter: linter-eslint-plugin-import/no-named-as-default (#2109)
* linter: promote no-new-array to correctness with better help message (#2123)
* linter: eslint config jsonc support (#2121)
* linter: eslint-plugin-react-perf (#2086)
* linter: support eslint config in nextjs eslint (#2107)
* linter: eslint-plugin-jest: no-restricted-jest-methods (#2091)- introduce --react-perf-plugin CLI flag, update rules to correctness (#2119) |- (eslint-plugin-jest): no-restricted-matchers (#2090) |

### Bug Fixes

* linter: allow `[...new Array(n)]` in no-useless-spread (#2124)
* linter: jsx_a11y/img-redundant linter enable test case(#2112)
* linter: not use `new_inline` with flexible str (#2106)

### Refactor

* ast: rename RestElement to BindingRestElement (#2116)
* semantic: add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114)

## [0.2.2] - 2024-01-20

### Refactor

* linter: perfect the scope linter (#2092)

### Features

* linter: improve no_redeclare rule implementation (#2084)- expose linter RULES and use it for listing (#2083) |

### Bug Fixes

* linter: eslint-plugin-import no-named-as-default-member rule (#2071)
* linter: s/consistent-type-export/consistent-type-exports (#2065)

## [0.2.1] - 2024-01-16

### Refactor

* linter: move `LintSettings` to its own file (#2052)
* linter: remove the `LintSettings` parameter from `LintContext::new`. (#2051)
* linter: move away from tuples for test cases (#2011)

### Features

* linter: remove the `--timings` feature (#2049)
* linter: eslint-plugin-import no-named-as-default-member rule (#1988)
* linter: eslint-plugin-jsx-a11y no-redundant-roles rule (#1981)
* linter: eslint-plugin-jsx-a11y aria-activedescendant-has-tabindex (#2012)
* linter: eslint-plugin-next: no-document-import-in-page (#1997)
* linter: eslint-plugin-next: no-head-element (#2006)
* linter: eslint-plugin-next:  no-typos (#1978)
* linter: eslint-plugin-jsx-a11y click-events-have-key-events (#1976)

### Bug Fixes

* linter: false positive for filename_case where filename doesn't have a proper casing (#2032)
* linter: keep rules disabled if the rule is not enabled in the config (#2031)
* linter: fix false positive for `erasing-op` in `0/0` case (#2009)

## [0.2.0] - 2024-01-12

### Features

* linter: eslint-plugin-jest: no-test-return-statement (#1979)
* linter: add support for same rule name but different plugin names (#1992)
* linter: support vue generic component (#1989)
* linter: implement @typescript-eslint/triple-slash-reference (#1903)
* linter: eslint-plugin-jsx-a11y autocomplete-valid (#1901)
* linter: eslint-plugin-react: no-direct-mutation-state (#1892)
* linter: support overriding oxlint rules by eslint config (#1966)
* linter: eslint-plugin-react: require-render-return (#1946)
* linter: eslint-plugin-jsx-a11y role-has-required-aria-props (#1881)
* linter: eslint-plugin-jsx-a11y role-support-aria-props (#1961)
* linter: eslint-plugin-jsx-a11y role-support-aria-props (#1949)
* linter: eslint-plugin-react: no-unknown-property (#1875)
* lsp: support vue, astro and svelte (#1923)- nextjs plugin (#1948) |

### Bug Fixes

* linter: allow eslintrc to add rule when overriding (#1984)
* linter: jsx-key: handle anonymous functional components in arrays that have a function body (#1983)
* linter: fix plugin name parsing when reading config file (#1972)
* linter: Support cases where aria-hidden includes expressions (#1964)
* linter: change severity of no-sparse-arrays to warnings

### Refactor

* formatter,linter,codegen: remove oxc_formatter (#1968)
* linter: remove duplicate `get_jsx_attribute_name` (#1971)

## [0.1.2] - 2024-01-06

### Features

* linter: disable no-unused-labels for svelte (#1919)
* linter: <script> part of svelte file (#1918)

### Refactor

* linter: rename *_partial_loader files (#1916)

### Bug Fixes

* linter: change no-var to restriction

## [0.1.1] - 2024-01-06

### Bug Fixes

* linter: fix vue parser not working for multiple scripts after <template> (#1904)
* linter: do not check empty file for vue / astro files (#1900)
* linter: error rule config in media_has_caption (#1864)
* linter: unexpected unwrap panic (#1856)
* linter: ignore false positives in eslint-plugin-react(jsx-key) (#1858)

### Features

* cli: support walk vue and astro (#1745)
* lint: add partial loader register (#1760)
* linter: eslint: no-var (#1890)
* linter: parse two script tags from vue (#1899)
* linter: parse multiple script tags in astro file (#1898)
* linter: add support for multiple script tags from vue and stro (#1897)
* linter: no irregular whitespace (#1877)
* linter: support astro front matter `---` block (#1893)
* linter: do not lint when vue file has no js section (#1891)
* linter: eslint-plugin-jsx-a11y prefer-tag-over-role (#1831)
* linter: eslint-plugin-jsx-a11y mouse-events-have-key-events (correctness) (#1867)
* linter: add Vue loader (#1814)
* linter: eslint-plugin-react: jsx-no-undef (#1862)
* linter: eslint plugin jsx a11y: aria-role (#1849)
* linter: use settings for eslint-plugin-jsx-a11y/html_has_lang (#1843)
* linter: support eslint/no-unused-private-class-members rule (#1820)
* linter: eslint-plugin-jsx-a11y media-has-caption (#1822)
* linter: Refine test for no-distracting-elements (#1824)
* linter: refine jsx-a11y settings (#1816)
* linter: eslint-plugin-jsx-a11y lang (#1812)

### Refactor

* linter: get js code slice from vue source code (#1876)
* linter: extract common code (#1848)
* linter: Simplify Parent Node Access in MediaHasCaption Rule (#1829)
* semantic: improve ClassTable implmention and merge properties and methods to elements (#1902)

### Performance

* semantic: check duplicate parameters in Binder of FormalParameters (#1840)

## [0.0.22] - 2023-12-25

### Features

* ast: enter/leave ClassBody and PrivateInExpression (#1792)
* linter: change double-comparisons to correctness
* linter: eslint-plugin-jsx-a11y aria-props (#1797)
* linter: eslint-plugin-jsx-a11y no-aria-hidden-on-focusable (#1795)
* linter: eslint-plugin-jsx-a11y no-distracting-elements rule (#1767)
* linter: correct example and docs url for number_arg_out_of_range (#1737)
* linter/eslint/no-cond-assign: span points to the operator (#1739)
* linter/eslint/no-useless-escape: support auto fix (#1743)

### Bug Fixes

* linter: fix a typo in no_redeclare message (#1789)
* linter: support read the third item in config file (#1771)
* linter: update snapshots
* linter: change non-error lints to warning
* linter: improve the help message for const-comparisons (#1764)
* linter: fix missing ` in the help message for const-comparisons
* linter/eslint/no-obj-calls: correctly resolves the binding name (#1738)

### Performance

* linter: reduce the `RuleEnum` enum size from 168 to 16 bytes (#1783)
* linter: use simd (memchr) for no-useless-escape search (#1766)
* linter: change regex to static in no_commented_out_tests
* linter: precompute `rule.name()` (#1759)

### Documentation

* linter: update comments (#1779)

### Refactor

* linter: shrink the error span for require_yield
* linter: explain no-empty-pattern

## [0.0.21] - 2023-12-18

### Features

* linter: eslint-plugin-jsx-a11y no-access-key (correctness) for  (#1708)
* linter: eslint-plugin-unicorn no-null(style) (#1705)
* linter: add  jsx-a11y settings (#1668)
* linter: `tabindex-no-positive` for eslint-plugin-jsx-a11y (#1677)
* linter: eslint-plugin-unicorn/prefer-prototype-methods (#1660)
* linter: add eslint-plugin-import(export) rule (#1654)
* linter: eslint-plugin-unicorn prefer-dom-node-text-content(style) (#1658)

### Refactor

* linter: use fxHashMap in jsx-a11y settings (#1707)
* linter: make some jest rules report more detailed (#1666)- use `new_without_config` for `jsx_key` (#1685) |

### Bug Fixes

* linter: prefer-string-starts-ends-with: ignore `i` and `m` modifiers. (#1688)
* linter: Panic in prefer string starts, ends with (#1684)
* linter: fix excape_case panicing on unicode strings (#1673)

### Performance

* linter/react: find class node by symbols in get_parent_es6_component (#1657)

## [0.0.20] - 2023-12-13

### Refactor

* linter: separate out the category in the output of `--rules`

### Bug Fixes

* linter: improve the span message for no-accumulating-spread- improve span for no accumulating spread (#1644) |- remove escapes in no array reduce test cases (#1647) |- remove escapes in prefer regexp test test cases (#1645) |

### Features

* linter: eslint-plugin-unicorn prefer-modern-dom-apis(style) (#1646)

## [0.0.19] - 2023-12-08

### Bug Fixes

* linter: improve the key span for jsx-key

### Features

* linter: eslint-plugin-jsx-a11y no-autofocus  (#1641)
* linter: eslint-plugin-jsx-a11y scope rule (correctness) (#1609)
* linter: cxc: no accumulating spread (#1607)
* linter: eslint-plugin-unicorn: explicit-length-check (#1617)
* linter: eslint-plugin-unicorn prefer-reflect-apply(style) (#1628)
* linter: add a `perf` category (#1625)
* linter: eslint-plugin-jsx-a11y iframe-has-title rule (correctness) (#1589)
* linter: eslint-plugin-unicorn require-array-join-separator(style) (#1608)
* linter: eslint-plugin-unicorn no-unreadable-array-destructuring (style) (#1594)
* linter: eslint-plugin-jsx-a11y  img-redundant-alt (correctness) (#1571)
* linter: eslint-plugin-unicorn numeric-separators-style (style) (#1490)
* linter: eslint-plugin-unicorn/no-unreadable-iife (#1572)
* linter: eslint-plugin-unicorn no-await-expression-member (style) (#1569)
* linter: eslint-lugin-unicorn no_useless_length_check (#1541)
* linter: no-is-mounted for eslint-plugin-react (#1550)
* linter: eslint 9.0 no empty static block (#1543)
* linter: eslint-plugin-unicorn: escape-case (#1495)
* linter: heading-has-content for eslint-plugin-jsx-a11y (#1501)
* linter: eslint-plugin-unicorn prefer-set-size (correctness) (#1508)
* linter: eslint-plugin-unicorn prefer-native-coercion-functions (pedantic) (#1507)
* linter: eslint-plugin-jsx-a11y anchor_is_valid (correctness) (#1477)- eslint-plugin-unicorn (recommended) prefer-node-protocol (#1618) |

### Refactor
- improve pattern match of prefer-reflect-apply (#1630) |

## [0.0.18] - 2023-11-22

### Features

* linter: eslint plugin unicorn: no useless switch case (#1463)
* linter: html-has-lang for eslint-plugin-jsx-a11y (#1436)
* linter: `anchor-has-content` for eslint-plugin-jsx-a11y
* linter: eslint-plugin-unicorn/no-nested-ternary (#1417)
* linter: for-direction rule add check for condition in reverse o… (#1418)
* linter: eslint-plugin-unicorn: no-hex-escape (#1410)
* linter: eslint-plugin-jest: no-deprecated-function (#1316)
* linter: Add rule `eslint(no_regex_spaces)` (#1129)
* linter: eslint-plugin-unicorn/number-literal-case (#1271)
* linter: eslint-plugin-jest/max_expects (#1239)
* linter: reimplement eslint-plugin-jest(no-identical-title) (#1229)
* linter: eslint-plugin-unicorn/no-abusive-eslint-disable (#1125)
* prettier: print leading comments with newlines (#1434)

### Refactor

* lint: replace `parse_jest_fn_*` methods in eslint-plugin-jest(no-standalone-expect) rule (#1231)
* lint: migrate eslint-plugin-jest(expec-expect) (#1225)
* linter: replace the old parse_expect_jest_fn.rs file (#1267)
* linter: remove all old `parse_expect_jest_fn_call` (#1259)
* linter: remove all old `parse_general_jest_fn_call` in jest rules (#1232)
* linter: replace all `is_type_of_jest_fn_call` (#1228)
* linter: migrate eslint-plugin-jest(no-alias-method) (#1226)
* linter: remove unused logic in `resolve_to_jest_fn` (#1208)
* rust: move to workspace lint table (#1444)

### Bug Fixes

* linter: detect assert function in Await Expression (#1202)

## [0.0.17] - 2023-11-09

### Bug Fixes

* linter: fix handling of repeated eslint-disable comments (#1200)

## [0.0.16] - 2023-11-08

### Refactor

* linter: reduce the lookup times of Call Expression in Jest rules (#1184)- change jest rule's category (#1155) |- split parse_jest_fn_call (#1152) |

### Features

* lint: remove unnecessary check (#1185)
* linter: eslint-plugin-jest: no-hooks (#1172)
* linter: support eslint(default-case-last) (#1156)
* linter: eslint-plugin-unicorn no-object-as-default-parameter (#1162)
* linter: jest/prefer-todo rule (#1065)
* linter: Add  rule `eslint-plugin-jsx-a11y(alt-text)` (#1126)- basic enable plugin (#1154) |

### Bug Fixes

* linter: fix covered span of eslint-disable-next-line comments (#1128)
* linter/jsx_key: ignore ObjectProterty nodes (#1139)

## [0.0.15] - 2023-10-30

### Features

* linter: change some rules pedantic and improve help message (#1112)
* linter: demote prefer_array_flat_map to style (#1108)
* linter: support unicorn/prefer-query-selector (#1068)
* linter: eslint-plugin-unicorn require-number-to-fixed-digits-argument (#1073)
* linter: eslint-plugin-unicorn  switch-case-braces (#1054)
* linter: support react/no-string-refs (#1055)
* linter: eslint-plugin-unicorn - no-empty-file (#1052)
* linter: eslint-plugin-react no-string-refs (#1053)
* linter: support react/no-render-return-value (#1042)
* linter: eslint-plugin-react(no-unescaped-entities) (#1044)
* linter: eslint-plugin-react/no-find-dom-node (#1031)
* linter/no_children_prop: point the span to "children" (#1106)
* transformer: implement some of needs_explicit_esm for typescript (#1047)

### Bug Fixes

* ast: jsx attribute value and text child should be jsx string (#1089)
* linter: fix panic in no_unescaped_entities (#1103)
* linter: revert changes to JSX attribute strings (#1101)
* linter: Fix panic on no_mixed_operators rule (#1094)
* linter: noTemplateLiterals configuration in no_string_refs rule not working (#1063)
* linter/no-render-return-value: remove duplicate test case (#1111)
* linter/no_empty_file: point to start of file instead of the entire file (#1105)
* linter/no_render_return_value: fix false positive when nested inside an arrow expression (#1109)

## [0.0.14] - 2023-10-23

### Bug Fixes

* linter: point to the opening fragment for jsx_no_useless_fragment
* linter: incorrect reporting for jsx_key (#1020)
* linter: fix panic with `strip_prefix` (#1013)
* linter: fix clippy

### Features

* linter: eslint-plugin-react: jsx-no-duplicate-props (#1024)
* linter: eslint/no-fallthrough (nursery)
* linter: eslint-plugin-react/no-useless-fragment (#1021)
* linter: eslint-plugin-unicorn(throw-new-error) (#1005)
* linter: eslint-plugin-unicorn(prefer-array-flat-map) (#997)
* linter: eslint-plugin-unicorn no console spaces (#991)
* linter: eslint-plugin-unicorn(filename-case) (#978)
* linter: add `jest/no-confusing-set-timeout` (#938)
* linter: add `eslint(jest/valid-title)` rule (#966)
* linter: add `jest/no-identical-title` rule (#957)
* linter: add `eslint(jest/valid-expect)` rule (#941)
* minifier: re-enable mangler (#972)
* transformer: finish 2016 exponentiation operator (#996)

### Refactor

* clippy: allow clippy::too_many_lines
* clippy: allow struct_excessive_bools

## [0.0.13] - 2023-09-29

### Bug Fixes

* linter: improve error span for no-thenable

### Features

* linter: improve help message of no-thenable
* linter: add no-redeclare rule. (#683)
* linter: add eslint(jest/no-standalone-expect) (#931)
* linter: add eslint(jest/no-export) (#925)
* linter: add eslint(jest/no-mocks-import) (#924)
* linter: implement eslint-plugin-unicorn/no-thenable rule (#910)
* linter: add eslint-plugin-jest/no-jasmine-globals (#914)
* linter: add no-console rule (#887)
* linter: add eslint-plugin-import/default (#895)
* linter: eslint-plugin-import(no-cycle) (#890)
* linter: add typescript/no-explicit-any (#881)
* linter: implement re-exports (#877)
* linter: add eslint-plugin-import/named
* linter: add current_working_directory to tester
* linter: add rule_path to tester so the file extension can be changed
* linter: add `eslint-plugin-jest/no-done-callback` rule (#846)
* linter: add runner for import-plugin (#858)
* syntax: add loaded_modules to ModuleRecord
* transformer: logical assignment operators (#923)- add jest/no-interpolation-in-snapshots rule (#867) |

### Performance

* linter: early bail out if not jest fn (#885)
* parser: lazily build trivia map instead of build in-place (#903)

## [0.0.12] - 2023-09-06

### Bug Fixes

* cli: spawn linting in another thread so diagnostics can be printed immediately
* linter: fix incorrect behaviour for "-D correctness -A rule-name"
* linter: no-var-requires not warning if has bindings in ancestors (#799)

### Features

* linter: implement unicorn/no-unnecessary-await (#856)
* linter: add eslint-plugin-jest/no-conditional-expect rule (#832)
* linter: add eslint-plugin-jest/no_alias_method rule (#818)
* linter: eslint-plugin-jest/expect-expect (#802)
* linter_plugin: Add linter plugin crate (#798)

### Refactor

* linter: remove complicated linter service setup
* linter: clean up Test a bit
* linter: less a global hashmap to reduce rule timer macro expansion (#822)

### Performance

* linter: parse ts-directive manually (#845)
* linter: swap the order of checks for no_caller (#844)

## [0.0.11] - 2023-08-27

### Refactor

* linter: move the label message to help
* linter: extract `is_valid_jest_call` (#781)
* linter: clean up tester with fixes (#773)

### Features

* ast: Add to ChainExpression and ExpressionArrayElement to ASTKind (#785)
* cli: use insta_cmd for cli snapshot testing (#791)
* linter: detect import (#778)
* linter: implement no-unsafe-declaration-merging (#748)

### Bug Fixes

* linter: show the escaped span for no-useless-escape (#790)

## [0.0.10] - 2023-08-21

### Bug Fixes

* cli: fix a race condition where the program will hang

## [0.0.8] - 2023-08-21

### Bug Fixes

* cli: fix race condition when resolving paths
* eslint/no-obj-calls: should resolve non-global binding correctly (#745)
* linter: change severity of no-obj-calls to warning
* linter: improve error and help message on no-duplicate-enum-values
* linter: improve help message on no-namespace
* linter: reduce the span of no-namespace to the keyword
* linter: no-extra-boolean-cast false positive
* linter: fix some race conditions
* linter: fix false positives in loss-of-precision lint (#664)
* parser,semantic: make semantic own `Trivias` (#711)

### Refactor

* cli: split out group options (#760)
* cli: clean up lint and cli options (#759)
* cli: add WalkOptions for walk logic (#757)
* cli,linter: move path processing logic from cli to linter (#766)
* cli,linter: move the lint runner from cli to linter (#764)
* cli,linter: move LintOptions from cli to linter (#753)
* linter: manually declare lint rules because `cargo fmt` breaks (#671)

### Features

* linter: implement eslint-plugin-unicorn/no-instanceof-array (#752)
* linter: add no-commented-out-tests (#723)
* linter: implement typescript-eslint/ban-ts-comment (#741)
* linter: implement @eslint/no-shadow-restricted-names (#617) (#728)
* linter: implement @typescript-eslint/no-duplicate-enum-values (#726)
* linter: valid-describe-callback(eslint-plugin-jest) (#706)
* linter: implement @typescript-eslint/prefer-as-const (#707)
* linter: @typescript-eslint/no-namespace (#703)
* linter: implement `no-undef` (#672)
* linter: add no-extra-boolean-cast rule (#677)
* linter: enable module record builder
* linter: no-focused-test(eslint-jest-plugin) (#609)
* resolver: add tracing (#710)- vscode extension (#690) |

## [0.0.7] - 2023-07-29

### Performance

* linter: reduce mallocs (#654)

### Bug Fixes

* linter: improve the span for no-inner-declarations
* linter: change no-control-regex to severity warning
* linter: make disable directives work with plugin rule names
* linter: change no-var-requires to severity warning
* linter: change severity of `no-this-alias` from error to warning- no return await error (#539) |

### Features

* cli: add support for `TIMING` env var (#535)
* linter: add style category and change no-empty-interface to style
* linter: eslint/no-loss-of-precision (#649)
* linter: implement no-global-assign (#624)
* linter: add a `run_once` callback (#647)
* linter: eslint/no-empty-character-class (#635)
* linter: implement no-var-requires (#575)
* linter: implement `adjacent-overload-signature` (#578)
* linter: implement `no-test-prefixes` (#531)
* linter: add eslint/no-control-regex (#516)
* linter: implement eslint rule `no-return-await` (#529)
* linter: no disabled tests(eslint-jest-plugin) (#507)
* linter: implement `no-misused-new` (#525)- add eslint/no-obj-calls (#508) |

### Refactor

* linter: remove `Box::leak` (#641)
* linter: run eq_eq_eq fix in some condition (#545)
* linter: expose LintContext as the API for Linter::run
* semantic: symbol declarations and references (#594)- format code |- avoid unstable let_chains |- remove unstable feature const_trait_impl & const_slice_index & slice_group_by (#629) |

## [0.0.6] - 2023-07-01

### Features

* linter: implement @typescript-eslint/no-unnecessary-type-constraint
* linter: implement @typescript-eslint/no-empty-interface
* linter: implement @typescript-eslint/no-non-null-asserted-optional-chain
* linter: implement @typescript-eslint/no-extra-non-null-assertion

## [0.0.5] - 2023-07-01

### Refactor

* linter: improve span for no-case-declarations
* linter: remove redundant backticks from no-constant-binary-expression's error message

### Bug Fixes

* linter: fix no_useless_escape crashing on unicode boundaries
* linter: fix error message for no_dupe_keys

### Features

* linter: implement no-prototype-builtins
* linter: implement no-useless-escape
* linter: implement no-inner-declarations
* linter: implement no-import-assign (nursery)
* linter: implement no-dupe-else-if
* linter: implement no-cond-assign
* linter: implement no-self-assign
* linter: implement no-unsafe-finally
* linter: implement no-unsafe-optional-chaining
* linter: implement no-useless-catch

## [0.0.4] - 2023-06-28

### Features

* linter: implement no_sparse_arrays
* linter: implement `no-ex-assign` (#495)
* linter: implement require_yield
* linter: implement no_delete_var
* linter: implement `no-case-declarations` (#491)

### Bug Fixes

* linter: fix disable directives not working for no_func_assign
* linter: s/no_function_assign/no_func_assign per eslint
* linter: fix no_empty_pattern broken on rest elements

