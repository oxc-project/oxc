# Whole-project token-transfer experiment

Compare Oxc PR #28892 with its parent, #28893, inside a fixed Rolldown checkout.
Both use the same pinned `oxc_sourcemap` dependency. The only production source
file switched between builds is `oxc_codegen/src/sourcemap_builder.rs`.

- Rolldown: `73a0b45bf8604a4cefaace350f269fed4ff1e677`.
- Inputs: `rolldown/benchmarks@f1e3c1355328a4081c097084f6b2b33ec4e4eb77`.
- Three.js uses only `three10x/copy1`, one real source tree, without duplication.
- Rome uses the complete archived TypeScript project entry and its tsconfig.
- Normal and minified builds write JS and serialized maps through `Bundler.write`.
- Sourcemap-off normal builds serve as controls.
- The executable uses mimalloc, as Rolldown's production native binding does.
- Release optimization and fat LTO are retained; symbols are kept for memory
  instrumentation of the statically linked allocator.
- Each invocation creates a fresh process and resolver; OS file caches are warm.
- Native comparisons use 20 rounds per pass in baseline/candidate/candidate/baseline
  order, on the same CI runner. Startup, I/O, source-map lifetimes and teardown are
  included. Memory mode records the full process high-water marks.
- Exact output filenames, byte lengths and SHA-256 hashes must match, including maps.

This measures Rolldown's Rust build path. It does not include Vite, Node.js startup,
JavaScript plugins, CSS processing or a development server.

All performance measurements run through CodSpeed. The temporary benchmark workflow
builds the two executables, generates exec configs, runs them and verifies output.
