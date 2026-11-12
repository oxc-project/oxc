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

The corpus lives in `target/pgo/corpus` as gzip-compressed JSONL shards. Its manifest
records source revisions, counts, and checksums, and must match the current Oxc
commit. `--corpus` selects another directory. Checkouts live in a temporary
directory outside Oxc's workspace; individual training repos are removed after
collection to bound disk usage.

Each package uses a separate instrumented release build on a matching native host
and target. Training exercises the published synchronous/asynchronous APIs and
parser raw transfer without executing input code. Transformer inputs train `esnext`
and `es2015`, except inputs containing `accessor`, which omit unsupported downlevel
lowering. Native diagnostics are counted; crashes, corrupt inputs, and empty
profiles fail the build.

Preparation and training reserve a 512 MiB worker stack for deeply nested inputs.
Async calls use small inputs because libuv stacks have a fixed size. Training
processes exit before the pinned Rust toolchain's `llvm-profdata` merges profiles.
Profiles are kept separate by package, target, and release commit.

The optimized binding is checked through the public API before replacing the
normal release artifact. Bindings, profiles, and provenance are stored in
`target/pgo/<package>/<target>/`. Full binding suites run in project CI.

## Release dry runs

Run **Release NAPI** in GitHub Actions, select the branch/tag, and choose a
package. The workflow prepares the corpus once and shares it between native target
jobs. Manual runs build the complete release matrix, assemble the normal
platform packages, and pack them with pnpm without publishing or requiring a
version bump. Download the normal package tarballs from `release-<package>` and
PGO provenance from `pgo-<package>-<target>`. Automatic releases retain their version
check and trusted publishing.
