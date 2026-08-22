# `oxc_wtf8`

WTF-8 encoding for oxc — lossless storage of JavaScript strings containing lone surrogates.

## Overview

[WTF-8](https://simonsapin.github.io/wtf-8/) is a superset of UTF-8 that additionally encodes
lone surrogate code points (U+D800..U+DFFF) as 3-byte sequences (`ED A0 80` .. `ED BF BF`).

JavaScript strings are arbitrary sequences of UTF-16 code units and may contain unpaired
surrogates (e.g. `"\uD800"`). Standard UTF-8 cannot represent these, but WTF-8 can — so oxc
uses WTF-8 as its internal string representation to store such strings **without loss**,
while remaining byte-compatible with UTF-8 for all well-formed input.

This crate is ported from SWC's `hstr::wtf8` (MIT), which itself derives from
[`rust-wtf8`](https://github.com/SimonSapin/rust-wtf8) and Rust std's `Utf8Error` handling.

## Relationship to `oxc_str`

This crate provides the raw encoding primitives. The arena-friendly wrapper used across the
oxc AST is [`oxc_str::Wtf8Str`](https://docs.rs/oxc_str/latest/oxc_str/struct.Wtf8Str.html)
(`&'a Wtf8` allocated in an `oxc_allocator::Allocator`), which backs
`StringLiteral::value`, `TemplateElementValue::cooked`, and `JSXText::value`.
