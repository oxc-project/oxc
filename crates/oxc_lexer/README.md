# Oxc Lexer

A spec compliant standalone lexer for JS/JSX/TS on x86_64; Other platforms coming soon.

## Overview

- Test262 compliant.
- TS/JSX Working
- UTF-8 validation costs 0.15CPB (Cycles per byte), optional parameter.

## Building

Requires static AVX2/BMI2 (x86_64). Without the flags the crate compiles to an
empty stub so workspace-wide builds still pass on any platform:

```sh
RUSTFLAGS="-C target-feature=+avx2,+bmi2" cargo test -p oxc_lexer --all-features
```
