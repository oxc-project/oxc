# Oxc SIMD

CPU feature sets, and SIMD vector types which work with them.

- CPU feature sets:
  - `Avx2`, `Neon` and `Generic`.
  - `DynCpu` which holds any of them.
  - `Cpu` trait which they all implement.
  - `BASELINE` const which is the most capable SIMD implementation statically known
    to exist on this CPU.
- SIMD vector types: `Vec128` and `Vec256`.
- Vector data types: `VecData128` and `VecData256`, to read/write SIMD vectors from/to.
- Macros which compile code for each CPU feature set: `cpu_multiversion` and `cpu_version`.
- Macro for testing SIMD code: `test_cpus` (with `testing` Cargo feature).

The docs on the `cpu` and `simd` modules (`src/cpu/mod.rs` and `src/simd/mod.rs`)
explain how they work.

At present, only used by `oxc_lexer`, and still under development.
Can be used in future by other Oxc crates.
