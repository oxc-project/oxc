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

The semantics of each building block live in the `builders.rs` rustdocs.
e.g. the three mechanisms for verbatim multi-line content
(`literal_line_break()`, multiline `text()`, `text(..).without_expand_parent()`, and `mark_as_root` / `dedent_to_root`),
with the non-obvious behaviors pinned by printer tests verified against Prettier's `printDocToString`.

Prettier doc primitives are ported on demand; still missing:

- `hardlineWithoutBreakParent` (markdown tables)
- and the `trim` doc

### Generic context design

The core is parameterized over a consumer-supplied context so it stays language-agnostic:

- `FormatContext` trait: no lifetime parameter
  - (avoids `oxc_allocator`'s `'ast` propagating through struct bounds and blocking anonymous lifetimes)
  - The allocator lives on `FormatState`, not the context
- `FormatOptions` trait: `indent_style()`, `indent_width()`, `line_width()`, `line_ending()`, `as_print_options() -> PrinterOptions`
  - Core option types: `IndentStyle`, `IndentWidth`, `LineWidth`, `LineEnding` (exactly the `PrinterOptions` inputs; see the boundary section below)
- `Format<'ast, C>` trait + `FormatState<'ast, C>`, `Formatted<'ast, C>`, `Formatter<'buf, 'ast, C>`, `Buffer<'ast, C>`
  - All generic over the context `C`, consumers add a `C` bound only on `impl` blocks
  - Not on struct definitions, and typically define a `type FooFormatter<…> = Formatter<…, FooContext<…>>` alias to keep lifetimes aligned

### Embedded-language infrastructure (`embedded.rs`)

`EmbeddedContext` / `FormatDispatcher` / `DispatchResult` / `TailwindCollector` let one
formatter's IR be built inside another's document (e.g. graphql-in-js):

- The orchestrator (oxfmt) assembles the dispatcher, mapping language names to
  formatter implementations (or a Prettier fallback); formatter crates only invoke it
- Parent and child share one arena and one `GroupId` space through `EmbeddedContext`
- A language crate's `format_to_ir` entry returns `EmbeddedIr` (IR + pre-sort
  Tailwind classes) — one shape for every child language, no per-crate tuples
- Cross-language contract data is first-class on `DispatchResult` (`tailwind_classes`);
  only truly language-pair specific data crosses as `dyn Any` (e.g. HTML's `has_multiple_root_elements`),
  core never learns concrete languages
- Consumers access `DispatchResult.docs` directly
  (single-doc takes `docs.into_iter().next()`, multi-doc walks `docs`);
  call `DispatchResult::remap_tailwind_into` first when the child may carry classes,
  the printer's `debug_assert` catches a forgotten merge

### What belongs in core (the boundary)

Two layers, two admission rules. A type/fn that fits neither belongs in a consumer crate.

- (1) engine: The IR + Printer + the option types the `Printer` actually consumes
  - `PrinterOptions`: `IndentStyle`, `IndentWidth`, `LineWidth`, `LineEnding`

Admission: the printing phase consumes it. "Shared by all languages" is NOT a reason on its own

- (2) `spec/`: Shared formatter behaviors reused across language formatters

Output targets Prettier compatibility, but the layer is defined by what it is, not by Prettier.

Three gates, all required and note "shared across languages" describes what lives here but is not the admission test.
The gates are:

1. pure functions only (no option/config types),
2. language differences arrive as explicit parameters, never hidden defaults or baked-in language rules,
3. nothing is re-aliased as a language's public config type.

Import discipline (convention): `spec/` only imports `std`, `cow-utils`, etc.

A pure predicate over text shared by design (e.g. `is_suppression_marker`: all formatters honor the same ignore directives) is a desired contract and belongs here.
Unlike option types like `QuoteStyle`, where sharing would encode a coincidental contract that breaks when languages diverge.

Parameterizing language differences (sharpened gate 2), when a shared helper needs to vary per language:

- a value / classifier / data parameter keeps it in core (core asserts nothing)
  - e.g. `normalize_string` takes a raw quote byte, `SourceText` takes byte offsets
- a parameter that would have to encode the language's grammar / logic structure is the language smuggled in disguise → it belongs in the consumer

`SourceText` follows this line. Core owns mechanical, offset-keyed access only (slicing, raw-byte lookups).
Lexical-semantic scanning whose answer is language-defined, what counts as a newline (U+2028/U+2029), a comment, or ASI/parens trivia lives in the consumer (`oxc_formatter`'s `SourceTextExt`), not here.
Even raw newline detection proved to be consumer-owned (every consumer needs the LS/PS-aware variant in addition to `\r|\n`), so core keeps no newline helpers.
Quote-style options, comment rules, and the like are likewise consumer-owned.

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
