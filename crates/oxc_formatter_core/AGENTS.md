# Coding agent guides for `crates/oxc_formatter_core`

## Overview

Language-agnostic formatting infrastructure.

Every language-specific formatter in the oxc ecosystem builds on this crate.
It owns the IR and the printing pipeline; it knows nothing about any concrete language (no comment placement rules, no quote rules, those live in the consumer crates).

### The IR ("Document") and pipeline

Formatting is two stages:

1. A consumer crate walks its AST and builds an IR, a tree of `FormatElement`s using the `builders` and the `write!` / `format_args!` macros
2. The `Printer` consumes that IR plus `PrinterOptions` and produces the output string, deciding line breaks, indentation, and group expansion

Key IR pieces are all exported from the crate root.

The semantics of each building block live in the `write/builders.rs` rustdocs.
e.g. the mechanisms for verbatim multi-line content (`exact_line_breaks()` for blank runs exempt from newline collapsing, `literal_line_break()`, multiline `text()`, `text(..).without_expand_parent()`, and `mark_as_root` / `dedent_to_root`), with the non-obvious behaviors pinned by printer tests verified against Prettier's `printDocToString`.

Prettier doc primitives are ported on demand; still missing: the `trim`.

Composite primitives translate rather than port 1:1:

- `hardlineWithoutBreakParent` is `hard_line_break().without_expand_parent()`
- `conditionalGroup` is `best_fitting!`: same expansion boundary, same flat-first variant trial.
  Prettier's own doesn't switch variants on inner breaks either ("the user is expected to manually handle what breaks");
  that manual wiring is `ifBreak({groupId})` there and `if_group_breaks(..).with_group_id(..)` here
  See `oxc_formatter_yaml`'s `mapping_item.rs` for the full pattern.
  Oxfmt's Doc→IR mechanical conversion maps `expandedStates` to the same `BestFitting` primitive.

### The printer never trims

Unlike Prettier's `printDocToString`, this printer emits exactly what was written:
end-of-line whitespace never appearing in the output is guaranteed by construction (pending space/indention, no indention on blank lines), not by a trimming pass.
Text/Token content is the emitter's responsibility, language crates write their values pre-trimmed.

NOTE: The printer's runtime optimizations (pending-space dedup, consecutive-hardline merging, this no-trim rule, ...) are mirrored downstream by `apps/oxfmt`'s `prettier_compat` (IR ↔ Prettier Doc interop; kept there because core never learns Prettier-as-a-system).
When changing printer runtime behavior, update that mirror in the same PR; oxfmt's embedded/E2E conformance is the backstop that catches drift.

### Choosing a staging buffer

The arena is a bump allocator and never reclaims, so a vector grown in it strands every grown-out-of allocation for the rest of the format run.
Pick by what you're building:

- Root document (feeds `Document::new` / `EmbeddedIr`): `VecBuffer` (arena)
  - it moves into the `Document` for free, and heap-staging it costs an extra copy for no benefit
- Unknown-length staging that ends interned/sliced: `HeapVecBuffer`
  - a watermarked view over one scratch vector owned by the format run
  - the arena receives one exactly-sized copy (see its rustdoc for the full rationale)
  - for `BestFitting` variants, `best_fitting_variant` already wraps this (entry tags + staging in one place)
- Accumulating across interleaved `write()` calls (multiple builders open at once), or staging that must release the state between writes and consumption: `ScratchBuffer` (one per accumulator)
  - write through `ScratchBuffer::writer`, finish via `Formatter::intern_elements` (or re-emit via `ScratchBuffer::drain`, abandon via `ScratchBuffer::discard`)
  - the shared scratch's LIFO rule and its exclusive state borrow rule out `HeapVecBuffer` there (see the JSX child-list builders and `AssignmentLike` in `oxc_formatter`)
- Known-length sequences: build exact-sized directly (e.g. `ArenaVec::from_iter_in`)

`Formatter::intern` and `BestFitting` already stage on the heap; consumer crates get this for free.

### Generic context design

The core is parameterized over a consumer-supplied context so it stays language-agnostic:

- `FormatContext` trait: no lifetime parameter
  - (avoids `oxc_allocator`'s `'ast` propagating through struct bounds and blocking anonymous lifetimes)
  - The allocator lives on `FormatState`, not the context
- `FormatOptions` trait:
  - `as_print_options() -> PrinterOptions` is provided from the getters
  - Core option types: `IndentStyle`, `IndentWidth`, `LineWidth`, `LineEnding` (exactly the `PrinterOptions` inputs; see the boundary section below)
  - `CoreFormatOptions`: the four bundled, for handing them across a host boundary (config resolver → language options) in one piece;
    - `apply_core` is the write half of the trait's read-only getters
- `Format<'ast, C>` trait + `FormatState<'ast, C>`, `Formatted<'ast, C>`, `Formatter<'buf, 'ast, C>`, `Buffer<'ast, C>`
  - All generic over the context `C`, consumers add a `C` bound only on `impl` blocks
  - Not on struct definitions, and typically define a `type FooFormatter<…> = Formatter<…, FooContext<…>>` alias to keep lifetimes aligned

### Embedded-language infrastructure (`session/`)

`FormatSession` / `FormatDispatcher` / `DispatchRequest` / `DispatchPayload` / `TailwindCollector` let one formatter's IR be built inside another's document (e.g. graphql-in-js):

- The orchestrator (oxfmt) assembles the dispatcher, mapping language names to formatter implementations (or a Prettier fallback)
  - Formatter crates only invoke it via `FormatSession::dispatch`
- Parent and child share one arena and one `GroupId` space through the session
- A language crate's `format_to_ir` entry returns `EmbeddedIr` (IR + pre-sort Tailwind classes), one shape for every child language, no per-crate tuples
- Cross-language contract data is first-class on `DispatchPayload` (`tailwind_classes`)
  - Only truly language-pair specific data crosses as `dyn Any` (e.g. HTML's `has_multiple_root_elements`), core never learns concrete languages
- Consumers take the doc via `DispatchPayload::into_doc(collector)`, which folds the Tailwind class merge into consumption
  - The printer's `debug_assert` backstops any hand-rolled consumption that skips the merge
- `FormatSession` (`session/mod.rs`) is the execution unit:
  - One arena, one shared `GroupId` space (`Arc<UniqueGroupIdBuilder>`), the host's `SessionServices`, and the input's envelope semantics (`InputKind`), usable by standalone roots and dispatched children alike
  - `SessionServices` names the three per-run duties, one field each: `dispatcher` (IR channel), `string_embedder` (string-out channel, `(language, code, print_width)`; temporary, see domain (4)'s exit criterion), `tailwind_sorter` (print-time batch sort). Core only transports them
  - `FormatSession::dispatch_to_string(request, printer_options)` is the string-out counterpart of `dispatch` (caller supplies the printer options; `Ok(None)` = deliberate keep; see its rustdoc)
  - `FormatState` holds one (`new_with_session`; plain `new` wraps a service-less `PhysicalFile` session), and `Formatter::session()` exposes it during a write
- A dispatch states its request as `DispatchRequest` (language, text, `InputKind`, pair-specific context) and yields `Result<DispatchResponse, String>`:
  - `DispatchResponse::PreserveOriginal` is the DELIBERATE "keep the source as-is" answer (unsupported language, child parse failure, no dispatcher installed); `Err` is reserved for operational failures (transport / internal, recursion limit)
  - Optional-embed callers degrade the same way for both, but never conflate them at the source
  - `FormatSession::dispatch` owns the no-dispatcher case and the recursion limit (`MAX_DISPATCH_DEPTH`), and runs the callback on a `derive_child`ed session

## What belongs in core (the boundary)

Five admission domains, five rules; the numbers are reference labels, not a dependency stack (e.g. the engine's `FormatState` holds a domain-(4) `FormatSession`).
A type/fn that fits none of them belongs in a consumer crate.

- (1) engine: The IR + Printer + the option types the `Printer` actually consumes
  - `PrinterOptions`: `IndentStyle`, `IndentWidth`, `LineWidth`, `LineEnding`
  - Split by pipeline stage: `write/` (IR construction), `format_element/` (the IR + the `Formatted` artifact), `printer/` (printing; knows only `PrinterOptions`)
  - `context/` sits beside them as the consumer contract (`FormatContext` / `FormatOptions` + the core option types); its `as_print_options()` produces the printer's inputs

Admission: the printing phase consumes it. "Shared by all languages" is NOT a reason on its own.

- (2) `source/`: Source-side access mechanics (`SourceText`, `SpanCursor`)

Admission: the structure makes no output decision by itself; language differences arrive as data (offsets, the item type), never as parameters encoding grammar or policy.

- (3) `spec/`: Shared formatter behaviors reused across language formatters

Output targets Prettier compatibility, but the domain is defined by what it is, not by Prettier.

- (4) `session/`: per-run host services, transported opaquely (`SessionServices`: dispatcher / string embedder / Tailwind sorter)

Admission: core stores the service and hands it back (or applies it mechanically at finalize);
every value crossing the closure boundary is an opaque string/vec, never a language enum or an option type, and core makes no decision from the result.
`string_embedder` additionally carries an exit criterion: it is removed when (a) md/html/angular gain IR-capable formatters AND (b) the host's string-out consumers (JSDoc fences) can express their re-embedding in IR (see `apps/oxfmt`'s AGENTS.md for that half); until then it is the string-out channel's transport.

- (5) `envelope/`: IR-composing behavior shared by document-envelope hosts (`write_front_matter`)

Admission: the host opts in by CALLING, and every language difference arrives as a data parameter (`embeddable_languages`); core asserts nothing about what the names mean. Unlike `spec/`, this domain writes IR and drives the dispatcher.

Three gates, all required and note "shared across languages" describes what lives here but is not the admission test.
The gates are:

1. pure functions only (no option/config types),
2. language differences arrive as explicit parameters, never hidden defaults or baked-in language rules,
3. nothing is re-aliased as a language's public config type.

Import discipline (convention): `spec/` only imports `std`, `cow-utils`, etc.

A pure predicate over text shared by design (e.g. `is_suppression_marker`: all formatters honor the same ignore directives) is a desired contract and belongs here.
Unlike option types like `QuoteStyle`, where sharing would encode a coincidental contract that breaks when languages diverge.

`spec/front_matter.rs` shows the same line from the envelope side:
core owns pure detection (`parse_front_matter`, a Prettier `front-matter/parse.js` port) and byte-preserving blanking; the IR half is domain (5).
What the header language MEANS stays host policy: a host opts in by calling and passes its embeddable set as data.
Astro's leading `---` is a JS component script, not YAML, so its host simply never calls — a central "every `---` is YAML" rule cannot exist in core.

Parameterizing language differences (sharpened gate 2), when a shared helper needs to vary per language:

- a value / classifier / data parameter keeps it in core (core asserts nothing)
  - e.g. `normalize_string` takes a raw quote byte, `SourceText` takes byte offsets
- a parameter that would have to encode the language's grammar / logic structure is the language smuggled in disguise → it belongs in the consumer

`SourceText` follows this line. Core owns mechanical, offset-keyed access only (slicing, raw-byte lookups).
Lexical-semantic scanning whose answer is language-defined, what counts as a newline (U+2028/U+2029), a comment, or ASI/parens trivia lives in the consumer (`oxc_formatter`'s `SourceTextExt`), not here.

Newline-adjacent helpers split along the same line:

- `spec/gap.rs` is the shared gap classifier for the CR, LF and CRLF terminator family
  - It takes a raw `&[u8]` slice, never offsets (`SourceText` addresses), `spec/` interprets, consumers compose the two at the call site
- Precedent for gate 2: json/js measure gaps under ECMAScript lexis (LS/PS terminators, blanks before a separator comma ignored)
  - So they keep their own helpers instead of a parameter here

`SpanCursor<T: GetSpan>` follows the same split from the position side:
core owns the span-ordered cursor mechanics; what the items mean (comments) and where they are placed stay consumer-owned.

Quote-style options, comment rules, and the like are likewise consumer-owned.

## Verification

```sh
cargo c -p oxc_formatter_core
```

This crate has basic tests only of its own, it is exercised through the conformance/snapshot tests of its consumers.
The fixture-test infrastructure those consumers use lives in `crates/oxc_formatter_tests`.
