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

## Formatter resource benchmark

```sh
cd tasks/benchmark/formatter_tools && pnpm install
cargo run -p oxc_benchmark --bin formatter_resources -- --repeats 5
cargo run -p oxc_benchmark --bin formatter_resources -- --synthetic-json-family --repeats 3
```

Prerequisites:

- Install the Node-based comparison tools in `tasks/benchmark/formatter_tools/`.
- Use the versions pinned in `tasks/benchmark/formatter_tools/package.json` for reproducible runs.

This benchmark prints Markdown tables comparing `oxfmt`, `biome`, and `prettier` across OPS/sec,
CPU, memory, disk, file descriptor, and IO metrics.
With `--synthetic-json-family`, it generates fixed-size `json`, `jsonc`, and `json5` corpora for
`1, 10, 25, 100, 200, 500, 1000, 10000, 25000` files and prints one row per dataset/tool pair.

Benchmark outputs are intentionally not checked in. Re-run the commands above to reproduce local
measurements.
