# Benchmark

See https://codspeed.io/oxc-project/oxc

## Locally

```sh
# On PR branch
cargo bench -p oxc_benchmark --bench parser -- --save-baseline pr
git checkout main
cargo bench -p oxc_benchmark --bench parser -- --save-baseline main
critcmp # via `cargo binstall critcmp`
```

## Comment assignment

The benchmark calls the public assignment pass on parsed ASTs, with a plain node-counting traversal for comparison. Parsing stays outside the timed section; writing attachments into comment storage is included. Both cases preserve the IDs assigned by the parser.

```sh
cargo bench -p oxc_benchmark --bench comment_assignment --no-default-features --features comment_assignment
```

Inputs include the standard parser and semantic fixtures: Radix UI, React, Excalidraw's `App.tsx`, `binder.ts`, and `kitchen-sink.tsx`. Synthetic inputs cover comment-free, sparse, dense, deeply nested, long comment runs, and template substitutions. CI runs this benchmark as a separate CodSpeed component.

## Lexer

`oxc_lexer` is benchmarked head-to-head against the lexer in `oxc_parser`, measuring wallclock time.
This benchmark does not run on CodSpeed, because its CPU simulation does not model the out-of-order
execution and branch prediction which `oxc_lexer` is designed to exploit.

```sh
just bench-lexer      # Scalar fallback
just bench-lexer-simd # SIMD core (x86_64 only - on ARM Mac, it runs under emulation)
```

`oxc_parser` is always built without the SIMD flags, so the 2 lexers are benchmarked in separate builds.
The "change" which criterion reports for `oxc_lexer` is its time relative to `oxc_parser`'s.
The "change" reported for `oxc_parser` (if any) is relative to its previous run.

CI runs `just bench-lexer-simd` when `oxc_lexer` changes, and prints the results in the job log.
