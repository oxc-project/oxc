# Task 1: ASCII Turbo-Loop Analysis

## Current State Analysis

### Existing Optimizations Already in Place

The Oxc lexer already implements several key optimizations that align with V8/WebKit approaches:

1. **Byte Dispatch Table** (`BYTE_HANDLERS[256]`)
   - O(1) character classification
   - No branching for character type lookup
   - Already implements the core "turbo-loop" concept from V8

2. **Batch Search Infrastructure** (`byte_search!` macro)
   - Processes 32 bytes per iteration in tight loop
   - Used for:
     - String literal scanning
     - Template literal scanning
     - Comment scanning
     - Whitespace runs after newlines
   - Already SIMD-friendly (compiler can auto-vectorize)

3. **Branchless Single-Space Skip**
   - In `read_next_token()` lines 347-351
   - Uses arithmetic instead of branches: `pos.add(usize::from(is_space))`
   - Avoids branch misprediction for common single-space case

4. **Pointer-Based Traversal**
   - Direct unsafe pointer operations via `Source` type
   - Minimal overhead, cache-friendly

5. **Cold Branch Optimization**
   - Unicode handling marked `#[cold]`
   - EOF handling in separate cold function
   - Keeps hot path minimal

### Where Traditional "ASCII Turbo-Loop" Would Apply

The classic ASCII turbo-loop optimization (using SIMD to skip non-interesting bytes) is most valuable when:
1. Large runs of skippable content
2. Content that's predictable and uniform
3. No per-character decision making needed

In a JavaScript lexer, this applies to:
- **Whitespace** - Already optimized via `byte_search!` in `line_break_handler()`
- **Comments** - Already optimized via `byte_search!` in `skip_single_line_comment()`
- **Strings/Templates** - Already optimized via `byte_search!` in string/template handlers

### Opportunity Assessment

After deep analysis, the opportunities for further "ASCII turbo-loop" optimization are:

#### Opportunity 1: Multi-Space Skip in Main Loop
**Current**: Skips 1 space branchless
**Potential**: Skip runs of spaces/tabs before dispatching

**Pros**:
- Could help with heavily indented code
- Simple to implement

**Cons**:
- Most code has single spaces between tokens
- Adding a loop might hurt the common case
- Branch prediction might suffer

**Expected Gain**: 2-5% on whitespace-heavy code, neutral or slight regression on typical code

#### Opportunity 2: Extend Byte Table with Direct Token Kinds
**Current**: Dispatch to handler functions
**Potential**: Encode simple token kinds directly in table

**Pros**:
- Could skip function call for punctuation
- Simpler code path for common tokens

**Cons**:
- Significant refactoring required
- Handler functions do more than just return Kind
- May increase code size and hurt I-cache

**Expected Gain**: 3-8% on punctuation-heavy code

#### Opportunity 3: SIMD Whitespace Detection
**Current**: Byte-by-byte in main loop
**Potential**: Use `core::simd` to check 16-32 bytes at once

**Pros**:
- Maximum theoretical speedup
- Aligns with modern CPU capabilities

**Cons**:
- Requires stable SIMD or nightly Rust
- Complex implementation
- May not help much given dispatch is already fast
- Real bottlenecks are elsewhere (Unicode, allocations)

**Expected Gain**: 10-15% on whitespace, but <5% overall

### Recommendation

Given that:
1. Lexer already has strong optimizations
2. `byte_search!` macro already provides batch processing
3. Real performance bottlenecks are likely in:
   - Allocation (escaped strings/templates)
   - Unicode handling
   - Keyword matching

**I recommend focusing optimization efforts on:**
1. **Task 2: Zero-copy literals** (high impact, clear path)
2. **Task 6: Keyword matching** (high frequency operation)
3. **Task 8: ASI metadata** (reduces parser work)

Then circle back to SIMD optimizations if profiling shows whitespace scanning is still a bottleneck.

### Alternative: Document and Validate Current Performance

Instead of premature optimization, let's:
1. Run comprehensive benchmarks
2. Profile with `perf` to find actual hot spots
3. Compare against V8/WebKit on same workloads
4. Optimize based on data, not assumptions

---

## If We Proceed with Task 1

If we still want to implement an ASCII turbo-loop optimization, the most targeted approach would be:

### Implementation: Multi-Space Skip

Replace single-space skip with a small fixed loop that handles 2-4 spaces efficiently:

```rust
// Current (lines 347-351)
let is_space = byte == b' ';
pos = unsafe { pos.add(usize::from(is_space)) };

// Proposed
// Skip up to 3 spaces/tabs in a tight loop
let mut skip_count = 0;
while skip_count < 3 && unsafe { pos.add(skip_count).read() } <= b' ' {
    skip_count += 1;
}
pos = unsafe { pos.add(skip_count) };
```

**Pros**: Simple, bounded, predictable
**Cons**: Adds a loop and branches
**Expected impact**: ±3% depending on code style

### Measurement Plan

Before implementing:
1. Run `lexer_micro` benchmark on whitespace patterns
2. Note baseline tokens/sec
3. Implement optimization
4. Re-run benchmark
5. Compare with `perf stat` for branch mispredictions

**Success Criteria**: ≥5% improvement on `whitespace/heavy_ws` without regression on other benchmarks

---

## Conclusion

The Oxc lexer is already well-optimized with table-driven dispatch and batch search. The traditional "ASCII turbo-loop" concept is largely already implemented. Further micro-optimizations in this area risk diminishing returns.

**Next steps**:
1. Run benchmarks to establish baseline
2. Profile to identify real bottlenecks
3. Focus on high-impact optimizations (allocations, Unicode, keywords)
4. Return to SIMD whitespace skip only if profiling warrants it
