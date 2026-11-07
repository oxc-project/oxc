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

Run the focused string parser workloads with:

```sh
cargo bench -p oxc_benchmark --no-default-features --features compiler --bench parser -- parser_strings
cargo bench -p oxc_benchmark --no-default-features --features compiler --bench parser -- parser_get_js_string
```

String workloads parse 64 statements per input in JavaScript or TypeScript module mode, covering both quote styles, Unicode, escapes, names, and literal types. They measure complete parsing and arena reset with default identifier hashes and regex parsing; throughput counts source bytes.

`parser_get_js_string` queries the same string tokens after lexing and escape decoding. Each timed batch contains 64 queries, or 128 for imports; throughput counts queries. Tokens and returned values use `black_box`. Scanning, allocation and arena reset stay outside the timer, so escaped values and their lookup tables remain populated across iterations. These measurements include loop overhead and repeated access to warm data.
