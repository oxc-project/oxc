# Benchmark

## Single run using criterion

```bash
cargo benchmark
```

## Comparing between branches

Install critcmp `cargo install critcmp`

```bash
# on pr branch
cargo benchmark --save-baseline pr

# on main branch
cargo benchmark --save-baseline main

critcmp
```

## bench file sizes
| -------------  | --   |
| pdf.js         | 412K |
| lodash.js      | 526K |
| d3.js          | 559K |
| typescript.js  | 9.6M |
| babylon.max.js |  10M |
