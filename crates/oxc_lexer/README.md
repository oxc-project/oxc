# Oxc Lexer

A spec compliant standalone lexer for JS/JSX/TS.

## Overview

- Test262 compliant.
- TS/JSX Working
- UTF-8 validation costs 0.15CPB (Cycles per byte), optional parameter.
- SIMD in x86_64 AVX2, Scalar for every other platform currently (in progress).
- Regex vs Division cases should be handled even in pathological cases.
