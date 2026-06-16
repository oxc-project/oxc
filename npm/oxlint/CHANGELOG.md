# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.70.0] - 2026-06-15

### 🚀 Features

- 2e8bda4 linter/vue: Implement no-dupe-keys rule (#23350) (bab)
- 1490a0a linter/react: Implement react-compiler rule (#23202) (Boshen)
- dd560ae linter/unicorn: Implement `no-array-fill-with-reference-type` rule (#23397) (Mikhail Baev)
- af36c2f linter: Add schema for `react/jsx-curly-brace-presence` (#23400) (WaterWhisperer)
- 47d34a3 linter: Add schema for `react/jsx-handler-names` (#23393) (WaterWhisperer)
- f4250d0 linter: Add schema for `unicorn/import-style` (#23386) (WaterWhisperer)
- 30c74ce linter: Add schema for `jsx_a11y/no-noninteractive-element-to-interactive-role` (#23384) (Sysix)
- cfbe8dc linter: Add schema for `jsx_a11y/no-interactive-element-to-noninteractive-role` (#23382) (WaterWhisperer)
- d15b7ff linter: Add schema for `typescript/no-restricted-types` (#23381) (WaterWhisperer)
- 028a811 linter: Add schema for `jsx-a11y/media-has-caption` (#23377) (Sysix)
- b3b1038 linter: Add schema for `jsx-a11y/label-has-associated-control` (#23376) (Sysix)
- 7ada6b2 linter: Add schema for `jsx_a11y/no-distracting-elements` (#23379) (WaterWhisperer)
- ee3dd49 linter: Add schema for `jsx-a11y/img-redundant-alt` (#23374) (Sysix)
- df5f8dd linter: Add short descriptions to most lint rules. (#23365) (Connor Shea)
- e3fd735 linter: Add schema for `jsx_a11y/alt-text` (#23369) (Sysix)
- 0f2fff4 linter: Add schema for `react/exhaustive-deps` (#23372) (Mikhail Baev)
- e3e4e10 linter: Add schema for `react_perf/jsx-no-new-object-as-prop` (#23368) (Mikhail Baev)
- 9366d44 linter: Add schema for `unicorn/prefer-at` (#23366) (WaterWhisperer)
- f57b55d linter: Add schema for `typescript/array-type` (#23355) (Sysix)
- 0dcf912 linter: Add schema for `typescript/ban-ts-comment` (#23354) (Sysix)
- 51fa83e linter: Add schema for `react/no-did-update-set-state` (#23357) (Mikhail Baev)
- 59db0bd linter: Add schema for `consistent-generic-constructors` (#23353) (Sysix)
- c4775c0 linter: Add schema for `typescript/consistent-type-assertions` (#23349) (Sysix)
- 6e516f7 linter: Add schema for `typescript/consistent-type-imports` (#23348) (Sysix)
- 012134d linter: Add schema for `react/jsx-no-target-blank` (#23345) (WaterWhisperer)
- 0806aae linter: Add schema for `jsx_a11y/no-noninteractive-tabindex` (#23337) (Mikhail Baev)
- 0708b5a linter: Add schema for `react/jsx-filename-extension` (#23315) (Mikhail Baev)
- 150bce1 linter: Add schema for `typescript/no-empty-object-type` (#23309) (Sysix)
- f9e36f1 linter: Add schema for `typescript/no-duplicate-type-constituents` (#23308) (Sysix)
- 937accf linter: Add schema for `typescript/no-invalid-void-type` (#23307) (Sysix)
- 3e042b9 linter: Add schema for `typescript/no-misused-promises` (#23306) (Sysix)
- da212d1 linter: Add schema for `typescript/no-unnecessary-condition` (#23305) (Sysix)
- f8f0d38 linter: Add schema for `typescript/parameter-properties` (#23304) (Sysix)
- 2275fc7 linter: Add schema for `typescript/prefer-nullish-coalescing` (#23302) (Sysix)
- d353858 linter: Add schema for `typescript/prefer-string-starts-ends-with` (#23301) (Sysix)
- 03060f5 linter: Add schema for `typescript/triple-slash-reference` (#23300) (Sysix)
- 6619cee linter: Add schema for `promise/param-names` (#23298) (Sysix)
- 8bf108e linter: Add schema for `promise/catch-or-return` (#23297) (Sysix)
- 48158d0 linter: Add schema for `vitest/consistent-each-for` (#23294) (Sysix)
- 7e74c98 linter: Add schema for `vitest/consistent-test-filename` (#23293) (Sysix)
- ff94d4a linter: Add schema for `vitest/consistent-vitest-vi` (#23292) (Sysix)
- 2409a10 linter: Add schema for `vitest/prefer-import-in-mock` (#23291) (Sysix)
- 3d782b7 linter: Add schema for `react/no-unstable-nested-components` (#23287) (Mikhail Baev)
- 0a0bc2f linter/jsx-a11y: Add `allowedRedundantRoles` option to `no-redundant-roles` (#22820) (bab)
- 80758a5 linter/vue: Implement no-side-effects-in-computed-properties rule (#23282) (bab)
- e3869ac linter: Add schema for `react/no-object-type-as-default-prop` (#23279) (Mikhail Baev)
- 4480609 linter: Add schema for `react/jsx-props-no-spreading` (#23276) (Mikhail Baev)
- 08d68a5 linter/react: Implement `jsx-no-literals` rule (#23145) (kapobajza)
- 9a2788b linter/unicorn: Implement `prefer-export-from` rule (#22935) (AliceLanniste)
- bdb723c linter/unicorn: Implement prefer-single-call rule (#23235) (Yuzhe Shi)
- 31543ed linter: Add schema for `vue/define-props-destructuring` (#23252) (Sysix)
- 21b6c3d linter: Add schema for `oxc/no-async-endpoint-handlers` (#23251) (Sysix)
- e77ff81 linter: Add schema for `unicorn/prefer-object-from-entries` (#23249) (Mikhail Baev)
- bcac2d6 linter: Add schema for `jest/vitest/no-restricted-matchers` (#23247) (Sysix)
- 539f036 linter: Add schema for `jest/vitest/no-restricted-*-methods` (#23246) (Sysix)
- dd1b927 linter/vue: Implement require-default-prop rule (#22951) (bab)
- 3f018e7 linter: Add schema for `unicorn/no-instanceof-builtins` (#23225) (Mikhail Baev)
- e0d0f78 linter: Verify promise/no-callback-in-promise schema (#23141) (beanscg)
- 123d4f4 linter: Add schema for `jest/vitest/valid-expect` (#23185) (Sysix)
- 46c8a21 linter: Add schema for `jest/vitest/require-top-level-describe` (#23184) (Sysix)
- 41465cf linter: Add schema for `jest/vitest/prefer-snapshot-hint` (#23183) (Sysix)
- d068b9b linter: Add schema for `jest/vitest/prefer-expect-assertions` (#23181) (Sysix)
- 064a1ee linter: Add schema for `jest/prefer-ending-with-an-expect` (#23180) (Sysix)
- d046797 linter: Add schema for `jest/vitest/no-standalone-expect` (#23179) (Sysix)
- 137b9a6 linter: Add schema for `jest/vitest/no-large-snapshots` (#23178) (Sysix)
- 0f3e4a5 linter: Add schema for `jest/vitest/no-hooks` (#23177) (Sysix)
- cd0b384 linter: Add schema for `unicorn/explicit-length-check` (#23155) (Mikhail Baev)
- 01b74c4 linter: Add schema for `jest/no-deprecated-functions` (#23136) (Sysix)
- 9d6a387 linter: Add schema for `unicorn/catch-error-name` (#23137) (Mikhail Baev)
- 0da8efa linter: Add schema for `jest/vitest/max-nested-describe` (#23131) (Sysix)
- d71c9fd linter: Add schema for `eslint/no-use-before-define` (#23129) (Sysix)

### 🐛 Bug Fixes

- 26ddac6 linter: Avoid config schema generation for `jsx_a11y/no-noninteractive-element-interactions` (#23385) (Sysix)
- 40556ad linter: Parse `jsx-a11y/control-has-associated-label` config with `DefaultRuleConfig` (#23373) (Sysix)
- 71e9648 linter: Expose no-noninteractive-element-interactions schema (#23283) (camc314)
- 6c86d1c linter/react-perf: Correct nativeAllowList all schema (#23229) (camc314)
- 4dd52de linter/react-perf: Re-generate stale snapshots (#23228) (camc314)
- 8f3db61 linter: Allow options for `eslint/capitalized-comments` (#23139) (Sysix)

### ⚡ Performance

- f09707e linter: `jest/no-deprecated-functions` store config version as `usize` (#23138) (Sysix)

### 📚 Documentation

- f682e25 linter: Remove manually written options doc for `eslint/prefer-arrow-callback` (#23438) (Mikhail Baev)
- 64c942c linter: Remove manually written options doc for `eslint/no-sequences` (#23420) (Mikhail Baev)
- 14abf32 linter/react-perf: Use autogenerated docs (#23227) (camc314)

## [1.69.0] - 2026-06-08

### 🚀 Features

- e805174 linter: Add schema for `jest/vitest/max-expects` (#23105) (Sysix)
- 7850577 linter: Add schema for `jest/vitest/expect-expect` (#23104) (Sysix)
- 75f641a linter: Add schema for `jest/vitest/consistent-test-it` (#23103) (Sysix)
- 5125f89 linter/unicorn: Support no-null `checkArguments` option (#23098) (camc314)
- b8b9797 linter: Add schema for `import-max-dependencies` (#23096) (Sysix)
- 65cb47a linter/eslint: Support no-unused-expressions `ignoreDirectives` option (#23097) (camc314)
- f6c36d5 linter: Add schema for `import/prefer-default-export` (#23091) (Sysix)
- 0d4a5d1 linter: Add schema for `eslint/sort-vars` (#23090) (Sysix)
- fdb5bf5 linter: Add schema for `eslint/radix` (#23082) (Sysix)
- 05b4dcf linter: Add schema for `eslint/prefer-const` (#23081) (Sysix)
- 5a06c4d linter/vue: Implement next-tick-style rule (#23041) (Alex Peshkov)
- e38a36a linter: Add schema for `eslint/operator-assignment` (#23080) (Sysix)
- 907cee7 linter: Add schema for `eslint/no-warning-comments` (#23075) (Sysix)
- 9470bb2 linter: Add schema for `eslint/no-unused-vars` (#23073) (Sysix)
- 234b5cf linter: Add schema for `eslint/no-shadow` (#23072) (Sysix)
- de0dd8b linter: Add schema for `eslint/no-restricted-exports` (#23020) (Sysix)
- faa3e0d linter: Add schema for `eslint/no-param-reassign` (#23018) (Sysix)
- dbc9c27 linter: Add schema for `eslint/no-magic-numbers` (#23017) (Sysix)
- 38d3569 linter: Add schema for `eslint/no-inner-declarations` (#23016) (Sysix)
- 008fa41 linter: Add schema for `eslint/no-constant-condition` (#22991) (Sysix)
- ca44623 linter: Add schema for `eslint/no-empty-function` (#22988) (Sysix)
- 43eb04d linter: Add schema for `eslint/id-match` (#22987) (Sysix)
- a800f27 linter: Add schema for `eslint/capitalized-comments` (#22984) (Sysix)
- 96e2d32 linter: Add schema for `eslint/id-length` (#22963) (Sysix)
- 545493f linter: Add schema for `eslint/complexity` (#22960) (Sysix)
- 5f0b558 linter: Add schema for `eslint/class-methods-use-this` (#22959) (Sysix)
- 719b720 linter: Add schema for simple rule configurations (#22948) (Sysix)
- fd00966 linter: Add right schema for `eslint/max-*` rules (#22923) (Sysix)
- 1226d78 linter: Fill schema with rule configurations (#22907) (Sysix)
- 8f423c1 linter/vue: Implement `require-direct-export` rule (#17623) (yefan)
- 78e915b linter/vue: Implement no-reserved-props rule (#22914) (bab)
- 0f200a9 linter/vue: Implement require-prop-types rule (#22083) (Alex Peshkov)
- 5da9da9 linter/vue: Implement no-reserved-keys rule (#21780) (bab)
- 75e14a8 linter/vue: Implement prop-name-casing rule (#22892) (bab)

### 🐛 Bug Fixes

- 0383e61 linter: Fix schema for rules without a config (#22946) (Sysix)

### 📚 Documentation

- dadafe3 oxlint, oxfmt: Mention migrate skills in npm READMEs (#22965) (Boshen)

## [1.68.0] - 2026-06-01

### 🚀 Features

- e4b1f46 linter/typescript: Implement `method-signature-style` rule (#22679) (Mikhail Baev)
- bc462ca linter/vue: Implement no-reserved-component-names rule (#22741) (bab)
- ef9e751 linter/vue: Implement component-definition-name-casing rule (#22818) (bab)
- d67f51a linter/vue: Implement require-prop-type-constructor rule (#22708) (bab)
- 8422e8b linter/jsdoc: Implement `require-yields-description` rule (#22805) (Mikhail Baev)
- fe93f97 linter/eslint: Implement `prefer-named-capture-group` rule (#22759) (Sebastian Poxhofer)

## [1.67.0] - 2026-05-26

### 🚀 Features

- b84941e linter/vue: Implement no-expose-after-await rule (#22675) (bab)
- 98b98c1 linter/vue: Implement no-computed-properties-in-data rule (#22674) (bab)
- 2d4c919 oxlint: Support `vite-plus/resolveConfig` for vite.config.ts (#22456) (leaysgur)
- 2a60012 linter/vue: Implement require-render-return rule (#22613) (bab)
- 9f227fd linter/vue: Implement no-deprecated-props-default-this rule (#21892) (bab)
- 87f065e linter/vue: Implement return-in-emits-validator rule (#21935) (bab)
- ea0380c linter/unicorn: Implement `import-style` rule (#22173) (Hao Chen)
- dde40fe linter/vue: Implement no-watch-after-await rule (#22006) (bab)
- a735eb0 linter/vue: Implement valid-next-tick rule (#22531) (bab)
- 6dc615d linter/vue: Implement no-shared-component-data rule (#21842) (bab)
- a656418 linter/vue: Implement valid-define-options rule (#22107) (bab)
- bb6f1b2 linter/vue: Implement require-slots-as-functions rule (#22244) (bab)
- 5fa4774 linter/n: Implement `callback-return` rule (#22470) (Mikhail Baev)

## [1.66.0] - 2026-05-18

### 🚀 Features

- 0440b0f linter/eslint: Implement `id-match` rule (#22379) (Vladislav Sayapin)
- 65bf119 linter: Implement react no-object-type-as-default-prop (#22481) (uhyo)
- 2a6ddce linter/eslint: Implement `no-implied-eval` rule (#22391) (Vladislav Sayapin)
- 625758a linter/vitest: Implement padding-around-after-all-blocks rule (#21788) (kapobajza)
- 37680b0 linter: Implement react no-unstable-nested-components (#22248) (Jovi De Croock)
- d8d9c74 linter: Implement import/newline-after-import rule (#19142) (Ryuya Yanagi)

## [1.65.0] - 2026-05-15

### 🚀 Features

- 5478fb5 linter/jsdoc: Implement `require-throws-description` rule (#22386) (Mikhail Baev)
- c73225e linter/eslint: Implement `prefer-arrow-callback` rule (#22312) (박천(Cheon Park))
- de82b59 linter: Add support for `eslint-plugin-jsx-a11y-x` (#22356) (mehm8128)
- f44b6c8 linter: Fill schemas `DummyRuleMap` with built-in rules (#22288) (Sysix)

## [1.64.0] - 2026-05-11

### 🚀 Features

- fbb8f22 linter: Support `ignores` in overrides (#22148) (camc314)

### 🐛 Bug Fixes

- 25b7017 linter: Undocument override `ignores` option (#22213) (camc314)

## [1.63.0] - 2026-05-05

### 📚 Documentation

- cacbc4a linter: Fix jest settings docs. (#22127) (connorshea)

## [1.62.0] - 2026-04-27

### 🚀 Features

- 348f46c linter: Add `respectEslintDisableDirectives` option (#21384) (Christian Vuerings)

### 🐛 Bug Fixes

- 8c425db linter: Allow string for jest version in config schema (#21649) (camc314)

## [1.61.0] - 2026-04-20

### 🚀 Features

- 38d8090 linter/jest: Implemented jest `version` settings in config file. (#21522) (Said Atrahouch)

## [1.60.0] - 2026-04-13

### 📚 Documentation

- cfd8a4f linter: Don't rely on old eslint doc for available globals (#21334) (Nicolas Le Cam)

## [1.59.0] - 2026-04-06

### 🐛 Bug Fixes

- dd2df87 npm: Export package.json for oxlint and oxfmt (#20784) (kazuya kawaguchi)

## [1.58.0] - 2026-03-30

### 🚀 Features

- 16516de linter: Enhance types for `DummyRule` (#20751) (camc314)

### 📚 Documentation

- be3dcc1 linter: Add note about node version + custom TS plugin (#19381) (camc314)

## [1.55.0] - 2026-03-12

### 🐛 Bug Fixes

- bc20217 oxlint,oxfmt: Omit useless `| null` for `Option<T>` field from schema (#20273) (leaysgur)

### 📚 Documentation

- f339f10 linter/plugins: Promote JS plugins to alpha status (#20281) (overlookmotel)

## [1.54.0] - 2026-03-12

### 📚 Documentation

- 0c7da4f linter: Fix extra closing brace in example config. (#20253) (connorshea)

## [1.52.0] - 2026-03-09

### 🚀 Features

- 61bf388 linter: Add `options.reportUnusedDisableDirectives` to config file (#19799) (Peter Wagenet)
- 2919313 linter: Introduce denyWarnings config options (#19926) (camc314)
- a607119 linter: Introduce maxWarnings config option (#19777) (camc314)

### 📚 Documentation

- 6c0e0b5 linter: Add oxlint.config.ts to the config docs. (#19941) (connorshea)
- 160e423 linter: Add a note that the typeAware and typeCheck options require oxlint-tsgolint (#19940) (connorshea)

## [1.51.0] - 2026-03-02

### 🚀 Features

- f34f6fa linter: Introduce typeCheck config option (#19764) (camc314)
- 694be7d linter: Introduce typeAware as config options (#19614) (camc314)

### 🐛 Bug Fixes

- 04e6223 npm: Add `preferUnplugged` for Yarn PnP compatibility (#19829) (Boshen)

### 📚 Documentation

- 2fa936f README.md: Map npm package links to npmx.dev (#19666) (Boshen)

## [1.45.0] - 2026-02-10

### 🐛 Bug Fixes

- 1b2f354 ci: Add missing riscv64/s390x napi targets for oxfmt and oxlint (#19217) (Cameron)

## [1.44.0] - 2026-02-10

### 🚀 Features

- ee2925b oxlint/lsp: Enable JS plugins (#18834) (overlookmotel)
- 9788a96 oxlint,oxfmt: Add more native builds (#18853) (Boshen)

### 📚 Documentation

- 9561e7f linter/plugins: Alter JS plugins example (#18900) (overlookmotel)
- b425a0c linter: Document jsPlugins examples (#18671) (Cameron)
- df2b7fa linter: Expand settings example with reference to custom plugins (#18670) (camc314)

## [1.42.0] - 2026-01-26

### 🚀 Features

- 15d69dc linter: Implement react/display-name rule (#18426) (camchenry)

### 📚 Documentation

- 8ccd853 npm: Update package homepage URLs and add keywords (#18509) (Boshen)

## [1.41.0] - 2026-01-19

### 📚 Documentation

- 8a294d5 oxfmt, oxlint: Update logo (#18242) (Dunqing)

## [1.37.0] - 2026-01-05

### 💥 BREAKING CHANGES

- f7da875 oxlint: [**BREAKING**] Remove oxc_language_server binary (#17457) (Boshen)

### 📚 Documentation

- 7e5fc90 linter: Update list of plugins that are reserved. (#17516) (connorshea)

## [1.35.0] - 2025-12-22

### 🚀 Features

- 9e624c9 linter/react: Add `version` to `ReactPluginSettings` (#17169) (camc314)

## [1.34.0] - 2025-12-19

### 🚀 Features

- a0f74a0 linter/config: Allow aliasing plugin names to allow names the same as builtin plugins (#15569) (Cameron)

### 🐛 Bug Fixes

- 005ec25 linter: Permit `$schema` `.oxlintrc.json` struct (#17060) (Copilot)
- d446c43 linter: Prevent extra fields from being present on oxlint config file (#16874) (connorshea)

## [1.30.0] - 2025-11-24

### 🚀 Features

- 595867a oxlint: Generate markdownDescription fields for oxlint JSON schema. (#15959) (connorshea)

## [1.29.0] - 2025-11-17

### 🚀 Features

- 84de1ca oxlint,oxfmt: Allow comments and also commas for vscode-json-ls (#15612) (leaysgur)

## [1.26.0] - 2025-11-05

### 🚀 Features

- 26f24d5 linter: Permit comments in `.oxlintrc.json` via json schema file (#15249) (Martin Leduc)

### 🐛 Bug Fixes

- d6996d0 linter: Fix JSON schema to deny additional properties for categories enum. (#15257) (Connor Shea)
- 9304f9f linter: Fix JSON schema to deny additional properties for plugins enum. (#15259) (Connor Shea)

### 📚 Documentation

- 84ef5ab linter: Avoid linebreaks for markdown links and update plugins docs in the configuration schema. (#15246) (Connor Shea)


## [1.25.0] - 2025-10-30

### 🚀 Features

- bd74603 linter: Add support for vitest/valid-title rule (#12085) (Tyler Earls)


## [1.24.0] - 2025-10-22

### 🐛 Bug Fixes

- 28e76ec oxlint: Resolving JS plugin failing when `extends` is used (#14556) (camc314)




## [1.21.0] - 2025-10-08

### 🐛 Bug Fixes

- 6e8d2f6 language_server: Ignore JS plugins (#14379) (overlookmotel)



## [1.19.0] - 2025-09-29

### 🚀 Features

- b4d716f linter/plugins: Move custom JS plugin config to `jsPlugins` (#14133) (overlookmotel)

### 🐛 Bug Fixes

- 8879b5a linter/plugins: Add types export to `npm/oxlint` (#14219) (overlookmotel)



## [1.17.0] - 2025-09-23

### 🚀 Features

- 3e117c6 linter/plugins: Add `defineRule` API (#13945) (overlookmotel)
- a14aa79 npm/oxlint: Convert to ES modules (#13876) (Boshen)
- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- 53d04dd linter: Convert `oxlint` to NAPI app (#13723) (overlookmotel)

### 🚜 Refactor

- bb040bc parser, linter: Replace `.mjs` files with `.js` (#14045) (overlookmotel)
- 7e0d736 linter/plugins: Rename `--experimental-js-plugins` to `--js-plugins` (#13860) (overlookmotel)




## [1.14.0] - 2025-08-30

### 🚀 Features

- 7fc4aef npm/oxlint: 'oxlint-tsgolint': '>=0.1.4' (Boshen)


## [1.13.0] - 2025-08-26

### 🐛 Bug Fixes

- 02c779f npm/oxlint: Make `oxlint-tsgolint` truly optional (#13153) (Boshen)




## [1.11.1] - 2025-08-09

### 🐛 Bug Fixes

- 8c57153 npm/oxlint: Fix `oxlint-tsgolint` version range for yarn (Boshen)

### 🚜 Refactor

- 238b183 linter: Use `fast-glob` instead of `globset` for `GlobSet` (#12870) (shulaoda)


## [1.11.0] - 2025-08-07

### 🚀 Features

- ac46347 oxlint: Add `tsgolint` integration (#12485) (camchenry)


## [1.10.0] - 2025-08-06

### 🚀 Features

- 9b35600 linter/jsx-a11y: Add support for mapped attributes in label association checks (#12805) (camc314)


## [1.9.0] - 2025-07-29

### 🚜 Refactor

- bea652f linter: Add `vue` and `regex` to `BuiltinLintPlugins` (#12542) (Sysix)



## [1.7.0] - 2025-07-16

### 🚀 Features

- a4dae73 linter: Introduce `LintPlugins` to store builtin + custom plugins (#12117) (camc314)










# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.16.12] - 2025-05-25

### Features

- 6a7018e linter: Generate stricter json schema for lint plugins (#11219) (camc314)

### Bug Fixes

- e8470d9 linter: Delay merging of oxlintrc configs (#10835) (camc314)

## [0.15.14] - 2025-03-11

### Features

- 3fce826 linter: Add support for `extends` property in oxlintrc (#9217) (camchenry)

## [0.15.13] - 2025-03-04

### Documentation

- 24850e7 linter: Add example of how configure rule (#9469) (Cédric DIRAND)

## [0.15.11] - 2025-02-16

### Features

- 5d508a4 linter: Support `env` and `globals` in `overrides` configuration (#8915) (Sysix)

## [0.15.8] - 2025-01-24

### Features

- 79ba9b5 linter: Added support to run in Node.JS legacy versions (#8648) (Luiz Felipe Weber)

## [0.15.7] - 2025-01-19

### Features

- 538b24a linter: Format the configuration documentation correctly (#8583) (Tapan Prakash)

## [0.14.0] - 2024-12-01

### Features

- 32f860d linter: Add support for ignorePatterns property within config file (#7092) (Nicholas Rayburn)

### Documentation

- a6b0100 linter: Fix config example headings (#7562) (Boshen)

## [0.13.0] - 2024-11-21

### Documentation

- df143ca linter: Add docs for config settings (#4827) (DonIsaac)

## [0.12.0] - 2024-11-20

### Features

- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)

## [0.11.0] - 2024-11-03

### Documentation

- 4551baa linter: Document `rules` (#6983) (Boshen)

## [0.10.3] - 2024-10-26

### Documentation

- 3923e63 linter: Add schema to config examples (#6838) (Dmitry Zakharov)

## [0.10.0] - 2024-10-18

### Features

- 6e3224d linter: Configure by category in config files (#6120) (DonIsaac)

## [0.9.9] - 2024-09-27

### Bug Fixes

- 01b9c4b npm/oxlint: Make bin/oxc_language_server an executable (#6066) (Boshen)

## [0.9.7] - 2024-09-23

### Refactor

- ba7b01f linter: Add `LinterBuilder` (#5714) (DonIsaac)

## [0.9.6] - 2024-09-18

### Refactor

- a438743 linter: Move `OxlintConfig` to `Oxlintrc` (#5707) (DonIsaac)

## [0.9.4] - 2024-09-12

### Features

- 023c160 linter: Impl `Serialize` for `OxlintConfig` (#5594) (DonIsaac)

## [0.9.3] - 2024-09-07

### Styling
- 694f032 Add trailing line breaks to `package.json` files (#5542) (overlookmotel)

## [0.8.0] - 2024-08-23

### Features

- a0effab linter: Support more flexible config.globals values (#4990) (Don Isaac)

## [0.7.2] - 2024-08-15

### Features

- 4d28d03 task/website: Support render `subschemas.all_of` (#4800) (mysteryven)

## [0.7.1] - 2024-08-12

### Features

- cc922f4 vscode: Provide config's schema to oxlint config files (#4826) (Don Isaac)

## [0.7.0] - 2024-08-05

### Bug Fixes

- 0fba738 npm: SyntaxError caused by optional chaining in low version node (#4650) (heygsc)

## [0.6.0] - 2024-07-11

### Features

- cc58614 linter: Better schemas for allow/warn/deny (#4150) (DonIsaac)

## [0.4.2] - 2024-05-28

### Bug Fixes

- 19bb1c0 website: Hack `schemars` to render code snippet in markdown (#3417) (Boshen)

## [0.3.2] - 2024-05-04

### Bug Fixes

- dcda1f6 cli: Update `--format` documentation (#3118) (Vasilii A)

## [0.3.0] - 2024-04-22

### Refactor

- 5241e1e cli: Improve `--help` documentation (Boshen)

## [0.2.8] - 2024-02-06

### Features

- 839e7c5 napi/parser: Add more linux-musl targets (Boshen)

## [0.2.7] - 2024-02-03

### Features

- 0ae28dd npm/oxlint: Display target triple when error is thrown (#2259) (Boshen)

## [0.2.4] - 2024-01-23

### Bug Fixes

- 382a187 npm: Fix bin script for musl / gnu (Boshen)

## [0.2.3] - 2024-01-23

### Features
- 20a34b5 Introduce --react-perf-plugin CLI flag, update rules to correctness (#2119) (Hulk)

## [0.0.17] - 2023-11-09

### Features

- d82ba5b cli: Run oxlint with no file arguments (#1201) (Boshen)

## [0.0.9] - 2023-08-21

### Bug Fixes

- e14dd06 npm: Add package.repository and other fields according to provernance (Boshen)
- f7d2675 npm: Fix github link according to provernance (Boshen)

