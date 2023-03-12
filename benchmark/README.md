# Benchmarks

## Initialize

```bash
./init.sh
```

## Bench

```bash
./bench.sh
```

## Results

The benchmark uses the `webpack/lib` directory, which contains 537 lintable files.

### CPU

* machdep.cpu.brand_string: Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz
* machdep.cpu.core_count: 6

### Single Run

Using the shell `time` command for cpu utilization

* oxc: 638% cpu
* Rome: 501% cpu
* ESLint: 161% cpu

### Hyperfine

## Intel i7 6-core

```
Benchmark 1: oxc
  Time (mean ± σ):      34.6 ms ±   1.3 ms    [User: 160.1 ms, System: 67.2 ms]
  Range (min … max):    31.8 ms …  40.9 ms    75 runs

  Warning: Ignoring non-zero exit code.

Benchmark 2: rome
  Time (mean ± σ):     147.4 ms ±   3.7 ms    [User: 695.4 ms, System: 72.4 ms]
  Range (min … max):   141.9 ms … 153.8 ms    20 runs

  Warning: Ignoring non-zero exit code.

Benchmark 3: eslint
  Time (mean ± σ):      2.905 s ±  0.185 s    [User: 4.387 s, System: 0.254 s]
  Range (min … max):    2.710 s …  3.287 s    10 runs

  Warning: Ignoring non-zero exit code.

Summary
  'oxc' ran
    4.26 ± 0.20 times faster than 'rome'
   83.94 ± 6.25 times faster than 'eslint'
```
