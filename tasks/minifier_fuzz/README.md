# Minifier semantic fuzzer

This task follows the central idea of Terser's [`ufuzz`](https://github.com/terser/terser/blob/v5.50.0/test/ufuzz.js): generate deterministic,
self-contained programs with bounded loops and calls, execute the original and
compressed forms, and compare their observable behavior.

```sh
just fuzz-minifier --seed 0 --iterations 10000
```

The current generator deliberately covers a restricted JavaScript subset. It
uses a shared function-call budget, per-loop brake variables, mutable canary
values, and an ordered side-effect trace. Inputs that throw or time out before
minification are skipped. Completed inputs must produce exactly the same tagged
`console.log` values after compression. Mangling is not enabled yet, so a
failure points at compression or code generation rather than name allocation.

On the first mismatch, the source, compressed source, and structured outcomes
are written under `target/minifier-fuzz/` using the failing seed as the file
name. Re-run that seed with `--seed <N> --iterations 1`.
