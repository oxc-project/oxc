# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.5.0] - 2024-06-27

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

- 0537d29 cfg: [**BREAKING**] Move control flow to its own crate. (#3728) (rzvxa)

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 4bce59d semantic/cfg: [**BREAKING**] Re-export `petgraph` as `control_flow::graph`. (#3722) (rzvxa)

### Features

- 3ae2628 linter: Change `no-import-assign` to correctness (#3928) (Boshen)
- a89d501 linter: Implement @typescript-eslint/no-non-null-asserted-nulli… (#3850) (kaykdm)
- fc48cb4 linter: eslint-plugin-jest/prefer-jest-mocked (#3865) (cinchen)
- 63b98bd linter: Accept multiple fixes when fix code (#3842) (mysteryven)
- 328445b linter: Support `vitest/no-disabled-tests` (#3717) (mysteryven)
- 8c61f9c linter: Implement @typescript-eslint/no-non-null-assertion (#3825) (kaykdm)
- 080ecbd linter: Add `no-fallthrough`. (#3673) (rzvxa)
- 9493fbe linter: Add `oxc/no-optional-chaining` rule (#3700) (mysteryven)
- 139adfe linter: Add `@typescript-eslint/no-import-type-side_effects` (#3699) (mysteryven)
- 5f84500 linter/eslint-plugin-react: Implement prefer-es6-class (#3812) (Jelle van der Waa)
- fafe67c linter/import: Implement max-dependencies (#3814) (Jelle van der Waa)
- d5f6aeb semantic: Check for illegal symbol modifiers (#3838) (Don Isaac)

### Bug Fixes

- 4bd2c88 linter: Fix and promote `getter-return` to correctness. (#3777) (rzvxa)
- 1190dee linter: False positives with setters in the `getter-return` rule. (#3714) (rzvxa)
- de0690f linter: Do not run getter-return in typescript (#3693) (Boshen)
- cf71c23 linter: Edge case with infinite loops. (#3672) (rzvxa)
- 99a40ce semantic: `export default foo` should have `ExportLocalName::Default(NameSpan)` entry (#3823) (Boshen)
- abd6ac8 semantic/cfg: Discrete finalization path after `NewFunction`s. (#3671) (rzvxa)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 4d2b7f1 linter: `LintContext` can now only be constructed with a cfg enabled semantic. (#3761) (rzvxa)
- 7302429 linter/prefer_number_properties: Remove the unused `IdentifierName` check (#3822) (Boshen)
- d8ad321 semantic: Make control flow generation optional. (#3737) (rzvxa)

### Testing

- 887da40 linter: Enable `no-fallthrough` test with `disable-next-line`. (#3766) (rzvxa)

## [0.5.0] - 2024-06-27

- 6796891 ast: [**BREAKING**] Rename all instances of `BigintLiteral` to `BigIntLiteral`. (#3898) (rzvxa)

- ae09a97 ast: [**BREAKING**] Remove `Modifiers` from ts nodes (#3846) (Boshen)

- 1af5ed3 ast: [**BREAKING**] Replace `Modifiers` with `declare` and `const` on `EnumDeclaration` (#3845) (Boshen)

- ee6ec4e ast: [**BREAKING**] Replace `Modifiers` with `declare` and `abstract` on `Class` (#3841) (Boshen)

- 4456034 ast: [**BREAKING**] Add `IdentifierReference` to `ExportSpecifier` (#3820) (Boshen)

- 0537d29 cfg: [**BREAKING**] Move control flow to its own crate. (#3728) (rzvxa)

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

- 4bce59d semantic/cfg: [**BREAKING**] Re-export `petgraph` as `control_flow::graph`. (#3722) (rzvxa)

### Features

- 3ae2628 linter: Change `no-import-assign` to correctness (#3928) (Boshen)
- a89d501 linter: Implement @typescript-eslint/no-non-null-asserted-nulli… (#3850) (kaykdm)
- fc48cb4 linter: eslint-plugin-jest/prefer-jest-mocked (#3865) (cinchen)
- 63b98bd linter: Accept multiple fixes when fix code (#3842) (mysteryven)
- 328445b linter: Support `vitest/no-disabled-tests` (#3717) (mysteryven)
- 8c61f9c linter: Implement @typescript-eslint/no-non-null-assertion (#3825) (kaykdm)
- 080ecbd linter: Add `no-fallthrough`. (#3673) (rzvxa)
- 9493fbe linter: Add `oxc/no-optional-chaining` rule (#3700) (mysteryven)
- 139adfe linter: Add `@typescript-eslint/no-import-type-side_effects` (#3699) (mysteryven)
- 5f84500 linter/eslint-plugin-react: Implement prefer-es6-class (#3812) (Jelle van der Waa)
- fafe67c linter/import: Implement max-dependencies (#3814) (Jelle van der Waa)
- d5f6aeb semantic: Check for illegal symbol modifiers (#3838) (Don Isaac)

### Bug Fixes

- 4bd2c88 linter: Fix and promote `getter-return` to correctness. (#3777) (rzvxa)
- 1190dee linter: False positives with setters in the `getter-return` rule. (#3714) (rzvxa)
- de0690f linter: Do not run getter-return in typescript (#3693) (Boshen)
- cf71c23 linter: Edge case with infinite loops. (#3672) (rzvxa)
- 99a40ce semantic: `export default foo` should have `ExportLocalName::Default(NameSpan)` entry (#3823) (Boshen)
- abd6ac8 semantic/cfg: Discrete finalization path after `NewFunction`s. (#3671) (rzvxa)

### Performance
- 4f7ff7e Do not pass `&Atom` to functions (#3818) (overlookmotel)

### Refactor

- 4d2b7f1 linter: `LintContext` can now only be constructed with a cfg enabled semantic. (#3761) (rzvxa)
- 7302429 linter/prefer_number_properties: Remove the unused `IdentifierName` check (#3822) (Boshen)
- d8ad321 semantic: Make control flow generation optional. (#3737) (rzvxa)

### Testing

- 887da40 linter: Enable `no-fallthrough` test with `disable-next-line`. (#3766) (rzvxa)

## [0.4.4] - 2024-06-14

### Features

- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)
- 29c78db linter: Implement @typescript-eslint/explicit-function-return-type (#3455) (kaykdm)
- 21d3425 linter: Typescript-eslint no-useless-empty-export (#3605) (keita hino)
- 85c3b83 linter: Eslint-plugin-jest/max-nested-describes (#3585) (cinchen)
- f6d9ca6 linter: Add `eslint/sort-imports` rule (#3568) (Wang Wenzhe)
- 046ff3f linter/eslint: Add `no_unreachable` rule. (#3238) (rzvxa)
- e32ce00 linter/jsdoc: Implement require-param-name rule (#3636) (Yuji Sugiura)
- 110661c linter/jsdoc: Implement require-param-description (#3621) (Yuji Sugiura)
- d6370f1 linter/jsdoc: Implement require-param-type rule (#3601) (Yuji Sugiura)
- d9c5b33 semantic/cfg: Add `Condition` instruction. (#3567) (Ali Rezvani)
- f2dfd66 semantic/cfg: Add iteration instructions. (#3566) (rzvxa)

### Bug Fixes

- f0b689d linter: Panic in jsdoc/require-param (#3590) (Don Isaac)
- e148a32 semantic/cfg: Correct unreachability propagation in try-finally. (#3667) (Ali Rezvani)

### Refactor

- 84304b4 linter: Add a `ctx.module_record()` method (#3637) (Boshen)
- f98f777 linter: Add rule fixer (#3589) (Don Isaac)
- fa11644 linter: Pass `Rc` by value (#3587) (overlookmotel)
- f702fb9 semantic/cfg: Cleanup control flow and it's builder. (#3650) (rzvxa)
- 5793ff1 transformer: Replace `&’a Trivias` with `Rc<Trivias>` (#3580) (Dunqing)

## [0.4.3] - 2024-06-07

### Features

- 1fb9d23 linter: Add fixer for no-useless-fallback-in-spread rule (#3544) (Don Isaac)
- 6506d08 linter: Add fixer for no-single-promise-in-promise-methods (#3531) (Don Isaac)
- daf559f linter: Eslint-plugin-jest/no-large-snapshot (#3436) (cinchen)
- 4c17bc6 linter: Eslint/no-constructor-return (#3321) (谭光志)
- 4a075cc linter/jsdoc: Implement require-param rule (#3554) (Yuji Sugiura)
- 747500a linter/jsdoc: Implement require-returns-type rule (#3458) (Yuji Sugiura)
- 6b39654 linter/tree-shaking: Support options (#3504) (Wang Wenzhe)
- 0cdb45a oxc_codegen: Preserve annotate comment (#3465) (IWANABETHATGUY)

### Bug Fixes

- b188778 linter/eslint: Fix `require-await` false positives in `ForOfStatement`. (#3457) (rzvxa)
- 350cd91 parser: Should parser error when function declaration has no name (#3461) (Dunqing)

## [0.4.2] - 2024-05-28

### Features

- 14ef4df lint/eslint: Implement require-await (#3406) (Todor Andonov)
- e275659 linter: Add `oxc/no-rest-spread-properties` rule (#3432) (Wang Wenzhe)
- 0d2c977 linter: Add `oxc/no-const-enum` rule (#3435) (Wang Wenzhe)
- 085f917 linter: Add `oxc/no-async-await` rule (#3438) (Wang Wenzhe)
- ded59bc linter: Eslint-plugin-jest/require-top-level-describe (#3439) (cinchen)
- edaa555 linter: Eslint-plugin-jest/prefer-hooks-on-top (#3437) (cinchen)
- aa26ce9 linter: @typescript-eslint/consistent-indexed-object-style (#3126) (Todor Andonov)
- b589fd6 linter/eslint: Implement no-div-regex (#3442) (Jelle van der Waa)
- 147864c linter/eslint: Implement no-useless-concat (#3363) (Jelle van der Waa)

### Bug Fixes

- 5e06298 linter: Memorize visited block id in `neighbors_filtered_by_edge_weight` (#3407) (mysteryven)
- 74b06a7 linter: Accept more valid regex (#3408) (magic-akari)
- 19bb1c0 website: Hack `schemars` to render code snippet in markdown (#3417) (Boshen)

### Documentation

- 5c7041b linter: Add docs for consistent-indexed-object-style (#3409) (Wang Wenzhe)

## [0.4.0] - 2024-05-24

### Features

- e241136 cli,linter: Add `--disable-oxc-plugin` (#3328) (Boshen)
- 8ab9856 cli,linter: Add `--disable`-react/unicorn/typescript-`plugin` (#3305) (Boshen)
- ecdffcf linter: Temporary move react/require-render-return to nursery (Boshen)
- b8997f5 linter: Eslint/no-restricted-globals (#3390) (mysteryven)
- 79811ca linter: Change jsdoc/require-returns from correctness to pedantic (Boshen)
- 8a1db67 linter: Change jsdoc/require-render-return from correctness to pedantic (Boshen)
- fe208dd linter: Start adding json schema for configuration file (#3375) (Boshen)
- aec613b linter: Eslint-plugin-jest/no-duplicate-hooks (#3358) (cinchen)
- e4b3a3c linter: Backward compability for `react-hooks` and `deepscan` plugins (#3334) (Boshen)
- 9744707 linter/eslint: Implement default_case rule (#3379) (Jelle van der Waa)
- 74be8b1 linter/eslint: Implement no-new (#3368) (Jelle van der Waa)
- c588e52 linter/eslint: Implement prefer-exponentiation-operator (#3365) (Jelle van der Waa)
- 283d6c7 linter/eslint: Implement symbol-description (#3364) (Jelle van der Waa)
- b6e2d62 linter/jsdoc: Implement require-returns-description (#3397) (Yuji Sugiura)
- 3a5f088 linter/jsdoc: Implement require-returns rule (#3218) (Yuji Sugiura)
- 3671b5c tasks/website: Code generate the linter rules (Boshen)
- 57d2bca tasks/website: Start generating linter config markdown from json schema (#3386) (Boshen)
- ead637b website: Generate linter configuration page (Boshen)

### Bug Fixes

- c664c6c linter: `no-new` false positive when return from arrow expression (#3393) (Boshen)
- fbccd1f linter: Only report issues on top-level fragment (#3389) (Jovi De Croock)
- a23bbf9 linter: Avoid infinite loop in `jest/expect-expect` (#3332) (mysteryven)
- 385965f linter: Avoid infinite loop when traverse ancestors in `jest/no_conditional_expect` (#3330) (mysteryven)
- 95e9b69 linter: Fix panic in jest/expect-expect (#3324) (Boshen)
- 6c3d99a linter/jsx-no-undef: Check for globals when an identifier is undefined (#3331) (Boshen)
- bb2221e linter/next: False positives for non-custom font link (#3383) (Dunqing)
- 712ee0d linter/react: Fix false positives for async components in `rules_of_hooks` (#3307) (rzvxa)
- 0864cd0 linter/react: Better detection for hooks in the `rules_of_hooks`. (#3306) (rzvxa)
- 9594441 linter/react: `rules_of_hooks` add support for property hooks/components. (#3300) (rzvxa)
- c8f1f79 linter/react: `rules_of_hooks` resolve false positives with conditional hooks. (#3299) (rzvxa)
- d46538e linter/react: Fix loop hooks false positives. (#3297) (Ali Rezvani)

### Performance

- 8388c7b linter: Use `usize` for `RuleEnum` hash (#3336) (Boshen)

### Refactor

- c9d84af diagnostics: S/warning/warn (Boshen)
- 5bf595d linter: Rename variable names prefix `ESLint` to `Oxlint` (Boshen)
- d8c3187 linter: Remove unnecessary check in `eslint/no-global-assign` (#3391) (mysteryven)
- d7849f8 linter: Find return statement by using CFG in `react/require-render-return` (#3353) (mysteryven)
- 8383b6e linter: Remove `with_rule_name` from the tight loop (#3335) (Boshen)
- 4f76cb6 linter: Merge deepscan rules into oxc rules (#3327) (Boshen)
- 78e6326 semantic/cfg: Alias petgraph's `NodeIndex` as `BasicBlockId`. (#3380) (rzvxa)

## [0.3.5] - 2024-05-15

### Features

- 5b2fc39 linter: Add use-isnan fixer for (in)equality operations (#3284) (Don Isaac)
- 3644400 linter/eslint: Implement fixer for unicode-bom rule (#3259) (Jelle van der Waa)

### Bug Fixes

- e12323f linter/no-direct-mutation-state: False positive when class is declared inside a `CallExpression` (#3294) (Boshen)

### Refactor

- 6128171 linter: Rewrite react/require-render-return (#3276) (Wang Wenzhe)

## [0.3.4] - 2024-05-13

### Features

- 6edcae8 linter: Move react/rules_of_hooks to nursery (Boshen)
- 44b16ef linter/eslint: Implement max-classes-per-file (#3241) (Jelle van der Waa)

## [0.3.3] - 2024-05-13

### Features

- c6874ad linter: Demote `no-inner-declarations` from correctness to pedantic (eslint v9) (Boshen)
- 4ccc3ee linter: Demote `react/jsx-no-useless-fragment` from correctness to pedantic (Boshen)
- d45b28a linter: Unicorn/no-anonymous-default-export (#3220) (1zumii)
- 7113e85 linter: Add `radix` rule (#3167) (Kuba Jastrzębski)
- fa0093b linter: Eslint-plugin-next/no-page-custom-font (#3185) (Dunqing)
- 4defe37 linter: Remove deprecated eslint v9 rules `no-return-await` and `no-mixed-operators` (#3188) (Boshen)
- ca9f13f linter: Eslint/no-new-native-nonconstructor (#3187) (Boshen)
- 5514936 linter: Eslint-plugin-next/no-styled-jsx-in-document (#3184) (Dunqing)
- cb2e651 linter: Eslint-plugin-next/no-duplicate-head (#3174) (Boshen)
- 8244d2b linter/eslint: Implement unicode-bom rule (#3239) (Jelle van der Waa)
- 5081652 linter/eslint: Implement no-empty-function rule (#3181) (Jelle van der Waa)
- f88f330 linter/import: Improve multiple exports error message (#3160) (Dunqing)
- 1f135ce linter/react: Add the `rules_of_hooks` rule. (#3071) (rzvxa)
- c0abbbd linter/tree-shaking: Add `isPureFunction` (#3175) (Wang Wenzhe)

### Bug Fixes

- edb30e1 linter: Handle `import { default as foo }` in import/named (#3255) (Boshen)
- 313fb83 linter/default: Ignore unsupported files (e.g. .vue) (Boshen)
- 0ba7778 parser: Correctly parse cls.fn<C> = x (#3208) (Dunqing)

### Refactor

- dbde5b3 diagnostics: Remove export of `miette` (Boshen)
- 551632a diagnostics: Remove thiserror (Boshen)
- 312f74b diagnostics: S/OxcDiagnostic::new/OxcDiagnostic::error (Boshen)
- f7a3773 linter: Clean up diagnostics (Boshen)
- 5671714 linter: Clean up diagnostics in fixer (Boshen)
- 6e90f67 linter: Remove unnecessary usages of `CompactStr` (Boshen)
- 15f275f linter: Reduce llvm lines generated by `RuleEnum::read_json` (#3207) (Boshen)
- a84454c linter: Clean up prefer_node_protocol and move to restriction (#3171) (Boshen)
- f6f7adc linter,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3235) (Boshen)
- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)- 893af23 Clean up more diagnostics usages (Boshen)

## [0.3.2] - 2024-05-04

### Features

- 80cf0b2 linter: @typescript-eslint/prefer-literal-enum-member (#3134) (kaykdm)
- cd600fa linter: Add more "ban-ts-comment" test cases. (#3107) (谭光志)
- bef8a71 linter: Eslint-plugin-jest/require-hook (#3110) (cinchen)
- 388ee51 linter: Typescript-eslint/prefer-enum-initializers (#3097) (Todor Andonov)
- be9cdfc linter: Eslint/no-await-in-loop (#3070) (谭光志)
- 6f5df11 linter/import: Move some rules out of nursery (#2841) (Dunqing)
- 5a1d63a linter/jsdoc: Implement require-yields rule (#3150) (Yuji Sugiura)
- d7a8345 linter/jsdoc: Support settings.ignore(Private|Internal) (#3147) (Yuji Sugiura)
- 5866086 linter/jsdoc: Implement no-defaults rule (#3098) (Yuji Sugiura)
- fa3d9d2 linter/jsdoc: Implement `implements-on-classes` rule (#3081) (Yuji Sugiura)
- d109767 linter/jsdoc: Implement check-tag-names rule (#3029) (Yuji Sugiura)
- 32df6d7 linter/tree-shaking: Support While/Switch/Yield Statement (#3155) (Wang Wenzhe)
- 8290421 linter/tree-shaking: Support SequenceExpression (#3154) (Wang Wenzhe)
- 5c21b7f linter/tree-shaking: Support UnaryExpression (#3153) (Wang Wenzhe)
- 7333618 linter/tree-shaking: Support JSX (#3139) (Wang Wenzhe)
- 16a31e9 linter/tree-shaking: Support import statement (#3138) (Wang Wenzhe)
- 88ded0c linter/tree-shaking: Support ForStatement (#3078) (Wang Wenzhe)
- c3ec710 linter/tree-shaking: Support ExportNamedDeclaration (#3072) (Wang Wenzhe)
- 8cdd5b0 linter/tree_shaking: Support LogicExpression and MemberExpression (#3148) (Wang Wenzhe)

### Bug Fixes

- fde7d65 linter: Handle named export default in import-plugin/named (#3158) (Boshen)
- b1bddac linter: Fix hang if a file fails to parse while using `--import-plugin` (Boshen)
- dcb2528 semantic: Revert test code pushed to the main by accident. (#3085) (Ali Rezvani)
- 8d17ab3 semantic: Allow `root_node` to be empty for empty trees. (#3084) (Ali Rezvani)

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)
- 942b2ba ast: Add array element `Elision` type (#3074) (overlookmotel)
- 222030c linter: Render `--rules` in a table (Boshen)
- 1f12aee linter/jsdoc: Misc improvements (#3109) (Yuji Sugiura)
- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- ae65613 syntax: Use `FxHashMap` for `ModuleRecord::request_modules` (#3124) (Boshen)

## [0.3.1] - 2024-04-22

### Bug Fixes

- a5a7351 linter: Fix unwanted plugin rules being enabled (Boshen)

## [0.3.0] - 2024-04-22

### Features

- 92d709b ast: Add `CatchParameter` node (#3049) (Boshen)
- 8d17bb4 linter: --deny all should not enable nursery rules (Boshen)
- c2ad8f8 linter: Implement fixer for `typescript-eslint/consistent-type-definitions` (#3045) (Todor Andonov)
- ae72be1 linter: Remove all ESLint Stylistic rules (Boshen)
- 58d6438 linter: Change no-empty-static-block to `correctness` (Boshen)
- 5cf55c2 linter: No barrel file. (#3030) (Ali Rezvani)
- ae1f15a linter: Support eslint globals (#3038) (Boshen)
- 1a1ba11 linter/tree-shaking: Support `ExportDefaultDeclaration` (#3052) (Wang Wenzhe)

### Bug Fixes

- b88dfd7 linter: Support `-D all -D nursery` (Boshen)
- 598bbba linter: Fix crashing with `unwrap` in import/no-cycle (#3035) (Boshen)

### Performance

- 6c82961 ast: Box typescript enum variants. (#3065) (Ali Rezvani)
- 48e2088 ast: Box enum variants (#3058) (overlookmotel)

### Refactor

- 53c0ff5 linter: Improve the ergonomics around `ESlintConfig` (#3037) (Boshen)
- 7e4beb0 linter/import/no_cycle: Use ModuleGraphVisitor. (#3064) (Ali Rezvani)

## [0.2.18] - 2024-04-19

### Features

- 2bac1d5 linter: Support `oxlint-disable` alongside `eslint-disable` (#3024) (Boshen)
- fa08abe linter: Remove import/no-unresolved (#3023) (Boshen)
- 9b4e87a linter: Eslint/max-len (#2874) (谭光志)
- df2036e linter: Implement plugin-jsdoc/check-property-names (#2989) (Yuji Sugiura)
- aa62dbb linter: Add missing test cases to no-empty-interface and add config (#2973) (Jose)
- ba2121f linter: Add --jsdoc-plugin flag (#2935) (Yuji Sugiura)
- 395ad76 linter/jsdoc: Update settings.jsdoc method (#3016) (Yuji Sugiura)
- 5d89e75 linter/jsdoc: Implement require-property-(type|name|description) rules (#3013) (Yuji Sugiura)
- 7de9c91 linter/jsdoc: Implement require-property rule (#3011) (Yuji Sugiura)
- ac37d55 linter/tree-shaking: Support DoWhileStatement and IfStatement (#2994) (Wang Wenzhe)
- 5b02ae1 linter/tree-shaking: Support ConditionalExpression (#2965) (Wang Wenzhe)
- da5ea41 linter/tree-shaking: Support Class (#2964) (Wang Wenzhe)

### Bug Fixes

- 627dd42 linter/no-empty-interface: Add missing test (#2979) (Jose)

## [0.2.17] - 2024-04-11

### Features

- 6757dba linter: Eslint-plugin-jest/prefer-lowercase-title (#2911) (cinchen)
- b4b471f linter: Typescript-eslint/consistent-type-definitions (#2885) (Todor Andonov)
- 990eda6 linter/tree-shaking: Support part BinaryExpression (#2922) (Wang Wenzhe)

### Bug Fixes

- 5abbb0c linter: Import/no-cycle ignore type-only imports (#2924) (John Daly)

### Refactor

- 0a77d62 semantic/jsdoc: Rework JSDoc struct for better Span handling (#2917) (Yuji Sugiura)

## [0.2.16] - 2024-04-08

### Features

- acb6eb2 linter: @typescript-eslint/prefer-for-of (#2789) (Denis Gonchar)
- aa63b64 linter: Implement jsdoc/check-access (#2642) (Yuji Sugiura)
- 6823482 linter: Implement jsdoc/empty-tags (#2893) (Yuji Sugiura)
- 7bc638e linter: Eslint-plugin-jest/prefer-mock-promise-sorthand (#2864) (cinchen)
- 6de1b77 linter/import: Add `ignoreTypes` option for the `import/no-cycle` rule (#2905) (John Daly)
- b053d54 linter/tree-shaking: Support try-catch and AwaitExpression (#2902) (Wang Wenzhe)
- 59869d0 linter/tree-shaking: Check `this` in different environment (#2901) (Wang Wenzhe)
- ce34829 linter/tree-shaking: Support ThisExpression and NewExpression (#2890) (Wang Wenzhe)
- 15d08f6 linter/tree-shaking: Support ArrowFunctionExpression (#2883) (Wang Wenzhe)
- 4a86dcb linter/tree-shaking: Support `ArrayExpression` and `ArrayPattern`  (#2882) (Wang Wenzhe)

### Bug Fixes

- 5f8f7f8 ast: `FinallyClause` won't get visited as `BlockStatement` anymore. (#2881) (Ali Rezvani)
- 79e2c95 linter: Handle self closing script tags in astro partial loader (#2017) (#2907) (Kalven Schraut)
- 1cd5e75 linter: Svelte partial loader handle generics (#2875) (#2906) (Kalven Schraut)

## [0.2.15] - 2024-03-30

### Features

- 2365198 cli: Add tsconfig file validation in LintRunner (#2850) (Dunqing)
- d63807e linter: Fallback to the default tsconfig path (#2842) (Dunqing)
- f6391f9 linter: Eslint-plugin-jest/prefer-comparison-matcher (#2806) (cinchen)
- f131442 linter: Eslint-plugin-jest/no-untyped-mock-factory (#2807) (cinchen)
- 451162e linter: Eslint/no-iterator (#2758) (Jose)
- 53ffbc6 linter: Eslint-plugin-react checked-requires-onchange-or-readonly (#2754) (keita hino)
- 1c07a99 linter: Default_param_last (#2756) (Jose)
- 291dc05 linter: No_script_url (#2761) (Jose)
- 76cc906 linter/import: Ignore type-only imports and exports in no_unresolved (#2849) (Dunqing)
- 0cae373 linter/tree-shaking: Pass CallExpression cases (#2839) (Wang Wenzhe)
- fa39fa8 linter/tree-shaking: Check CallExpression when called (#2809) (Wang Wenzhe)
- 3c9e77d linter/tree-shaking: Detect CallExpression in MemberExpression (#2772) (Wang Wenzhe)

### Bug Fixes

- df62828 linter/import: Ignore export declaration in no-duplicates (#2863) (Dunqing)
- c452897 linter/import: False positive for indirect export in namespace (#2862) (Dunqing)
- 64e4de7 linter/max-lines: Only report codes that exceed the line limit (#2778) (Wang Wenzhe)

### Refactor

- 1b5e544 semantic: Distinguish whether requested_modules is type imports/exports (#2848) (Dunqing)
- d9b77d8 sourcemap: Change sourcemap name to take a reference (#2779) (underfin)

## [0.2.14] - 2024-03-19

- c3477de ast: [**BREAKING**] Rename BigintLiteral to BigIntLiteral (#2659) (Arnaud Barré)

### Features

- ac813a6 linter: No_template_curly_in_string (#2763) (Jose)
- 134e15e linter: Eslint/no-proto (#2760) (Jose)
- 39b98ba linter: No_eq_null (#2757) (Jose)
- 86d006e linter: Eslint/max-params (#2749) (Jose)
- 22c84c5 linter: Eslint/guard-for-in (#2746) (Jose)
- f5b4599 linter: Eslint/no-ternary (#2744) (Jose)
- 6189985 linter: Eslint/no-continue (#2742) (Jose)
- 91e8a71 linter: Eslint/no-with (#2741) (Andi Pabst)
- 81752b2 linter: Eslint/max-lines (#2739) (Andi Pabst)
- 0623a53 linter: Eslint-plugin-jest: `prefer-to-contain` (#2735) (cinchen)
- 9edda49 linter: Eslint-plugin-jest: `prefer-expect-resolves` (#2703) (cinchen)
- 53a8e7f linter: Add settings.jsdoc (#2706) (Yuji Sugiura)
- f8fe3af linter: Eslint-plugin-jest: prefer-to-be (#2702) (cinchen)
- 265030d linter: Eslint-plugin-jest: prefer-spy-on (#2666) (cinchen)
- 3ae9479 linter: Report side effect for array element in node_side_effects rule (#2683) (Wang Wenzhe)
- 366a879 linter: Resolve ESM star exports (#2682) (Boshen)
- 9b56134 linter: Support check ImportNamespaceSpecifier in no_import_assign (#2617) (Dunqing)
- 7605cd3 linter: Change ban-ts-comment to pedantic (Boshen)
- 95ac265 linter/import: Check ObjectPattern syntax in namespace (#2691) (Dunqing)
- e86cd62 linter/import: Support check reexport binding in namespace (#2678) (Dunqing)
- 4947809 linter/jest: Add new property for `parse_jest_fn` (#2715) (cinchen)
- 2ef4762 linter/tree-shaking: Add cache for checking mutating identifiers (#2743) (Wang Wenzhe)
- 11219d4 linter/tree_shaking: Check assignment of identifier  (#2697) (Wang Wenzhe)
- 57ce737 semantic: Move redeclare varaibles to symbol table (#2614) (Dunqing)
- 8b3de77 span: `impl<'a> PartialEq<str> for Atom<'a>` (#2649) (Boshen)
- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)
- f8e8af2 task: Init eslint-plugin-tree-shaking rule (#2662) (Wang Wenzhe)- 265b2fb Miette v7 (#2465) (Boshen)

### Bug Fixes

- a671d75 linter: Fix guard_for_in span error (#2755) (Jose)
- 09d4c7d linter: Correct example for no-obj-calls rule (#2618) (overlookmotel)
- b453a07 parser: Parse named rest element in type tuple (#2655) (Arnaud Barré)

### Refactor

- 0f86333 ast: Refactor `Trivias` API - have less noise around it (#2692) (Boshen)
- 220eba1 lint: Split files for no_side_effects rule (#2684) (Wang Wenzhe)
- 47e735a linter: Improve the implementation of no_shadow_restricted_names based on symbols (#2615) (Dunqing)
- 240ff19 parser: Improve parsing of `BindingPattern` in TypeScript (#2624) (Boshen)
- 798a6df span: Disallow struct expression constructor for `Span` (#2625) (Boshen)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.2.13] - 2024-03-05

### Features

- 35ce3cc linter: Eslint-plugin-jest: prefer-to-have-length (#2580) (cinchen)
- 212f128 linter: Eslint-plugin-jest: prefer-strict-equal (#2581) (cinchen)
- fe777f3 linter/import: Partial support namespace check (#2538) (Dunqing)

### Bug Fixes

- 951297e linter: Avoid crash if no members in TSTypeLiteral in typescript/prefer-function-type (#2604) (Wenzhe Wang)
- c09c602 linter: Exclude typescript syntax function in only_used_in_recursion (#2595) (Dunqing)
- f00834d linter: Fix getter return rule false positives in TypeScript (#2543) (BlackSoulHub)
- 24d46bc parser: Fix span start for TSModuleDeclaration (#2593) (Arnaud Barré)
- 37de80d semantic: Jsx reference with an incorrect node id (#2546) (Dunqing)- 7cc9013 Broken build from codegen API change (Boshen)

### Refactor

- ef932a3 codegen: Clean up API around building sourcemaps (#2602) (Boshen)
- 1391e4a semantic/jsdoc: Misc fixes for JSDoc related things (#2531) (Yuji Sugiura)

## [0.2.12] - 2024-02-28

### Features

- 3efbbb2 ast: Add "abstract" type to `MethodDefinition` and `PropertyDefinition` (#2536) (Boshen)
- 02c82c3 cli,linter: Provide tsconfig path from the cli (#2526) (Boshen)
- d41dcc3 linter: Remove all commonjs logic for import plugin (#2537) (Boshen)

## [0.2.11] - 2024-02-26

### Features

- f5aadc7 linter: Handle cjs `module.exports = {} as default export (#2493) (Boshen)
- f64c7e0 linter: Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492) (Boshen)
- d0a9c46 linter: Handle top-level `require` for import plugin (#2491) (Boshen)
- 696818a linter: Implement @typescript-eslint/prefer-ts-expect-error (#2435) (Alex Yip)
- 6aa8c2d linter: Initialize resolver lazily and automatically read tsconfig.json for now (#2482) (Boshen)
- 135e56a linter: Ignore unsupported extensions in import/no_unresolved (#2481) (Boshen)
- 7f86722 linter: Handle built-in modules in import/no_unresolved (#2479) (Boshen)
- 015b2ee linter: Eslint-plugin-react void-dom-elements-no-children (#2477) (keita hino)
- c5f67fe linter: Add boilerplate for eslint-plugin-import/no_duplicates (#2476) (Boshen)
- f1e364f linter: Eslint-plugin-import/no_unresolved (#2475) (Boshen)
- 2714a32 linter: Continue working on no_cycle (#2471) (Boshen)
- 6527bfd linter: Add boilerplace code for import/namespace,no_deprecated,no_unused_modules (#2470) (Boshen)
- ff6a337 linter: Typescript-eslint: prefer-function-type (#2337) (zhangrunzhao)

### Bug Fixes

- 93742f8 linter: Correct configuration file parsing for jsx-no-useless-fragment (#2512) (keita hino)
- fba66dc linter: Improve import/no-named-as-default (#2494) (Boshen)
- d741d72 linter: Fix import plugin hanging when ignored modules are imported (#2478) (Boshen)
- 35a0f89 linter: Handle cases where createElement is an Identifier in is_create_element_call (#2474) (keita hino)
- bc22ae5 semantic: Refactor jsdoc finding (#2437) (Yuji Sugiura)

### Refactor

- 540f917 ast: Remove `TSEnumBody` (#2509) (Boshen)
- 9087f71 ast: S/TSThisKeyword/TSThisType to align with estree (Boshen)
- d08abc6 ast: S/NumberLiteral/NumericLiteral to align with estree (Boshen)
- e6b391a ast: S/ArrowExpression/ArrowFunctionExpression to align estree (Boshen)

## [0.2.10] - 2024-02-21

### Features

- e6d536c codegen: Configurable typescript codegen (#2443) (Andrew McClenaghan)
- f7e1576 linter: Eslint no-nonoctal-decimal-escape (#2428) (tudorbarbu)

### Bug Fixes

- 5bd2ce6 semantic: Incorrect reference flag for MemberExpression assign (#2433) (Dunqing)

### Refactor

- cd48e1e linter: Simplify getting ImportDefaultSpecifier (#2453) (Dunqing)
- 2a2bb2b linter: Improve implementation of no_dupe_class_members based on ClassTable (#2446) (Dunqing)- a2c173d Remove `panic!` from examples (#2454) (Boshen)

## [0.2.9] - 2024-02-18

### Features

- 0b9e122 linter: Implement `unicorn/no-process-exit` rule (#2410) (Yuji Sugiura)
- d9f073e linter: Detect jest file by default glob pattern (#2408) (Wenzhe Wang)
- 8d6202f linter: Eslint-plugin-jest require-to-throw-message (#2384) (keita hino)
- 92afbb1 linter: Eslint-plugin-jest: prefer-equality-matcher (#2358) (cinchen)
- 40e9541 semantic: Add export binding for ExportDefaultDeclarations in module record (#2329) (Dunqing)- 8e0277e Add Typescript ban-tslint-comment (#2371) (tudorbarbu)

### Bug Fixes

- f49ffb2 linter: `getter-return` false positive with TypeScript syntax (#2363) (Boshen)
- ebc08d4 linter: Add missing typescript-eslint(_) prefix for some errors (#2342) (Maurice Nicholson)
- 6a25864 linter/jsx_a11y: Refactor jsx-a11y related utils and its usage (#2389) (Yuji Sugiura)
- 2521b52 linter/jsx_a11y: Ensure plugin settings are used (#2359) (Yuji Sugiura)

### Performance

- 7d7a3fc lint/no_var_requires: Quicker way to check if the `IdentifierReference` point to a global variable (#2376) (Yunfei He)

### Refactor

- 67d7a46 linter: Get arrow expression by scope_id in no_render_return_value (#2424) (Dunqing)
- 63b4741 linter/config: Use serde::Deserialize for config parsing (#2325) (Yuji Sugiura)

## [0.2.8] - 2024-02-06

### Features

- d571839 ast: Enter AstKind::ExportDefaultDeclaration, AstKind::ExportNamedDeclaration and AstKind::ExportAllDeclaration (#2317) (Dunqing)
- a762d17 linter: Promote `no-this-before-super` to correctness (#2313) (Boshen)
- 0060d6a linter: Implement no_this_before_super with cfg (#2254) (Tzvi Melamed)
- f3035f1 semantic: Apply ImportSpecifier's binder and remove ModuleDeclaration's binder (#2307) (Dunqing)- 8771c64 Add typescript-eslint rule array-type (#2292) (luhc228)

### Bug Fixes

- b5e43fb linter: Fix no_dupe_keys false postive on similar key names (#2291) (Boshen)

### Refactor

- 1822cfe ast: Fix BigInt memory leak by removing it (#2293) (Boshen)

## [0.2.7] - 2024-02-03

### Features

- 2578bb3 ast: Remove generator property from ArrowFunction (#2260) (Dunqing)
- a95a16c linter: Complete custom components setting (#2234) (hjio)
- da3b305 linter: Implement @next/next/no-before-interactive-script-outsi… (#2203) (kaykdm)
- b694a6a linter: Implement @next/next/no-unwanted-polyfillio (#2197) (kaykdm)
- e561457 semantic: Track cfg index per ast node (#2210) (Tzvi Melamed)

### Bug Fixes

- 2beacd3 lexer: Correct the span for irregular whitespaces (#2245) (Boshen)
- 37a2676 linter: AllowFunction doesn't support generator (#2277) (Dunqing)
- f039ad6 linter: Ban `--fix` for variety files(vue, astro, svelte) (#2189) (Wenzhe Wang)
- f32228e linter: Jsx no undef match scope should check with ancestors (#2027) (西了意)
- 73ccf8a oxc_semantic: Proper traversal of try statements (#2250) (Tzvi Melamed)

### Refactor

- 1de3518 linter: Remove Regex and change error position (#2188) (Wenzhe Wang)- 650f6c9 Use our forked version of miette::Reporter for tests (#2266) (Boshen)- 87b9978 Move all miette usages to `oxc_diagnostics` (Boshen)

## [0.2.6] - 2024-01-26

### Features

- ee5b968 linter: Support read env from eslintrc (#2130) (fi3ework)
- 8898377 semantic: Cfg prototype (#2019) (Boshen)
- 2794064 transfrom: Transform-json-strings (#2168) (underfin)

### Bug Fixes

- 2e3153e linter: Rename react_perf/jsx_no_new_function_as_props to jsx_no_new_function_as_prop (#2175) (Yuji Sugiura)

### Refactor

- a17e43e linter: Move settings and env to the config module (#2181) (Boshen)

## [0.2.5] - 2024-01-25

### Features

- ac1d318 linter: Eslint-plugin-jest: prefer-called-with (#2163) (cin)
- 3891430 linter: Eslint: no-void (#2162) (cin)

### Bug Fixes

- 989ab88 codegen: Print `Directive` original string (#2157) (underfin)
- c18619e linter: Use correct rule name (#2169) (Yuji Sugiura)
- 2602232 linter: Explicit-length-check inside ternary (#2165) (Maurice Nicholson)

## [0.2.3] - 2024-01-23

### Features

- 6d808a6 linter: Linter-eslint-plugin-import/no-named-as-default (#2109) (Valerii Smirnov)
- d90db3a linter: Promote no-new-array to correctness with better help message (#2123) (Boshen)
- 69fecac linter: Eslint config jsonc support (#2121) (Boshen)
- 5ca07bc linter: Eslint-plugin-react-perf (#2086) (Hulk)
- b160842 linter: Support eslint config in nextjs eslint (#2107) (kaykdm)
- 16b3261 linter: Eslint-plugin-jest: no-restricted-jest-methods (#2091) (cin)- 20a34b5 Introduce --react-perf-plugin CLI flag, update rules to correctness (#2119) (Hulk)- 4adce6f (eslint-plugin-jest): no-restricted-matchers (#2090) (cin)

### Bug Fixes

- d00c44c linter: Allow `[...new Array(n)]` in no-useless-spread (#2124) (Boshen)
- 2228aa8 linter: Jsx_a11y/img-redundant linter enable test case(#2112) (msdlisper)
- 142f84f linter: Not use `new_inline` with flexible str (#2106) (Wenzhe Wang)

### Refactor

- 766ca63 ast: Rename RestElement to BindingRestElement (#2116) (Dunqing)
- 8bccdab semantic: Add binder for FormalParameters and RestElement, replacing the binder for FormalParameters (#2114) (Dunqing)

## [0.2.2] - 2024-01-20

### Features

- 721a869 linter: Improve no_redeclare rule implementation (#2084) (Dunqing)- 2f1e1e2 Expose linter RULES and use it for listing (#2083) (Yuji Sugiura)

### Bug Fixes

- d7ecd21 linter: Eslint-plugin-import no-named-as-default-member rule (#2071) (Valerii Smirnov)
- 3faa2aa linter: S/consistent-type-export/consistent-type-exports (#2065) (Boshen)

### Refactor

- a368134 linter: Perfect the scope linter (#2092) (msdlisper)

## [0.2.1] - 2024-01-16

### Features

- 9e06bd7 linter: Remove the `--timings` feature (#2049) (Boshen)
- c60c315 linter: Eslint-plugin-import no-named-as-default-member rule (#1988) (Valerii Smirnov)
- 530d1be linter: Eslint-plugin-jsx-a11y no-redundant-roles rule (#1981) (Yuto Yoshino)
- 198f0e5 linter: Eslint-plugin-jsx-a11y aria-activedescendant-has-tabindex (#2012) (keita hino)
- a356918 linter: Eslint-plugin-next: no-document-import-in-page (#1997) (kaykdm)
- c70a065 linter: Eslint-plugin-next: no-head-element (#2006) (kaykdm)
- 8f0f824 linter: Eslint-plugin-next:  no-typos (#1978) (kaykdm)
- 04540f7 linter: Eslint-plugin-jsx-a11y click-events-have-key-events (#1976) (Yuji Sugiura)

### Bug Fixes

- 3b40fbd linter: False positive for filename_case where filename doesn't have a proper casing (#2032) (Boshen)
- 68606c4 linter: Keep rules disabled if the rule is not enabled in the config (#2031) (Boshen)
- 107a32e linter: Fix false positive for `erasing-op` in `0/0` case (#2009) (Cameron)

### Refactor

- f514410 linter: Move `LintSettings` to its own file (#2052) (Boshen)
- ae4e714 linter: Remove the `LintSettings` parameter from `LintContext::new`. (#2051) (Boshen)
- b386177 linter: Move away from tuples for test cases (#2011) (Boshen)

## [0.2.0] - 2024-01-12

### Features

- d51c9f1 linter: Eslint-plugin-jest: no-test-return-statement (#1979) (cin)
- fb5d0a7 linter: Add support for same rule name but different plugin names (#1992) (Boshen)
- b7ea4e5 linter: Support vue generic component (#1989) (Boshen)
- c5887bc linter: Implement @typescript-eslint/triple-slash-reference (#1903) (kaykdm)
- ac704cc linter: Eslint-plugin-jsx-a11y autocomplete-valid (#1901) (Yuto Yoshino)
- 40dbfae linter: Eslint-plugin-react: no-direct-mutation-state (#1892) (zhangrunzhao)
- 856b9a5 linter: Support overriding oxlint rules by eslint config (#1966) (Boshen)
- 7891670 linter: Eslint-plugin-react: require-render-return (#1946) (kaykdm)
- 2b7ca59 linter: Eslint-plugin-jsx-a11y role-has-required-aria-props (#1881) (Yuto Yoshino)
- f6047b6 linter: Eslint-plugin-jsx-a11y role-support-aria-props (#1961) (Rintaro Itokawa)
- fd5856e linter: Eslint-plugin-jsx-a11y role-support-aria-props (#1949) (Rintaro Itokawa)
- c6eb519 linter: Eslint-plugin-react: no-unknown-property (#1875) (Valerii Smirnov)
- fe48bfa lsp: Support vue, astro and svelte (#1923) (IWANABETHATGUY)- ac3b44b Nextjs plugin (#1948) (Cameron)

### Bug Fixes

- e0da12a linter: Allow eslintrc to add rule when overriding (#1984) (fi3ework)
- d4acd14 linter: Jsx-key: handle anonymous functional components in arrays that have a function body (#1983) (Maurice Nicholson)
- b5f4f1e linter: Fix plugin name parsing when reading config file (#1972) (Hao Cheng)
- 8d9894a linter: Support cases where aria-hidden includes expressions (#1964) (keita hino)
- 66e95a5 linter: Change severity of no-sparse-arrays to warnings (Boshen)

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)
- 64310fa linter: Remove duplicate `get_jsx_attribute_name` (#1971) (Cameron)

## [0.1.2] - 2024-01-06

### Features

- c2e8ef5 linter: Disable no-unused-labels for svelte (#1919) (Boshen)
- f2ed83c linter: <script> part of svelte file (#1918) (Boshen)

### Bug Fixes

- bb6128b linter: Change no-var to restriction (Boshen)

### Refactor

- 450791d linter: Rename *_partial_loader files (#1916) (Boshen)

## [0.1.1] - 2024-01-06

### Features

- 8f27a98 cli: Support walk vue and astro (#1745) (Wenzhe Wang)
- 0feeac5 lint: Add partial loader register (#1760) (Wenzhe Wang)
- 55a87b2 linter: Eslint: no-var (#1890) (zhangrunzhao)
- 497a207 linter: Parse two script tags from vue (#1899) (Boshen)
- 8a3eff1 linter: Parse multiple script tags in astro file (#1898) (Boshen)
- 4c5c61e linter: Add support for multiple script tags from vue and stro (#1897) (Boshen)
- 5f29e3f linter: No irregular whitespace (#1877) (Deivid Almeida)
- a63490c linter: Support astro front matter `---` block (#1893) (Boshen)
- 11ca5c2 linter: Do not lint when vue file has no js section (#1891) (Boshen)
- 8a1e894 linter: Eslint-plugin-jsx-a11y prefer-tag-over-role (#1831) (Yuto Yoshino)
- 2ac2630 linter: Eslint-plugin-jsx-a11y mouse-events-have-key-events (correctness) (#1867) (Ken-HH24)
- 3d41637 linter: Add Vue loader (#1814) (Wenzhe Wang)
- ba0a4a8 linter: Eslint-plugin-react: jsx-no-undef (#1862) (Valerii Smirnov)
- af61894 linter: Eslint plugin jsx a11y: aria-role (#1849) (msdlisper)
- 4c1673c linter: Use settings for eslint-plugin-jsx-a11y/html_has_lang (#1843) (msdlisper)
- f45a3cc linter: Support eslint/no-unused-private-class-members rule (#1820) (Dunqing)
- 98895ca linter: Eslint-plugin-jsx-a11y media-has-caption (#1822) (poteboy)
- 0280e06 linter: Refine test for no-distracting-elements (#1824) (Tapan Prakash)
- f0ad356 linter: Refine jsx-a11y settings (#1816) (msdlisper)
- d984d59 linter: Eslint-plugin-jsx-a11y lang (#1812) (msdlisper)

### Bug Fixes

- 24d209c linter: Fix vue parser not working for multiple scripts after <template> (#1904) (Boshen)
- 4ea6e5d linter: Do not check empty file for vue / astro files (#1900) (Boshen)
- b2a62dd linter: Error rule config in media_has_caption (#1864) (msdlisper)
- 1ddbe8f linter: Unexpected unwrap panic (#1856) (msdlisper)
- d803a3a linter: Ignore false positives in eslint-plugin-react(jsx-key) (#1858) (Maurice Nicholson)

### Performance

- dae5f62 semantic: Check duplicate parameters in Binder of FormalParameters (#1840) (Dunqing)

### Refactor

- 4515644 linter: Get js code slice from vue source code (#1876) (Wenzhe Wang)
- 040278a linter: Extract common code (#1848) (msdlisper)
- 0db2b84 linter: Simplify Parent Node Access in MediaHasCaption Rule (#1829) (poteboy)
- 6c5b22f semantic: Improve ClassTable implmention and merge properties and methods to elements (#1902) (Dunqing)

## [0.0.22] - 2023-12-25

### Features

- d41e3fd ast: Enter/leave ClassBody and PrivateInExpression (#1792) (Dunqing)
- b4138aa linter: Change double-comparisons to correctness (Boshen)
- 32413f5 linter: Eslint-plugin-jsx-a11y aria-props (#1797) (poteboy)
- bf527f4 linter: Eslint-plugin-jsx-a11y no-aria-hidden-on-focusable (#1795) (poteboy)
- b8a90c1 linter: Eslint-plugin-jsx-a11y no-distracting-elements rule (#1767) (Tapan Prakash)
- f8b386e linter: Correct example and docs url for number_arg_out_of_range (#1737) (legend80s)
- 117f44c linter/eslint/no-cond-assign: Span points to the operator (#1739) (Dunqing)
- 521aa2c linter/eslint/no-useless-escape: Support auto fix (#1743) (Dunqing)

### Bug Fixes

- b25f014 linter: Fix a typo in no_redeclare message (#1789) (Milo)
- 0bf7596 linter: Support read the third item in config file (#1771) (Wenzhe Wang)
- 2286181 linter: Update snapshots (Boshen)
- 5d7ea9d linter: Change non-error lints to warning (Boshen)
- 6d42022 linter: Improve the help message for const-comparisons (#1764) (Boshen)
- 0e563b3 linter: Fix missing ` in the help message for const-comparisons (Boshen)
- 8e6004f linter/eslint/no-obj-calls: Correctly resolves the binding name (#1738) (Dunqing)

### Performance

- 51a243b linter: Reduce the `RuleEnum` enum size from 168 to 16 bytes (#1783) (Boshen)
- 2e707bc linter: Use simd (memchr) for no-useless-escape search (#1766) (Boshen)
- d0cc3ec linter: Change regex to static in no_commented_out_tests (Boshen)
- e741b8f linter: Precompute `rule.name()` (#1759) (Boshen)

### Documentation

- 18f0e20 linter: Update comments (#1779) (Wenzhe Wang)

### Refactor

- 2d3ac95 linter: Shrink the error span for require_yield (Boshen)
- 38cb487 linter: Explain no-empty-pattern (Boshen)

## [0.0.21] - 2023-12-18

### Features

- 621a943 linter: Eslint-plugin-jsx-a11y no-access-key (correctness) for  (#1708) (yoshi2no)
- 774a257 linter: Eslint-plugin-unicorn no-null(style) (#1705) (Ken-HH24)
- 6a90cd4 linter: Add  jsx-a11y settings (#1668) (msdlisper)
- cf0793b linter: `tabindex-no-positive` for eslint-plugin-jsx-a11y (#1677) (yoshi2no)
- c9589b5 linter: Eslint-plugin-unicorn/prefer-prototype-methods (#1660) (Hao Cheng)
- 90524c8 linter: Add eslint-plugin-import(export) rule (#1654) (Wenzhe Wang)
- 282771a linter: Eslint-plugin-unicorn prefer-dom-node-text-content(style) (#1658) (Ken-HH24)

### Bug Fixes

- d101acf linter: Prefer-string-starts-ends-with: ignore `i` and `m` modifiers. (#1688) (Andy Armstrong)
- 0a8746c linter: Panic in prefer string starts, ends with (#1684) (Cameron)
- e5752a5 linter: Fix excape_case panicing on unicode strings (#1673) (RiESAEX)

### Performance

- 0080638 linter/react: Find class node by symbols in get_parent_es6_component (#1657) (Dunqing)

### Refactor

- 8f296df linter: Use fxHashMap in jsx-a11y settings (#1707) (msdlisper)
- d719af4 linter: Make some jest rules report more detailed (#1666) (Wenzhe Wang)- 0d7e166 Use `new_without_config` for `jsx_key` (#1685) (Cameron)

## [0.0.20] - 2023-12-13

### Features

- b425b73 linter: Eslint-plugin-unicorn prefer-modern-dom-apis(style) (#1646) (Ken-HH24)

### Bug Fixes

- 117d95f linter: Improve the span message for no-accumulating-spread (Boshen)- ef740c3 Improve span for no accumulating spread (#1644) (Cameron)- 65c0772 Remove escapes in no array reduce test cases (#1647) (Cameron)- a09060b Remove escapes in prefer regexp test test cases (#1645) (Cameron)

### Refactor

- 4ced3f9 linter: Separate out the category in the output of `--rules` (Boshen)

## [0.0.19] - 2023-12-08

### Features

- d88f4f4 linter: Eslint-plugin-jsx-a11y no-autofocus  (#1641) (msdlisper)
- ddb3c62 linter: Eslint-plugin-jsx-a11y scope rule (correctness) (#1609) (Shinobu Hayashi)
- 795db7c linter: Cxc: no accumulating spread (#1607) (Don Isaac)
- c8e2ef6 linter: Eslint-plugin-unicorn: explicit-length-check (#1617) (RiESAEX)
- 519b5f2 linter: Eslint-plugin-unicorn prefer-reflect-apply(style) (#1628) (Ken-HH24)
- 32504ca linter: Add a `perf` category (#1625) (Boshen)
- b573036 linter: Eslint-plugin-jsx-a11y iframe-has-title rule (correctness) (#1589) (Shinobu Hayashi)
- 967aa35 linter: Eslint-plugin-unicorn require-array-join-separator(style) (#1608) (Ken-HH24)
- ba5b13d linter: Eslint-plugin-unicorn no-unreadable-array-destructuring (style) (#1594) (Ken-HH24)
- a614255 linter: Eslint-plugin-jsx-a11y  img-redundant-alt (correctness) (#1571) (Shinobu Hayashi)
- 8bef1f1 linter: Eslint-plugin-unicorn numeric-separators-style (style) (#1490) (Jon Surrell)
- 72dd72b linter: Eslint-plugin-unicorn/no-unreadable-iife (#1572) (Hao Cheng)
- 3b2b6a0 linter: Eslint-plugin-unicorn no-await-expression-member (style) (#1569) (Ken-HH24)
- afeed17 linter: Eslint-lugin-unicorn no_useless_length_check (#1541) (Radu Baston)
- a2510be linter: No-is-mounted for eslint-plugin-react (#1550) (Ken-HH24)
- 59d0428 linter: Eslint 9.0 no empty static block (#1543) (Radu Baston)
- 8afda7d linter: Eslint-plugin-unicorn: escape-case (#1495) (RiESAEX)
- ebf5cf8 linter: Heading-has-content for eslint-plugin-jsx-a11y (#1501) (Ken-HH24)
- 7930f90 linter: Eslint-plugin-unicorn prefer-set-size (correctness) (#1508) (Cameron)
- 0dd5ec1 linter: Eslint-plugin-unicorn prefer-native-coercion-functions (pedantic) (#1507) (Cameron)
- c8cc814 linter: Eslint-plugin-jsx-a11y anchor_is_valid (correctness) (#1477) (msdlisper)- 872e8ad Eslint-plugin-unicorn (recommended) prefer-node-protocol (#1618) (IWANABETHATGUY)

### Bug Fixes

- b973261 linter: Improve the key span for jsx-key (Boshen)

### Refactor
- b5f8a65 Improve pattern match of prefer-reflect-apply (#1630) (IWANABETHATGUY)

## [0.0.18] - 2023-11-22

### Features

- 8b0032d linter: Eslint plugin unicorn: no useless switch case (#1463) (Cameron)
- 822ce76 linter: Html-has-lang for eslint-plugin-jsx-a11y (#1436) (Ken-HH24)
- 2ba69f1 linter: `anchor-has-content` for eslint-plugin-jsx-a11y (zhangpeng)
- 5543e8c linter: Eslint-plugin-unicorn/no-nested-ternary (#1417) (Hao Cheng)
- b72f83b linter: For-direction rule add check for condition in reverse o… (#1418) (Angelo Annunziata)
- 98279fc linter: Eslint-plugin-unicorn: no-hex-escape (#1410) (RiESAEX)
- 3df810b linter: Eslint-plugin-jest: no-deprecated-function (#1316) (cin)
- 5ade895 linter: Add rule `eslint(no_regex_spaces)` (#1129) (Iván Ovejero)
- 71b3f48 linter: Eslint-plugin-unicorn/number-literal-case (#1271) (Hao Cheng)
- 6ac5403 linter: Eslint-plugin-jest/max_expects (#1239) (cin)
- 18a3525 linter: Reimplement eslint-plugin-jest(no-identical-title) (#1229) (Wenzhe Wang)
- 82c1769 linter: Eslint-plugin-unicorn/no-abusive-eslint-disable (#1125) (Hao Cheng)
- 0218ae8 prettier: Print leading comments with newlines (#1434) (Boshen)

### Bug Fixes

- 5a4e611 linter: Detect assert function in Await Expression (#1202) (Wenzhe Wang)

### Refactor

- f775488 lint: Replace `parse_jest_fn_*` methods in eslint-plugin-jest(no-standalone-expect) rule (#1231) (Wenzhe Wang)
- 071311a lint: Migrate eslint-plugin-jest(expec-expect) (#1225) (Wenzhe Wang)
- dfc2c6a linter: Replace the old parse_expect_jest_fn.rs file (#1267) (Wenzhe Wang)
- f3788ee linter: Remove all old `parse_expect_jest_fn_call` (#1259) (Wenzhe Wang)
- 1dd321e linter: Remove all old `parse_general_jest_fn_call` in jest rules (#1232) (Wenzhe Wang)
- 72b3bdf linter: Replace all `is_type_of_jest_fn_call` (#1228) (Wenzhe Wang)
- efc346e linter: Migrate eslint-plugin-jest(no-alias-method) (#1226) (Wenzhe Wang)
- b4ce2b5 linter: Remove unused logic in `resolve_to_jest_fn` (#1208) (Wenzhe Wang)
- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.0.17] - 2023-11-09

### Bug Fixes

- 49b6be6 linter: Fix handling of repeated eslint-disable comments (#1200) (Hao Cheng)

## [0.0.16] - 2023-11-08

### Features

- 6e76669 lint: Remove unnecessary check (#1185) (cin)
- 9369424 linter: Eslint-plugin-jest: no-hooks (#1172) (cin)
- 033a112 linter: Support eslint(default-case-last) (#1156) (Yiming Pan)
- a5b87c4 linter: Eslint-plugin-unicorn no-object-as-default-parameter (#1162) (Cameron)
- 85651af linter: Jest/prefer-todo rule (#1065) (cin)
- 1aa4b4e linter: Add  rule `eslint-plugin-jsx-a11y(alt-text)` (#1126) (Trevor Manz)- 4e9260e Basic enable plugin (#1154) (Wenzhe Wang)

### Bug Fixes

- fa4e0ca linter: Fix covered span of eslint-disable-next-line comments (#1128) (Hao Cheng)
- 278a1d6 linter/jsx_key: Ignore ObjectProterty nodes (#1139) (Wei Zhu)

### Refactor

- 1cc449f linter: Reduce the lookup times of Call Expression in Jest rules (#1184) (Wenzhe Wang)- 3b88f74 Change jest rule's category (#1155) (Wenzhe Wang)- e9b88e0 Split parse_jest_fn_call (#1152) (Wenzhe Wang)

## [0.0.15] - 2023-10-30

### Features

- 407e406 linter: Change some rules pedantic and improve help message (#1112) (Boshen)
- 8b11592 linter: Demote prefer_array_flat_map to style (#1108) (Boshen)
- d4c05ff linter: Support unicorn/prefer-query-selector (#1068) (Dunqing)
- 0a0e93b linter: Eslint-plugin-unicorn require-number-to-fixed-digits-argument (#1073) (Mariusz Antas)
- 162c720 linter: Eslint-plugin-unicorn  switch-case-braces (#1054) (Mariusz Antas)
- 47837e5 linter: Support react/no-string-refs (#1055) (Dunqing)
- db83f66 linter: Eslint-plugin-unicorn - no-empty-file (#1052) (Cameron)
- ebab50e linter: Eslint-plugin-react no-string-refs (#1053) (Trevor Manz)
- d8f07ca linter: Support react/no-render-return-value (#1042) (Dunqing)
- 64988f4 linter: Eslint-plugin-react(no-unescaped-entities) (#1044) (Cameron)
- a2e40ef linter: Eslint-plugin-react/no-find-dom-node (#1031) (Hao Cheng)
- b703c0c linter/no_children_prop: Point the span to "children" (#1106) (Boshen)
- af1a76b transformer: Implement some of needs_explicit_esm for typescript (#1047) (Boshen)

### Bug Fixes

- 6295f9c ast: Jsx attribute value and text child should be jsx string (#1089) (Boshen)
- b4739e5 linter: Fix panic in no_unescaped_entities (#1103) (Boshen)
- a455c81 linter: Revert changes to JSX attribute strings (#1101) (Boshen)
- bd34dc7 linter: Fix panic on no_mixed_operators rule (#1094) (Cameron)
- 59660a5 linter: NoTemplateLiterals configuration in no_string_refs rule not working (#1063) (Dunqing)
- 144c881 linter/no-render-return-value: Remove duplicate test case (#1111) (Dunqing)
- 22c31ce linter/no_empty_file: Point to start of file instead of the entire file (#1105) (Boshen)
- 4975440 linter/no_render_return_value: Fix false positive when nested inside an arrow expression (#1109) (Boshen)

## [0.0.14] - 2023-10-23

### Features

- dea9b7c linter: Eslint-plugin-react: jsx-no-duplicate-props (#1024) (Cameron)
- 25247e3 linter: Eslint/no-fallthrough (nursery) (Sg)
- f13fc22 linter: Eslint-plugin-react/no-useless-fragment (#1021) (Cameron)
- 88cf98a linter: Eslint-plugin-unicorn(throw-new-error) (#1005) (Cameron)
- 952139c linter: Eslint-plugin-unicorn(prefer-array-flat-map) (#997) (Cameron)
- 41c55bc linter: Eslint-plugin-unicorn no console spaces (#991) (Cameron)
- eaa0c58 linter: Eslint-plugin-unicorn(filename-case) (#978) (Boshen)
- 205f66b linter: Add `jest/no-confusing-set-timeout` (#938) (cin)
- 90828c4 linter: Add `eslint(jest/valid-title)` rule (#966) (Wenzhe Wang)
- 7a62d4b linter: Add `jest/no-identical-title` rule (#957) (Wenzhe Wang)
- 812baeb linter: Add `eslint(jest/valid-expect)` rule (#941) (Wenzhe Wang)
- ef8aaa7 minifier: Re-enable mangler (#972) (Boshen)
- 0f72066 transformer: Finish 2016 exponentiation operator (#996) (Boshen)

### Bug Fixes

- b6b6853 linter: Point to the opening fragment for jsx_no_useless_fragment (Boshen)
- ee134f0 linter: Incorrect reporting for jsx_key (#1020) (Cameron)
- c95f2e0 linter: Fix panic with `strip_prefix` (#1013) (Boshen)
- a710e73 linter: Fix clippy (Boshen)

### Refactor

- db5417f clippy: Allow clippy::too_many_lines (Boshen)
- eaeb630 clippy: Allow struct_excessive_bools (Boshen)

## [0.0.13] - 2023-09-29

### Features

- 982ae9b linter: Improve help message of no-thenable (Boshen)
- 2453954 linter: Add no-redeclare rule. (#683) (cin)
- d700cf8 linter: Add eslint(jest/no-standalone-expect) (#931) (Wenzhe Wang)
- e2a4927 linter: Add eslint(jest/no-export) (#925) (Wenzhe Wang)
- eec9fd4 linter: Add eslint(jest/no-mocks-import) (#924) (Wenzhe Wang)
- 4bf329e linter: Implement eslint-plugin-unicorn/no-thenable rule (#910) (Devin-Yeung)
- 9178451 linter: Add eslint-plugin-jest/no-jasmine-globals (#914) (Wenzhe Wang)
- dd74d93 linter: Add no-console rule (#887) (Todor Andonov)
- 8c44a9e linter: Add eslint-plugin-import/default (#895) (Wenzhe Wang)
- f488e1f linter: Eslint-plugin-import(no-cycle) (#890) (Boshen)
- 35e1898 linter: Add typescript/no-explicit-any (#881) (Don Isaac)
- 4e5f63a linter: Implement re-exports (#877) (Boshen)
- 1b8e2c0 linter: Add eslint-plugin-import/named (Boshen)
- a358856 linter: Add current_working_directory to tester (Boshen)
- 9ee7593 linter: Add rule_path to tester so the file extension can be changed (Boshen)
- 9679ca7 linter: Add `eslint-plugin-jest/no-done-callback` rule (#846) (Wenzhe Wang)
- ee54575 linter: Add runner for import-plugin (#858) (Boshen)
- 75d928a syntax: Add loaded_modules to ModuleRecord (Boshen)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)- f93c861 Add jest/no-interpolation-in-snapshots rule (#867) (Wenzhe Wang)

### Bug Fixes

- 85b113d linter: Improve error span for no-thenable (Boshen)

### Performance

- 9c46e7e linter: Early bail out if not jest fn (#885) (Wenzhe Wang)
- babbc47 parser: Lazily build trivia map instead of build in-place (#903) (Boshen)

## [0.0.12] - 2023-09-06

### Features

- 286049b linter: Implement unicorn/no-unnecessary-await (#856) (Devin-Yeung)
- fa1d7da linter: Add eslint-plugin-jest/no-conditional-expect rule (#832) (Wenzhe Wang)
- 7233aef linter: Add eslint-plugin-jest/no_alias_method rule (#818) (Wenzhe Wang)
- 3721837 linter: Eslint-plugin-jest/expect-expect (#802) (Wenzhe Wang)
- a44dde5 linter_plugin: Add linter plugin crate (#798) (u9g)

### Bug Fixes

- b89e931 cli: Spawn linting in another thread so diagnostics can be printed immediately (Boshen)
- 2b60b8a linter: Fix incorrect behaviour for "-D correctness -A rule-name" (Boshen)
- 8b24052 linter: No-var-requires not warning if has bindings in ancestors (#799) (Wenzhe Wang)

### Performance

- a969f69 linter: Parse ts-directive manually (#845) (Devin-Yeung)
- 6f270f1 linter: Swap the order of checks for no_caller (#844) (Boshen)

### Refactor

- 6706541 linter: Remove complicated linter service setup (Boshen)
- 91f4896 linter: Clean up Test a bit (Boshen)
- 6931451 linter: Less a global hashmap to reduce rule timer macro expansion (#822) (Boshen)

## [0.0.11] - 2023-08-27

### Features

- 741aa8d ast: Add to ChainExpression and ExpressionArrayElement to ASTKind (#785) (u9g)
- 5921375 cli: Use insta_cmd for cli snapshot testing (#791) (Boshen)
- fd2f8fb linter: Detect import (#778) (Wenzhe Wang)
- 9c50bc0 linter: Implement no-unsafe-declaration-merging (#748) (Makoto Tateno)

### Bug Fixes

- 33ea858 linter: Show the escaped span for no-useless-escape (#790) (Boshen)

### Refactor

- ed9e3e0 linter: Move the label message to help (Boshen)
- 31d5669 linter: Extract `is_valid_jest_call` (#781) (Wenzhe Wang)
- 29f8c02 linter: Clean up tester with fixes (#773) (Boshen)

## [0.0.10] - 2023-08-21

### Bug Fixes

- 58d2d1e cli: Fix a race condition where the program will hang (Boshen)

## [0.0.8] - 2023-08-21

### Features

- 2fde225 linter: Implement eslint-plugin-unicorn/no-instanceof-array (#752) (Kei Sakamoto)
- 3022655 linter: Add no-commented-out-tests (#723) (Wenzhe Wang)
- 607fa6a linter: Implement typescript-eslint/ban-ts-comment (#741) (Devin-Yeung)
- 6f00461 linter: Implement @eslint/no-shadow-restricted-names (#617) (#728) (Lqxc)
- 4f5e4c1 linter: Implement @typescript-eslint/no-duplicate-enum-values (#726) (Kei Sakamoto)
- 0c64517 linter: Valid-describe-callback(eslint-plugin-jest) (#706) (Wenzhe Wang)
- 3adca1c linter: Implement @typescript-eslint/prefer-as-const (#707) (Kei Sakamoto)
- f8358a1 linter: @typescript-eslint/no-namespace (#703) (Alexandr Metreniuc)
- c6245f8 linter: Implement `no-undef` (#672) (Makoto Tateno)
- d1531cd linter: Add no-extra-boolean-cast rule (#677) (Alexandr Metreniuc)
- e7d8d4b linter: Enable module record builder (Don Isaac)
- 3cf08a2 linter: No-focused-test(eslint-jest-plugin) (#609) (Wenzhe Wang)
- 9714e46 resolver: Add tracing (#710) (Boshen)- 8a915ce Vscode extension (#690) (阿良仔)

### Bug Fixes

- de7735d cli: Fix race condition when resolving paths (Boshen)
- ec85fd8 eslint/no-obj-calls: Should resolve non-global binding correctly (#745) (Yunfei He)
- 6eca2ad linter: Change severity of no-obj-calls to warning (Boshen)
- 4032e47 linter: Improve error and help message on no-duplicate-enum-values (Boshen)
- a1c2fa6 linter: Improve help message on no-namespace (Boshen)
- e5d7618 linter: Reduce the span of no-namespace to the keyword (Boshen)
- 77bc913 linter: No-extra-boolean-cast false positive (Boshen)
- a7a834a linter: Fix some race conditions (Boshen)
- ba8dbf5 linter: Fix false positives in loss-of-precision lint (#664) (Devin-Yeung)
- 2f48bdf parser,semantic: Make semantic own `Trivias` (#711) (Boshen)

### Refactor

- 1fdce7e cli: Split out group options (#760) (Boshen)
- 6f1daa6 cli: Clean up lint and cli options (#759) (Boshen)
- 772f71f cli: Add WalkOptions for walk logic (#757) (Boshen)
- a9a6bb8 cli,linter: Move path processing logic from cli to linter (#766) (Boshen)
- 324acfc cli,linter: Move the lint runner from cli to linter (#764) (Boshen)
- 3110490 cli,linter: Move LintOptions from cli to linter (#753) (Boshen)
- 98e4240 linter: Manually declare lint rules because `cargo fmt` breaks (#671) (Boshen)

## [0.0.7] - 2023-07-29

### Features

- 21f8abe cli: Add support for `TIMING` env var (#535) (Shannon Rothe)
- 1bc564e linter: Add style category and change no-empty-interface to style (Boshen)
- 72afdf6 linter: Eslint/no-loss-of-precision (#649) (Devin-Yeung)
- 786cf82 linter: Implement no-global-assign (#624) (SoonIter)
- 252d334 linter: Add a `run_once` callback (#647) (Boshen)
- 453edd2 linter: Eslint/no-empty-character-class (#635) (Sg)
- 52c3c37 linter: Implement no-var-requires (#575) (Makoto Tateno)
- 836d430 linter: Implement `adjacent-overload-signature` (#578) (阿良仔)
- 4b566c0 linter: Implement `no-test-prefixes` (#531) (阿良仔)
- 0346adb linter: Add eslint/no-control-regex (#516) (Don Isaac)
- 5d67094 linter: Implement eslint rule `no-return-await` (#529) (vagusX)
- 1b6fa7b linter: No disabled tests(eslint-jest-plugin) (#507) (Wenzhe Wang)
- 1aaeb79 linter: Implement `no-misused-new` (#525) (阿良仔)- 8fdb6b6 Add eslint/no-obj-calls (#508) (Don Isaac)

### Bug Fixes

- f5c9908 linter: Improve the span for no-inner-declarations (Boshen)
- e4020d6 linter: Change no-control-regex to severity warning (Boshen)
- ad51157 linter: Make disable directives work with plugin rule names (Boshen)
- 06aac50 linter: Change no-var-requires to severity warning (Boshen)
- 6089898 linter: Change severity of `no-this-alias` from error to warning (Boshen)- 357b28e No return await error (#539) (vagusX)

### Performance

- 6628fc8 linter: Reduce mallocs (#654) (Don Isaac)

### Refactor

- d8bfe14 linter: Remove `Box::leak` (#641) (阿良仔)
- 2f8b3f8 linter: Run eq_eq_eq fix in some condition (#545) (Wenzhe Wang)
- cdaff8b linter: Expose LintContext as the API for Linter::run (Boshen)
- 87e65ac semantic: Symbol declarations and references (#594) (Don Isaac)- 318d558 Format code (Matthew "strager" Glazar)- ad00720 Avoid unstable let_chains (Matthew "strager" Glazar)- fbb8aa3 Remove unstable feature const_trait_impl & const_slice_index & slice_group_by (#629) (Sg)

## [0.0.6] - 2023-07-01

### Features

- a2809bf linter: Implement @typescript-eslint/no-unnecessary-type-constraint (Boshen)
- 2e6cb6d linter: Implement @typescript-eslint/no-empty-interface (Boshen)
- 83f69b1 linter: Implement @typescript-eslint/no-non-null-asserted-optional-chain (Boshen)
- 7f300f8 linter: Implement @typescript-eslint/no-extra-non-null-assertion (Boshen)

## [0.0.5] - 2023-07-01

### Features

- 4085a95 linter: Implement no-prototype-builtins (Boshen)
- 9250811 linter: Implement no-useless-escape (Boshen)
- 1aa1129 linter: Implement no-inner-declarations (Boshen)
- c5402c1 linter: Implement no-import-assign (nursery) (Boshen)
- 2f8f974 linter: Implement no-dupe-else-if (Boshen)
- bb9838a linter: Implement no-cond-assign (Boshen)
- 9d16e68 linter: Implement no-self-assign (Boshen)
- 230f1d3 linter: Implement no-unsafe-finally (Boshen)
- 553af9b linter: Implement no-unsafe-optional-chaining (Boshen)
- af3ae9b linter: Implement no-useless-catch (Boshen)

### Bug Fixes

- f9aeebd linter: Fix no_useless_escape crashing on unicode boundaries (Boshen)
- ecdd7bc linter: Fix error message for no_dupe_keys (Boshen)

### Refactor

- 3ec2974 linter: Improve span for no-case-declarations (Boshen)
- 3b22992 linter: Remove redundant backticks from no-constant-binary-expression's error message (Boshen)

## [0.0.4] - 2023-06-28

### Features

- 2be637c linter: Implement no_sparse_arrays (Boshen)
- 87a9ce8 linter: Implement `no-ex-assign` (#495) (阿良仔)
- 2ad1339 linter: Implement require_yield (Boshen)
- a93a876 linter: Implement no_delete_var (Boshen)
- be2200d linter: Implement `no-case-declarations` (#491) (阿良仔)

### Bug Fixes

- 32bffa3 linter: Fix disable directives not working for no_func_assign (Boshen)
- 715698f linter: S/no_function_assign/no_func_assign per eslint (Boshen)
- 072abcc linter: Fix no_empty_pattern broken on rest elements (Boshen)

