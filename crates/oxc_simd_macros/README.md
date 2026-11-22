# Oxc SIMD Macros

Procedural macros for `oxc_simd`.

- `cpu_multiversion` compiles a function for each supported CPU feature set.
- `cpu_version` defines code for a single CPU feature set.
- `test_cpus` defines a test function that runs for each supported CPU feature set.

These macros are intended for use within `oxc_simd`, and crates which use it.
