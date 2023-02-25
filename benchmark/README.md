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

```
Benchmark 1: oxc
  Time (mean ± σ):      30.9 ms ±   1.4 ms    [User: 138.2 ms, System: 54.7 ms]
  Range (min … max):    28.6 ms …  35.7 ms    83 runs

Benchmark 2: rome
  Time (mean ± σ):     145.0 ms ±   2.8 ms    [User: 674.9 ms, System: 69.9 ms]
  Range (min … max):   141.5 ms … 151.8 ms    19 runs

  Warning: Ignoring non-zero exit code.

Benchmark 3: eslint
  Time (mean ± σ):      2.661 s ±  0.057 s    [User: 4.076 s, System: 0.223 s]
  Range (min … max):    2.593 s …  2.790 s    10 runs

  Warning: Ignoring non-zero exit code.

Summary
  'oxc' ran
    4.70 ± 0.23 times faster than 'rome'
   86.20 ± 4.35 times faster than 'eslint'
```
