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

The optimized binding is loaded and checked through the public API before replacing
the package's normal release artifact. Existing project CI owns the full binding
suites; release builds do not rerun them. Staged bindings, raw profiles, merged data,
and provenance are stored in `target/pgo/<package>/<target>/`.

## Release dry runs

Run **Release NAPI** in GitHub Actions, select the branch/tag, and choose a
package. Manual runs build the complete release matrix, assemble the normal
platform packages, and pack them with pnpm without publishing or requiring a
version bump. Download the normal package tarballs from `release-<package>` and
PGO provenance from `pgo-<package>-<target>`. Automatic releases retain their version
check and trusted publishing.
