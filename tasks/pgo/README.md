# NAPI profile-guided optimization

Parser, transformer, and minifier releases train their native Linux GNU x64/ARM64,
macOS x64/ARM64, and Windows x64 bindings on the full ecosystem and conformance
corpus. Other targets use their existing release builds.

```sh
rustup component add llvm-tools-preview
just submodules
just pgo all --ecosystem-root /path/to/oxc-ecosystem-ci
```

The ecosystem root must contain both matrices and every configured checkout under
`repos/`. Preparation reads committed Git blobs and does not change those checkouts.
It includes every tracked regular `.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, `.tsx`, `.mts`,
and `.cts` file, including repositories whose ecosystem commands currently skip them.
Conformance inputs use the existing suite loaders, exclusions, and source modes.
TypeScript virtual files and strict variants are exported individually.

Preparation lowers supported TS/JSX to JavaScript for minifier training. Its manifest
reports inputs with diagnostics or no runtime output; those still train parser and
transformer. Training never executes ecosystem source code.
Native diagnostics from conformance inputs, such as duplicate exports, are counted
in training provenance. They do not fail profile collection; crashes still do.
Transformer training uses `esnext` and `es2015`. Inputs containing `accessor` train
only `esnext`, because class-field lowering does not yet support private
auto-accessors. The provenance records how many inputs omit downlevel training.

To prepare CI inputs from HTTPS checkouts, without installing ecosystem projects:

```sh
just pgo prepare --fetch
just pgo parser --corpus target/pgo/corpus
just pgo transform --corpus target/pgo/corpus
just pgo minify --corpus target/pgo/corpus
```

Corpus shards are gzip-compressed JSONL. The manifest records Oxc, ecosystem,
repository, and suite revisions, preparation counts, and shard checksums. A prepared
corpus must match the current Oxc commit. Release workflows prepare it once, then
share it between native target jobs. Clones created with `--fetch` live under
`target/pgo/prepare`; source-only sparse checkouts are processed one at a time.

Each package is built separately with release features and the existing target
flags. Generation and use builds share a target directory, while changed PGO flags
force Cargo to rebuild target code. Explicit `--target` keeps host build scripts
and procedural macros out of the profile. Linux builds retain `--use-napi-cross`.
C/C++ dependencies keep their normal compilation settings with PGO explicitly
disabled, avoiding incompatible profiling runtimes between Rust LLVM and the
system C compiler.
Select-instruction instrumentation is disabled to avoid the LLVM 23 counter
corruption regression.

Training loads the exact instrumented `.node` through the published JS wrapper,
exercises synchronous/asynchronous APIs and parser raw transfer, and waits for each
process to exit before merging profiles with the pinned Rust toolchain's
`llvm-profdata`. Missing inputs, corrupt shards, crashes, and empty profiles fail
the build. Profiles are never reused between packages, targets, or release commits.

Preparation and training use a worker with a 512 MiB stack reservation so deeply
nested generated inputs remain part of the corpus. Stack pages are committed on
demand. Async NAPI calls use small inputs because libuv uses fixed-size stacks.

The optimized binding is packed with pnpm, installed offline into an isolated
consumer, and checked through its native binding and public API before replacing
the package's normal artifact. Parser packages also verify their bundled types.
Existing project CI owns the full binding suites; release builds do not rerun them.
Staged bindings, raw profiles, merged data, provenance, and installable tarballs are
stored in `target/pgo/<package>/<target>/`. There are no benchmarks or performance
gates.

## Download installable packages from CI

Releases use the original corpus. Optional expanded preparation preserves every
original shard and adds pnpm-lockfile-pinned npm package
files (including generated SDK/protobuf code), Vue scripts and client/server render
modules, Svelte RealWorld client/server modules, and development/production,
formatted/minified bundles built from real React and Preact application entries.
The framework and bundler outputs are training inputs; their application code is
never executed. Duplicate added inputs are removed by source and parser options.
The manifest records package versions, source revisions, compiler lockfile hash,
generated source hashes, and category counts. Framework compilation failures fail
preparation. Input, output, and preparation directories must not overlap. Existing
unrelated output directories are preserved.

Prepare these additions locally after preparing the original corpus:

```sh
pnpm node tasks/pgo/expand.mjs --original target/pgo/corpus --output target/pgo/corpus-expanded
just pgo parser --corpus target/pgo/corpus-expanded
```

Added inputs primarily use the default parser AST settings. A subset also trains
`preserveParens: false` and alternate `astType` settings through the JSON API. These
are explicit option variants, not captured production request frequencies. Existing
source/conformance collection and release workflows keep their original behavior.

Run **Release NAPI** in GitHub Actions, select the branch/tag, and choose a
package. Manual runs build the complete release matrix, assemble the normal
platform packages, and pack them with pnpm without publishing or requiring a
version bump. Automatic releases retain their version check and trusted publishing.

Download the `napi-pgo-<package>-<target>` artifact for your machine and extract it.
It contains a `.tgz` npm package and the build's `profile.json`. Install the tarball:

```sh
pnpm add ./oxc-parser-<version>-aarch64-apple-darwin.tgz
```

The package keeps its public name (`oxc-parser`, `oxc-transform`, or `oxc-minify`)
and Node API. Its version is `<base>-pgo.<run_id>.<run_attempt>.g<sha8>`. It includes
the JS wrappers, declarations, and validated native binary; parser also bundles
the matching `@oxc-project/types`. These packages target native Node only, with
matching OS/CPU/libc metadata. Browser routing and registry binding dependencies
are omitted. No install-time build is required.

For a transitive dependency, point your project's override at the downloaded
tarball so nested consumers use the candidate too. With pnpm, add this to the
project's `pnpm-workspace.yaml`, then run `pnpm install`:

```yaml
overrides:
  oxc-parser: file:./downloads/oxc-parser-<version>-aarch64-apple-darwin.tgz
```

To package an existing local PGO build without regenerating its profile:

```sh
pnpm node tasks/pgo/pack.mjs --package parser --target aarch64-apple-darwin
```

The tarball is written under `target/pgo/<package>/<target>/npm/`. Packaging uses
temporary manifests, checks the binary against its PGO provenance, installs the
archive offline into a clean consumer project, and exercises its public API before
retaining the archive. Tracked npm manifests remain unchanged.
