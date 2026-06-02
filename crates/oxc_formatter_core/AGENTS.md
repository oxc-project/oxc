# Coding agent guides for `crates/oxc_formatter_core`

## Overview

Language-agnostic formatting infrastructure, ported from [Biome](https://github.com/biomejs/biome)'s `biome_formatter` crate.

Every language-specific formatter in the oxc ecosystem (`oxc_formatter` for JS/TS, `oxc_formatter_json`, and future CSS/GraphQL/etc.) builds on this crate.
It owns the IR and the printing pipeline; it knows nothing about any concrete language (no comments, no quote rules, those live in the consumer crates).

### The IR ("Document") and pipeline

Formatting is two stages:

1. A consumer crate walks its AST and builds an IR, a tree of `FormatElement`s using the `builders` and the `write!` / `format_args!` macros
2. The `Printer` consumes that IR plus `PrinterOptions` and produces the output string, deciding line breaks, indentation, and group expansion

Key IR pieces are all exported from the crate root.

### Generic context design

The core is parameterized over a consumer-supplied context so it stays language-agnostic:

- `FormatContext` trait: no lifetime parameter
  - (avoids `oxc_allocator`'s `'ast` propagating through struct bounds and blocking anonymous lifetimes)
  - The allocator lives on `FormatState`, not the context
- `FormatOptions` trait: `indent_style()`, `indent_width()`, `line_width()`, `line_ending()`, `as_print_options() -> PrinterOptions`
  - Core option types: `IndentStyle`, `IndentWidth`, `LineWidth`, `LineEnding`, `Expand`, `BracketSpacing`
- `Format<'ast, C>` trait + `FormatState<'ast, C>`, `Formatted<'ast, C>`, `Formatter<'buf, 'ast, C>`, `Buffer<'ast, C>`
  - All generic over the context `C`, consumers add a `C` bound only on `impl` blocks
  - Not on struct definitions, and typically define a `type FooFormatter<…> = Formatter<…, FooContext<…>>` alias to keep lifetimes aligned

## Cargo features

`test_harness` exposes `test_support/` (fixture test generation) to downstream crates.
Consumers still need their own `insta` dev-dep so the recorded `source:` header points to the consumer crate.

## Verification

```sh
cargo c -p oxc_formatter_core
cargo c -p oxc_formatter_core --features test_harness
```

Run `clippy` for both configurations and resolve all warnings.

This crate has basic tests only of its own, it is exercised through the conformance/snapshot tests of its consumers.
