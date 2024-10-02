# fastcmp
[![Build Status](https://travis-ci.org/saschagrunert/fastcmp.svg)](https://travis-ci.org/saschagrunert/fastcmp) [![Build status](https://ci.appveyor.com/api/projects/status/hdv8d12xgjbvsiju?svg=true)](https://ci.appveyor.com/project/saschagrunert/fastcmp) [![Coverage Status](https://coveralls.io/repos/github/saschagrunert/fastcmp/badge.svg?branch=master)](https://coveralls.io/github/saschagrunert/fastcmp?branch=master) [![master doc fastcmp](https://img.shields.io/badge/master_doc-peel_ip-blue.svg)](https://saschagrunert.github.io/fastcmp) [![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/saschagrunert/fastcmp/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/fastcmp.svg)](https://crates.io/crates/fastcmp) [![doc.rs](https://docs.rs/fastcmp/badge.svg)](https://docs.rs/fastcmp)
## A fast byte slice comparison library
The library is intended to provide a faster byte slice comparison than the standard library. Also raw string literals
`b"like this"` are compareable this way. It also supports simd comparisons by enabling the feature `simd_support` in the
`Cargo.toml`.

## oxc Note

This is a patched version of the original `fastcmp` crate, developed by the contributors of oxc. The original crate can be found [here](https://github.com/saschagrunert/fastcmp).

## Example usage

```rust
use fastcmp::Compare;

let vec = vec![1, 2, 3, 4, 5];
assert!(vec.feq(&[1, 2, 3, 4, 5]));
```

## Benchmarks
The benchmarking results for comparison of two `&[u8]` with a size of 256:

```
test fast_compare_equal    ... bench:          14 ns/iter (+/- 9) = 18285 MB/s
test fast_compare_unequal  ... bench:          14 ns/iter (+/- 0) = 18285 MB/s
test slice_compare_equal   ... bench:          35 ns/iter (+/- 29) = 7314 MB/s
test slice_compare_unequal ... bench:          37 ns/iter (+/- 3) = 6918 MB/s
```

## Contributing
You want to contribute to this project? Wow, thanks! So please just fork it and send me a pull request.
