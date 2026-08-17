# JS plugin worker benchmark

Scores whether running JS plugins on one isolate per Rayon thread actually helps, and whether it
taxes the cases it is not meant to help.

Everything here measures `lint_ms`, the spike timer around `lint_files`. It excludes process boot,
config load and JS plugin import, so it isolates the part that worker isolates change. Wall clock
is measured separately with `hyperfine`.

## What gets compared

Two CLIs, both built the same way (`pnpm run build-napi --release` then `pnpm run build-js` in
`apps/oxlint`). Comparing `--threads=N` against `--threads=1` on a single binary would not answer
the question, because the thread count changes the Rust side too.

- **baseline** — the commit before the worker stack, plus the commit that adds the stderr timers.
  Easiest as a separate `git worktree`, so the branch stays checked out.
- **branch** — the worker stack.

## Corpus

```sh
node make-corpus.mjs <repo-root> /tmp/oxlint-bench/files 3
```

620 distinct real library sources from the repo's pnpm store (20–300 KB, minified and bundled
files filtered out), copied 3x: 1,860 files, 145 MiB.

Two configs run over the same files, both with native rules off and exactly one JS rule:

- `slow.oxlintrc.json` → `plugin-slow.js`. A naming-convention rule that per `Identifier` walks the
  ancestor chain on the raw-transfer AST, reads node fields and slices source text. `PASSES`
  repeats that per-identifier work so the run is JS-bound rather than parse-bound. No sleeps and no
  busy loops — every millisecond is AST access.
- `cheap.oxlintrc.json` → `plugin-cheap.js`. The one-visitor `no-debugger` rule from the e2e
  fixtures. The control for "does routing tax runs where JS does almost nothing?".

A corpus only counts as slow-JS if, measured **on the baseline**:

- parse-help (`lint_ms(N) / lint_ms(1)`) is ≥ 0.80 — threads barely help while JS is serialised, so
  the run is not parse-bound; and
- `lint_ms(N)` is ≥ 10 s — long enough that fixed startup cost cannot dominate.

Raise `PASSES` (in `plugin-slow.js`, or `OXLINT_BENCH_PASSES`) until parse-help clears 0.80, then
grow the copy count until the run clears 10 s. At `PASSES=4` on a 16-core machine, parse-help is
0.835 and `lint_ms(16)` is 13.1 s.

## Running

```sh
node lint-ms.mjs --cli <dist/cli.js> --config slow.oxlintrc.json \
  --files /tmp/oxlint-bench/files --threads 16 --runs 11 --label branch-slow --json
```

Median of 11 after one discarded warmup. Run the cells one at a time — concurrent runs measure
core contention, not the linter.

Wall clock:

```sh
hyperfine --warmup 1 --runs 11 -i \
  -n baseline "node <baseline-cli> -c slow.oxlintrc.json --threads=16 --silent <corpus>" \
  -n branch   "node <branch-cli>   -c slow.oxlintrc.json --threads=16 --silent <corpus>"
```

## Thresholds

Hold `--threads=N` constant except in the single-thread row.

| check          | corpus      | threads | ratio             | must be |
| -------------- | ----------- | ------- | ----------------- | ------- |
| Single-thread  | both        | 1       | branch / baseline | ≤ 1.15  |
| Parallel JS    | slow-JS     | N       | branch / baseline | ≤ 0.70  |
| Cheap tax      | no-debugger | N       | branch / baseline | ≤ 1.15  |
| CLI wall clock | slow-JS     | N       | branch / baseline | ≤ 1.30  |

The single-thread row should also show `worker_boot_ms=0` and no `js_workers=` line, which together
mean `K = 1` and no `Worker` was constructed.

If `(worker_boot_ms + plugin_load_ms) / wall` is ≥ 0.15, the corpus is too small for the wall-clock
number to mean anything: grow it and re-run the parallel-JS and wall-clock checks.
