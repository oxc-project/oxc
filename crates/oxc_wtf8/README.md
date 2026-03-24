# oxc_wtf8

WTF-8 string types for the [Oxc](https://oxc.rs) JavaScript/TypeScript toolchain.

## Overview

WTF-8 is a superset of UTF-8 that can represent lone Unicode surrogate code units
(U+D800–U+DFFF) which appear in JavaScript strings. This crate provides:

- [`Wtf8`] — a borrowed WTF-8 string slice (analogous to `str`)
- [`Wtf8Buf`] — an owned WTF-8 string buffer (analogous to `String`)
- [`Wtf8Atom<'a>`] — an arena-allocated WTF-8 atom (analogous to `Atom<'a>`)

## Features

- `serialize` — enables `serde::Serialize` and `oxc_estree::ESTree` implementations
  that correctly escape lone surrogates as `\uXXXX` in JSON output.
