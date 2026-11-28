# NAPI profile-guided optimization

Parser, transformer, and minifier releases train their native Linux GNU x64/ARM64,
macOS x64/ARM64, and Windows x64 bindings on ecosystem and npm source
corpus. Other targets use their existing release builds.

```sh
rustup component add llvm-tools-preview
just pgo prepare
just pgo all
```

To train packages separately after preparation:

```sh
just pgo parser --corpus target/pgo/corpus
just pgo transform --corpus target/pgo/corpus
just pgo minify --corpus target/pgo/corpus
```

Preparation clones `oxc-project/oxc-ecosystem-ci` and `oxc-project/monitor-oxc`,
initializes the ecosystem matrices and monitor runtime repos, and installs each
project's dependencies with its package manager. Lifecycle scripts are skipped,
and Yarn uses `node_modules` so dependency sources can be collected. Installation
failures are reported and recorded in the manifest; available sources are still
collected.

All JS/TS/JSX files are collected, including installed dependencies, hidden package
stores, and build output. Symlinks are skipped to avoid collecting linked packages
repeatedly. Preparation prints file counts for each extension. Supported TS/JSX is
lowered to JavaScript for minifier training. Inputs with diagnostics or no runtime
output still train parser and transformer.

Preparation identifies JavaScript bundles by `.bundle` or `.chunk` filename
suffixes and by adjacent source maps with multiple inputs for files at least
16 KiB. Files named with a `.min` suffix still enter the ordinary corpus.
Recognized bundles get their own rotation and count in the manifest.

The corpus lives in `target/pgo/corpus` as gzip-compressed JSONL shards. Its manifest
records source revisions, counts, and checksums, and must match the current Oxc
commit. `--corpus` selects another directory. Checkouts live in a temporary
directory outside Oxc's workspace; individual training repos are removed after
collection to bound disk usage.

## Workloads

- **Parser.** Default JSON transfer is used on three quarters of the inputs,
  including calls that request ranges, omit parenthesis nodes, or request a
  TypeScript-shaped AST for JavaScript. The rest use
  raw transfer with and without semantic checks. This covers AST consumers
  such as unimport, ranges in Angular CLI, TypeScript parenthesis handling in
  prettier-oxc-parser, and raw transfer in Knip and Storybook. The default
  wrapper still runs Rust AST serialization and native-to-JS transfer. Its
  lazy `JSON.parse` runs in the smoke check because it contributes no Rust
  profile counters.
  [unimport](https://github.com/unjs/unimport/blob/main/src/detect-oxc.ts) ·
  [Angular CLI](https://github.com/angular/angular-cli/blob/main/packages/angular/build/src/tools/oxc/oxc-transform.ts#L264) ·
  [prettier-oxc-parser](https://github.com/ArnaudBarre/prettier-oxc-parser/blob/main/src/oxcParse.ts#L13-L19) ·
  [Knip](https://github.com/webpro-nl/knip/blob/main/packages/knip/src/typescript/ast-nodes.ts) ·
  [Storybook](https://github.com/storybookjs/storybook/blob/next/code/core/src/csf-tools/estree/ast.ts) ·
  [React Doctor](https://github.com/millionco/react-doctor/blob/main/packages/oxlint-plugin-react-doctor/src/plugin/utils/parse-source-file.ts#L51-L73)
- **Transformer.** Each file runs one workload. The rotation covers syntax
  stripping with and without source maps, browser lowering, development JSX,
  JSX preservation, and explicit type-only import removal. TypeScript files
  also train standalone declarations and combined JavaScript and declaration
  emit. Inputs containing `accessor` fall back to the default target when
  scheduled for lowering.
  [Remix Node TSX](https://github.com/remix-run/remix/blob/main/packages/node-tsx/src/lib/transform.ts) ·
  [Qwik](https://github.com/QwikDev/qwik/blob/main/packages/ts-optimizer/src/optimizer/edit/strip-types.ts#L95-L106) ·
  [Naive UI](https://github.com/tusen-ai/naive-ui/blob/main/build/utils/tsToJs.ts#L7-L10) ·
  [tschema](https://github.com/lukeed/tschema/blob/main/scripts/build.ts) ·
  [Hrana](https://github.com/lukeed/hrana/blob/main/scripts/build.ts#L66-L84)
- **Minifier.** The rotation covers default compression and mangling, source
  maps, and an ES2019 compression target with name preservation. Recognized
  bundles rotate through minification with license comments, readable
  code generation with compression and mangling disabled, and ASCII
  minification targeting ES2015. The trainer preserves input module or script
  mode and explicitly sets `module` for known ES modules.
  [unplugin-oxc](https://github.com/unplugin/unplugin-oxc/blob/main/src/index.ts) ·
  [Jco](https://github.com/bytecodealliance/jco/blob/main/packages/jco-transpile/src/transpile.ts) ·
  [Paper.js](https://github.com/paperjs/paper.js/blob/develop/build.mjs#L31-L51) ·
  [Devjar](https://github.com/huozhi/devjar/blob/main/src/cli/modules.ts#L335-L345)

The rotations are coverage choices based on these callers. They continue across
shard boundaries and sample synchronous and asynchronous entry points
independently of the workload. Training processes source as data. Diagnostics
are counted, and crashes, corrupt inputs, and empty profiles fail the build.

Preparation and training reserve a 512 MiB worker stack for deeply nested inputs.
Async calls use small inputs because libuv stacks have a fixed size. Training
processes exit before the pinned Rust toolchain's `llvm-profdata` merges profiles.
Profiles are kept separate by package, target, and release commit.

The optimized binding is checked through each workload's synchronous and
asynchronous public APIs before replacing the normal release artifact.
The profile metadata records how many inputs exercised each workload.
Bindings, profiles, and provenance are stored in
`target/pgo/<package>/<target>/`. Full binding suites run in project CI.

## Release dry runs

Run **Release NAPI** in GitHub Actions, select the branch/tag, and choose a
package. The workflow prepares the corpus once and shares it between native target
jobs. Manual runs build the complete release matrix, assemble the normal
platform packages, and pack them with pnpm without publishing or requiring a
version bump. Download the normal package tarballs from `release-<package>` and
PGO provenance from `pgo-<package>-<target>`. Automatic releases retain their version
check and trusted publishing.
