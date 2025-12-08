# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.102.0] - 2025-12-08

### 💥 BREAKING CHANGES

- 083fea9 napi/parser: [**BREAKING**] Represent empty optional fields on JS side as `null` (#16411) (overlookmotel)

## [0.100.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 934d873 napi: [**BREAKING**] Drop `armv7-unknown-linux-musleabihf` support (#16105) (Boshen)

## [0.98.0] - 2025-11-17

### 💥 BREAKING CHANGES

- ea51b0b napi: [**BREAKING**] Standardize function naming with sync suffixes (#15661) (Boshen)

### 🚀 Features

- f5ce55a napi: Export all options using wildcard exports (Boshen)

## [0.97.0] - 2025-11-11

### 🚀 Features

- 1c31cb1 napi/minify: Expose `treeshake` options (#15109) (copilot-swe-agent)

### 📚 Documentation

- 3dc24b5 linter,minifier: Always refer as "ES Modules" instead of "ES6 Modules" (#15409) (sapphi-red)

## [0.96.0] - 2025-10-30

### 🚀 Features

- 9e36186 napi/minify: Expose `drop_labels` option (#14635) (sapphi-red)
- d09c7ee minifier: Add `drop_labels` feature (#14634) (sapphi-red)


## [0.95.0] - 2025-10-15

### 🚀 Features

- c19c9ec napi/minify: Expose join_vars, sequences, and max_iterations options (#14545) (IWANABETHATGUY)

### 🐛 Bug Fixes

- 32a41cf napi/minify: S/passes/max_iterations to avoid confusion (#14608) (Boshen)
- 14686a4 napi/minify: Handle boolean values for `compress.unused` option (#14513) (Kentaro Suzuki)






## [0.92.0] - 2025-09-24

### 🐛 Bug Fixes

- 2f9e16d napi/minifier, napi/transformer: Rename CommonJS file to `.cjs` (#14047) (overlookmotel)

### 🚜 Refactor

- cc0019f napi: Move scripts into `scripts` directory (#14048) (overlookmotel)


## [0.92.0] - 2025-09-24

### 🐛 Bug Fixes

- 2f9e16d napi/minifier, napi/transformer: Rename CommonJS file to `.cjs` (#14047) (overlookmotel)

### 🚜 Refactor

- cc0019f napi: Move scripts into `scripts` directory (#14048) (overlookmotel)


## [0.91.0] - 2025-09-22

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🐛 Bug Fixes

- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-22

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🐛 Bug Fixes

- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-21

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🐛 Bug Fixes

- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)


## [0.91.0] - 2025-09-21

### 💥 BREAKING CHANGES

- 6fcb0d0 minifier: [**BREAKING**] Receive supported engines instead of ecmascript versions (#13933) (sapphi-red)

### 🐛 Bug Fixes

- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)


## [0.90.0] - 2025-09-18

### 🚀 Features

- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)

### 🐛 Bug Fixes

- 9796ec1 napi: Fix binding files (Boshen)


## [0.90.0] - 2025-09-18

### 🚀 Features

- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)

### 🐛 Bug Fixes

- 9796ec1 napi: Fix binding files (Boshen)














## [0.83.0] - 2025-08-29

### 💥 BREAKING CHANGES

- 34d0a01 napi/minify,transform: [**BREAKING**] Change module type to ESM (#13349) (Boshen)

### 🚀 Features

- 593f54c minifier: Add `--max-iterations` for debugging (#13291) (sapphi-red)
- a0e0a91 oxc_minify_napi: Expose `CodeGenOptions` (#13288) (sapphi-red)


## [0.83.0] - 2025-08-29

### 💥 BREAKING CHANGES

- 34d0a01 napi/minify,transform: [**BREAKING**] Change module type to ESM (#13349) (Boshen)

### 🚀 Features

- 593f54c minifier: Add `--max-iterations` for debugging (#13291) (sapphi-red)
- a0e0a91 oxc_minify_napi: Expose `CodeGenOptions` (#13288) (sapphi-red)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- f10ac33 codegen: Remove end sourcemaps for `}`, `]`, `)` (#13180) (Boshen)


## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- f10ac33 codegen: Remove end sourcemaps for `}`, `]`, `)` (#13180) (Boshen)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- 5223562 codegen: Adjust some source mappings (#13084) (Boshen)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- 5223562 codegen: Adjust some source mappings (#13084) (Boshen)


## [0.82.1] - 2025-08-13

### 🚀 Features

- 993db89 minifier: `.minify` and `.dce` methods; run dce in loop (#13026) (Boshen)


## [0.82.1] - 2025-08-13

### 🚀 Features

- 993db89 minifier: `.minify` and `.dce` methods; run dce in loop (#13026) (Boshen)






## [0.80.0] - 2025-08-03

### 🚀 Features

- 2093f65 napi/minify: Make `MinifyOptions` pub (#12753) (Boshen)

### 🧪 Testing

- 0ec214b napi: Compile tests in debug mode (#12750) (overlookmotel)


## [0.80.0] - 2025-08-03

### 🚀 Features

- 2093f65 napi/minify: Make `MinifyOptions` pub (#12753) (Boshen)

### 🧪 Testing

- 0ec214b napi: Compile tests in debug mode (#12750) (overlookmotel)


## [0.79.1] - 2025-07-31

### 🚀 Features

- a286dd4 minifier: Remove unnecessary 'use strict' directive (#12642) (Boshen)
- 75cf797 napi/minify: Publish crate (#12611) (Boshen)


## [0.79.1] - 2025-07-31

### 🚀 Features

- a286dd4 minifier: Remove unnecessary 'use strict' directive (#12642) (Boshen)
- 75cf797 napi/minify: Publish crate (#12611) (Boshen)



## [0.78.0] - 2025-07-24

### 🚜 Refactor

- 1cf08c0 minifier: Make DCE remove more code to align with rollup (#12427) (Boshen)


## [0.77.3] - 2025-07-20

### ⚡ Performance

- 8bae417 codegen: Remove the useless tokens generated by some expressions (#12394) (Boshen)



## [0.77.1] - 2025-07-16

### 🚀 Features

- 1b80633 minifier: Remove unused function declaration (#12318) (Boshen)
- 3f33e8c minifier: Remove unused assignment expression (#12314) (Boshen)
- fb8289c minifier: Remove unused variable declaration (#11796) (Boshen)

### 🐛 Bug Fixes

- cd98426 semantic: Handle var hoisting in catch block with same catch parameter name (#12313) (Dunqing)


## [0.77.0] - 2025-07-12

### 🚜 Refactor

- baa3726 tests/napi: Add `build-test` script for tests (#12132) (camc314)


## [0.76.0] - 2025-07-08

### 🚀 Features

- 395aa5e napi/minify: Return parse errors (#12112) (Boshen)




## [0.74.0] - 2025-06-23

### 💥 BREAKING CHANGES

- 7a05e71 minifier: [**BREAKING**] Add `Treeshake` options (#11786) (Boshen)



## [0.73.1] - 2025-06-17

### 🚀 Features

- 81ef443 napi: Add `aarch64-linux-android` target (#11769) (LongYinan)


## [0.73.0] - 2025-06-13

### 📚 Documentation

- b5a6a6e napi: Add stackblitz examples (Boshen)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Features

- 1401839 napi: Add webcontainer fallback to transform and minify packages (#11471) (Boshen)

### Bug Fixes

- ab0dd29 napi: Napi build cache problem (#11479) (LongYinan)

## [0.71.0] - 2025-05-20

### Features

- d67c9e5 napi: Bump napi to beta (#11159) (Boshen)

### Bug Fixes

- 963167d napi: Fix cfg feature on global_allocator (Boshen)

## [0.70.0] - 2025-05-15

### Features

- 647b6f3 napi: Add arm musl (#10958) (Bernd Storath)

### Bug Fixes

- 584d8b9 napi: Enable mimalloc `no_opt_arch` feature on linux aarch64 (#11053) (Boshen)

## [0.69.0] - 2025-05-09

### Features

- 22ba60b napi: Add `s390x-unknown-linux-gnu` build (#10892) (Boshen)
- 308fe73 napi: Add `x86_64-unknown-freebsd` and `riscv64gc-unknown-linux-gnu` builds (#10886) (Boshen)

## [0.68.1] - 2025-05-04

### Bug Fixes

- bd953fc napi/minify: Need to remove all comments (#10785) (Boshen)

## [0.68.0] - 2025-05-03

### Features

- b01cb45 codegen: A way to keep legal comments after minification (#10689) (Boshen)

## [0.61.2] - 2025-03-23

### Features

- ea3de06 mangler: Support `keep_names` option (#9898) (sapphi-red)

## [0.61.0] - 2025-03-20

### Features

- dcd356e minifier: Support `keep_names` option (#9867) (sapphi-red)
- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)

### Testing

- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.59.0] - 2025-03-18

### Performance

- 84fa538 minify: Use mimalloc-safe to replace mimalloc (#9810) (LongYinan)

## [0.58.0] - 2025-03-13

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

