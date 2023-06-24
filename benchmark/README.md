# Benchmarks

This benchmark does not compare against Rome because the rules do not match.

## Initialize

```bash
./init.sh
```

## Bench

```bash
./bench.sh
```

## Results

The benchmark uses the `vscode/src` directory, which contains 3628 lintable files.

### CPU

* machdep.cpu.brand_string: Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz
* machdep.cpu.core_count: 6

### Single Run

Using the shell `time` command for cpu utilization

* oxc: 638% cpu
* ESLint: 161% cpu

### Hyperfine

## Intel i7 6-cores

```
Benchmark 1: oxc
  Time (mean ± σ):      36.0 ms ±   3.2 ms    [User: 166.0 ms, System: 69.2 ms]
  Range (min … max):    32.4 ms …  51.8 ms    63 runs

Benchmark 2: eslint
  Time (mean ± σ):      3.044 s ±  0.187 s    [User: 4.606 s, System: 0.260 s]
  Range (min … max):    2.824 s …  3.387 s    10 runs

Summary
  'oxc' ran
   84.66 ± 9.18 times faster than 'eslint'
```

## M2 8-cores

```
Benchmark 1: oxc
  Time (mean ± σ):     297.0 ms ±  31.7 ms    [User: 1772.3 ms, System: 205.7 ms]
  Range (min … max):   269.8 ms … 379.7 ms    10 runs

  Warning: Ignoring non-zero exit code.

Benchmark 2: eslint
  Time (mean ± σ):     22.722 s ±  0.470 s    [User: 39.437 s, System: 1.705 s]
  Range (min … max):   22.177 s … 23.805 s    10 runs

  Warning: Ignoring non-zero exit code.

Summary
  'oxc' ran
   76.50 ± 8.32 times faster than 'eslint'
```
