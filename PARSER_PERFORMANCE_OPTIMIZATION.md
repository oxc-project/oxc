# Oxc Parser Performance Optimization

## Goal
Bring Oxc's parser (string → tokens → AST) closer to V8/WebKit-level throughput by systematically improving scanning (lexing) and tokenization performance.

## Current Implementation Analysis

### Architecture Strengths
1. **Byte dispatch table**: Uses `BYTE_HANDLERS[256]` for O(1) character classification
2. **Pointer-based traversal**: Unsafe pointer operations via `Source` type for minimal overhead
3. **ASCII fast paths**: Specialized handling for ASCII in identifier/string scanning
4. **SIMD-ready search**: `byte_search!` macro with `SafeByteMatchTable` for pattern matching
5. **memchr integration**: For multi-line comment end detection

### Key Components

#### Lexer (`crates/oxc_parser/src/lexer/`)
- `mod.rs`: Main lexer with `read_next_token` loop
- `byte_handlers.rs`: Dispatch table mapping bytes to handlers
- `source.rs`: Low-level pointer-based source traversal
- `identifier.rs`: Identifier and keyword scanning
- `numeric.rs`: Number literal parsing
- `string.rs`: String literal handling
- `template.rs`: Template literal processing

#### Current Hot Path (`read_next_token`)
```rust
loop {
    // Single space optimization (branchless)
    let byte = unsafe { pos.read() };
    let is_space = byte == b' ';
    pos = unsafe { pos.add(usize::from(is_space)) };

    // Dispatch to handler
    let kind = unsafe { self.handle_byte(byte) };
    if kind != Kind::Skip { return kind; }
}
```

### Identified Bottlenecks

1. **Whitespace/Comment Scanning**
   - Current: Character-by-character with single-space optimization
   - Opportunity: Vectorized skipping of runs of whitespace

2. **String/Template Literal Allocation**
   - Current: `FxHashMap<u32, &'a str>` for escaped strings
   - Stores escaped strings even when not needed by parser immediately
   - Opportunity: Zero-copy for non-escaped literals

3. **Identifier Scanning**
   - Current: `byte_search!` for ASCII, char-by-char for Unicode
   - Keyword matching via string comparison after scanning
   - Opportunity: Inline keyword recognition, optimized Unicode tables

4. **Numeric Parsing**
   - Current: Partially split by type (binary/octal/hex/decimal)
   - Opportunity: More aggressive specialization and inlining

5. **Token Metadata**
   - Current: Minimal flags (escaped, has_separator, etc.)
   - Opportunity: Add ASI-relevant flags (saw_line_terminator)

## Benchmark Infrastructure

### Created Benchmarks
1. **`lexer_baseline.rs`**: End-to-end lexing with realistic code samples
   - Measures tokens/sec and MB/sec throughput
   - Embedded test cases (no external dependencies)

2. **`lexer_micro.rs`**: Granular operation benchmarks
   - Whitespace skipping patterns
   - Identifier scanning (ASCII, Unicode, keywords)
   - String literals (simple, escaped, Unicode)
   - Numeric literals (int, float, hex, binary, bigint)
   - Template literals
   - Comments
   - Punctuation
   - Mixed realistic patterns

### Running Benchmarks
```bash
# Baseline throughput
cargo bench --bench lexer_baseline

# Micro-benchmarks
cargo bench --bench lexer_micro

# Specific operation
cargo bench --bench lexer_micro -- identifier
```

## Optimization Tasks

### Task 1: ASCII Turbo-Loop (Vectorized Whitespace Skip)
**Goal**: Skip runs of non-interesting ASCII bytes using SIMD/memchr

**Current State**: Single-space branchless optimization
```rust
let is_space = byte == b' ';
pos = unsafe { pos.add(usize::from(is_space)) };
```

**Target**: Use `memchr` or SIMD to find next "interesting" character
- Interesting: `['/', '"', '\'', '\\', '$', '`', '<', newline]`
- Skip everything else in bulk

**Implementation Plan**:
1. Create static table of "interesting" vs "skippable" bytes
2. Use `memchr::memchr_iter` or SIMD scan to find next interesting byte
3. Dispatch only when hitting interesting character
4. Benchmark: Expect ≥1.3× on whitespace-heavy code

**V8 Reference**: `Scanner::Scan()` in `src/parsing/scanner.cc`

---

### Task 2: Zero-Copy Literal Slices + Latin-1 Optimism
**Goal**: Avoid allocating during tokenization; track encoding

**Current State**:
- Escaped strings stored in `FxHashMap<u32, &'a str>`
- Allocated during lexing even if not used

**Target**:
- Store token as `(start, end)` slice reference
- Add `encoding` flag: `OneByte | TwoByte | Escaped`
- Only allocate when escape processing is needed
- Delay string materialization until AST construction

**Implementation Plan**:
1. Add encoding flag to `Token`
2. Modify string/template handlers to set flag, not allocate
3. Move escape processing to on-demand getter
4. Benchmark: Measure allocation count reduction

**V8 Reference**: `ScanIdentifierOrKeyword()` and `LiteralBuffer` in V8

---

### Task 3: Identifier Classification (SWAR + Unicode Tables)
**Goal**: Reduce cost of Unicode ID_Start/ID_Continue checks

**Current State**:
- Fast path for ASCII via `byte_search!`
- Unicode via `is_identifier_part_unicode()` (char-by-char)

**Target**:
- Two-level Unicode table: high byte → block bitset (BMP)
- Range table for astral codepoints
- ASCII handled via bitmask (already done)
- Optionally: SIMD to skip runs of ASCII letters

**Implementation Plan**:
1. Generate compact Unicode tables at build time
2. Replace char-by-char Unicode checks with table lookups
3. Benchmark on mixed ASCII/Unicode identifiers

**V8 Reference**: `unibrow` tables in V8

---

### Task 4: Template Literal Fast Path
**Goal**: Skip clean template segments without allocation

**Current State**:
- Uses `byte_search!` to find `` ` `` or `${`
- Creates escaped template map

**Target**:
- Add `has_escape` and `has_non_ascii` flags
- Only allocate if flags set
- Use `memchr2` to find `` ` `` or `$` in one pass

**Implementation Plan**:
1. Modify `read_template_literal` to track flags
2. Skip allocation for clean templates
3. Benchmark on JSX-heavy code

---

### Task 5: Numeric DFA Split
**Goal**: Specialized fast paths for common numeric forms

**Current State**:
- `read_zero()` dispatches by second character
- `decimal_literal_after_first_digit()` handles decimals

**Target**:
- Separate functions for:
  - Decimal integer (no dot, no exponent)
  - Float (with dot)
  - Scientific notation (with exponent)
  - Radix literals (0x, 0b, 0o)
- Inline and optimize each path independently

**Implementation Plan**:
1. Split numeric handlers into specialized functions
2. Mark with `#[inline]` and add branch hints
3. Benchmark on number-heavy code

**V8 Reference**: `ScanNumber()` in V8

---

### Task 6: Keyword Matching Without Allocations
**Goal**: Match keywords on raw slices

**Current State**:
- `identifier_name_handler()` returns `&str`
- `Kind::match_keyword()` compares string

**Target**:
- Dispatch by identifier length first
- Direct byte comparison without allocating
- Use perfect hash or switch on first char + length

**Implementation Plan**:
1. Modify byte handlers to inline keyword checks
2. Avoid string allocation for keyword paths
3. Benchmark on keyword-dense code

**V8 Reference**: `KeywordMatcher` in V8

---

### Task 7: Memory & Arena Optimization
**Goal**: Reduce per-node allocation overhead

**Current State**:
- Bump allocator for AST nodes
- `(start, end)` stored as `Span` with `u32` offsets (good!)

**Target**:
- Confirm all nodes use arena
- Compact `NodeId` to `u32` indices if not already
- Delay string interning until semantic pass

**Implementation Plan**:
1. Audit allocation sites
2. Measure memory footprint per AST node
3. Profile allocator overhead with flamegraph

---

### Task 8: ASI Metadata in Lexer
**Goal**: Reduce parser lookahead for ASI

**Current State**:
- Parser checks for line terminators when needed for ASI

**Target**:
- Lexer tracks `saw_line_terminator` during whitespace/comment skip
- Include flag in `Token` metadata
- Parser reads flag instead of re-scanning

**Implementation Plan**:
1. Add `after_newline` flag to `Token`
2. Set during `line_break_handler()`
3. Consume in parser for ASI rules
4. Benchmark on newline-sensitive code

---

## Measurement Methodology

### For Each Optimization:
1. **Before**: Run micro-benchmark for affected operation
2. **Implement**: Make targeted change with minimal scope
3. **After**: Re-run micro-benchmark
4. **Document**:
   - Tokens/sec delta
   - Allocation count change (via `massif` or `dhat`)
   - Assembly inspection (via `cargo asm` or `perf annotate`)
   - Flamegraph comparison (if hot path affected)

### Success Criteria:
- ≥1.2× speedup on affected operation
- No correctness regressions (all tests pass)
- Code remains maintainable

---

## Performance Tracking

### Baseline Measurements
*To be filled after running benchmarks*

#### Throughput (MB/s)
- Small JS (< 1KB): TBD
- Medium JS (~5KB): TBD
- Large JS (~20KB): TBD

#### Tokens/Second
- Overall: TBD tokens/sec

#### Operation Timings
- Identifier scan (ASCII): TBD µs
- String literal (no escape): TBD µs
- Numeric literal (integer): TBD µs
- Template literal (simple): TBD µs
- Whitespace skip (10 spaces): TBD µs

### After Each Optimization
*Document here*

---

## References

### V8 (Chromium)
- **Repository**: https://chromium.googlesource.com/v8/v8/
- **Scanner**: `src/parsing/scanner.cc`
- **Parser**: `src/parsing/parser.cc`
- **Preparser**: `src/parsing/preparser.cc`
- **Blog**: https://v8.dev/blog/scanner

### WebKit (JavaScriptCore)
- **Repository**: https://github.com/WebKit/WebKit/
- **Lexer**: `Source/JavaScriptCore/parser/Lexer.cpp`
- **Parser**: `Source/JavaScriptCore/parser/Parser.cpp`

### Tools
- **Profiling**: `perf`, `cargo-flamegraph`, `Instruments` (macOS)
- **Memory**: `valgrind --tool=massif`, `dhat`, `jemalloc` stats
- **Assembly**: `cargo asm`, `perf annotate`
- **Benchmarking**: `criterion.rs`

---

## Notes

- All changes must pass existing conformance tests
- Performance is secondary to correctness
- Maintain code readability where possible
- Document any unsafe code thoroughly
- Use data-driven decisions (measure, don't guess)

---

## Implementation Log

### 2025-11-10: Initial Analysis
- Explored current lexer implementation
- Created benchmark infrastructure (`lexer_baseline.rs`, `lexer_micro.rs`)
- Identified 8 optimization tasks
- Next: Run baselines, then start with Task 1 (ASCII turbo-loop)
