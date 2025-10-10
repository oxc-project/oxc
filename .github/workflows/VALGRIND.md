# Valgrind Performance Analysis Guide

This document explains how to use and interpret results from the Valgrind performance analysis workflow.

## Overview

The Valgrind workflow (`valgrind.yml`) profiles the **core compilation pipeline** of Oxc using a real-world test file (TypeScript's `checker.ts`, ~50k LOC). It runs weekly and can be triggered manually.

## Test File

**TypeScript checker.ts** - The type checker from microsoft/TypeScript
- ~50,000 lines of complex TypeScript
- Real-world production code
- Contains typical large-scale patterns: classes, interfaces, complex control flow
- Stresses the full compilation pipeline

## Pipeline Stages Analyzed

1. **Parser** (`oxc_parser`) - Lexical + syntactic analysis
   - Token generation
   - AST construction
   - TypeScript-specific parsing

2. **Semantic Analysis** (`oxc_semantic`) - Scope and symbol resolution
   - Scope tree building
   - Symbol table construction
   - Binding resolution
   - Reference tracking

3. **Codegen** (`oxc_codegen`) - Code generation
   - AST traversal
   - String building
   - Output formatting

## Profiling Tools

### Cachegrind - CPU Cache Analysis

**What it measures:**
- L1/L2/LL cache hits and misses
- Branch predictions and mispredictions
- Instruction and data reference counts

**Key metrics:**
- **L1 Cache Miss Rate**: Higher rates indicate poor data locality
- **Branch Mispredictions**: Opportunities for better control flow
- **Instructions Executed**: Total computational cost

**How to interpret:**
```bash
# Download the artifact and extract
cg_annotate parser_cachegrind.out

# Look for:
# 1. Functions with high Ir (instruction reads) - hot paths
# 2. High Dr/Dw (data reads/writes) - memory-intensive operations
# 3. High D1mr/D2mr (L1/L2 cache misses) - poor cache locality
```

**Common issues:**
- High cache misses → Improve data structure layout, reduce pointer chasing
- High branch mispredictions → Simplify conditionals, use likely/unlikely hints
- Excessive instructions → Algorithmic improvements needed

### Massif - Heap Profiling

**What it measures:**
- Heap memory allocation over time
- Peak memory usage
- Allocation call stacks

**Key metrics:**
- **Peak Bytes**: Maximum heap memory used
- **Useful Heap**: Actual data vs. overhead
- **Allocation Sites**: Where memory is allocated

**How to interpret:**
```bash
ms_print parser_massif.out

# Look for:
# 1. Peak memory points - when does allocation spike?
# 2. Major allocation sites - which functions allocate most?
# 3. Allocation patterns - steady growth or spikes?
```

**Common issues:**
- Growing heap → Memory leaks or unbounded collections
- Allocation spikes → Arena/pool allocations working as expected (good!)
- High overhead → Too many small allocations, need batching

### Callgrind - Call Graph Profiling

**What it measures:**
- Function call relationships
- Instruction counts per function
- Call frequencies

**Key metrics:**
- **Self Cost**: Instructions executed in function itself
- **Cumulative Cost**: Instructions including callees
- **Call Count**: How often function is called

**How to interpret:**
```bash
callgrind_annotate linter_callgrind.out

# Look for:
# 1. Hot functions - high self cost
# 2. Hot paths - high cumulative cost
# 3. Call frequency - unexpectedly frequent calls
```

**Common issues:**
- Unexpected hot spots → Profile-guided optimization targets
- Frequent calls to small functions → Inlining candidates
- Deep call stacks → Recursion or abstraction overhead

## Running the Workflow

### Automatic Runs
- Runs every Sunday at midnight UTC
- Results available in workflow artifacts

### Manual Trigger
1. Go to Actions → "Valgrind Performance Analysis"
2. Click "Run workflow" → "Run workflow"
3. Wait for completion (~10-20 minutes)
4. Download artifacts from workflow run

## Interpreting Results

### 1. Download Artifacts
Each component has its own artifact containing:
- `*_cachegrind.out` - Raw cachegrind data
- `*_massif.out` - Raw massif data
- `*_callgrind.out` - Raw callgrind data (linter only)
- `*_analysis.md` - Pre-generated summary

### 2. Review Summary
Start with `*_analysis.md` in each artifact for high-level overview.

### 3. Deep Dive Locally
For detailed source-level analysis:

```bash
# Install Valgrind tools locally
sudo apt-get install valgrind  # Linux
brew install valgrind          # macOS (experimental)

# Annotate with source code
cg_annotate --auto=yes parser_cachegrind.out > detailed.txt

# Visualize with kcachegrind (GUI)
kcachegrind parser_cachegrind.out
```

### 4. Compare Over Time
Download artifacts from multiple runs to track:
- Regression detection (increasing cache misses)
- Optimization validation (reduced allocations)
- Performance trends

## Optimization Priorities

### Critical Path (Highest Impact)

1. **Parser** - Affects every single file
   - Token/AST construction
   - Data structure layout
   - Memory locality during parsing

2. **Semantic Analysis** - Second-most expensive stage
   - Scope tree traversal
   - Symbol table lookups
   - Reference resolution

3. **Codegen** - Final output generation
   - String building efficiency
   - AST traversal patterns
   - Output buffer management

### Cross-Cutting Concerns

4. **Arena Allocator** (`oxc_allocator`)
   - Used by all three stages
   - Bump allocation patterns
   - Memory fragmentation

5. **AST Node Layout**
   - Cache line efficiency
   - Field ordering
   - Pointer chasing reduction

## Example Optimization Workflow

### Case Study: Parser Optimization

1. **Identify bottleneck** from Valgrind results
   ```
   Parser Cachegrind Results:
   - L1 Data Cache Miss Rate: 15.2% (expected: <5%)
   - Hot function: oxc_parser::lexer::Lexer::read_token (12M instructions)
   ```

2. **Locate source** using cg_annotate
   ```bash
   cg_annotate --auto=yes parser_cachegrind.out
   # Shows: Token struct access pattern causes cache misses
   ```

3. **Formulate hypothesis**
   - Token fields accessed sequentially: `kind`, `start`, `end`
   - Current layout has padding between fields
   - **Hypothesis**: Reorder fields for better cache line utilization

4. **Implement fix**
   ```rust
   // Before (poor cache locality)
   pub struct Token {
       pub kind: TokenKind,    // 1 byte + 3 padding
       pub start: u32,         // 4 bytes
       pub flags: TokenFlags,  // 1 byte + 3 padding
       pub end: u32,           // 4 bytes
   }

   // After (better cache locality)
   pub struct Token {
       pub kind: TokenKind,    // 1 byte
       pub flags: TokenFlags,  // 1 byte + 2 padding
       pub start: u32,         // 4 bytes
       pub end: u32,           // 4 bytes
   }
   ```

5. **Validate improvement**
   - Run workflow again on same test file
   - Compare: L1 miss rate reduced from 15.2% → 8.7%
   - Cache improvement: 42% reduction in L1 misses

6. **Measure real-world impact**
   - Check `benchmark.yml` for wall-clock time
   - Expected: 3-5% parsing speedup on large files

## Common Patterns in Oxc

### Expected Patterns (Good)

**Parser:**
- High arena allocation spikes at start (AST construction)
- Sequential token reading with good cache locality
- Predictable branch patterns in keyword recognition

**Semantic:**
- Moderate arena allocation (scope/symbol tables)
- Tree traversal with reasonable cache behavior
- HashTable lookups concentrated in binding resolution

**Codegen:**
- String buffer growth (expected, not a leak)
- Sequential AST traversal
- Low branch misprediction (straightforward code generation)

### Red Flags (Investigate)

**Cache Issues:**
- L1 data cache miss rate >10%
- Excessive pointer chasing (linked list traversal)
- Poor struct field layout (padding issues)

**Memory Issues:**
- Heap growth after arena allocation phase
- Excessive small allocations outside arena
- Allocation hot spots in tight loops

**Execution Issues:**
- High branch misprediction (>5%)
- Hot spots in unexpected functions
- Excessive recursion depth

## Troubleshooting

### Workflow Fails
- Check if test files are available (submodules)
- Verify binary builds successfully
- Review Valgrind stderr for errors

### Results Look Wrong
- Ensure release build (debug builds skew results)
- Check if input files are representative
- Verify Valgrind version (3.18+ recommended)

### No Clear Bottleneck
- Results within expected range
- Focus on micro-benchmarks in benchmark.yml
- Consider algorithmic improvements instead

## Integration with Other Tools

### Complement with:
- **`benchmark.yml`** - Wall-clock time measurements
- **`bloat.yml`** - Binary size analysis
- **`cargo_llvm_lines.yml`** - Generic monomorphization bloat
- **`cargo flamegraph`** - CPU sampling profiler (manual)
- **`cargo criterion`** - Statistical benchmarking (manual)

### Workflow:
1. **Valgrind** → Identify what's slow (cache, allocations, calls)
2. **Flamegraph** → Confirm on real workloads
3. **Criterion** → Measure optimization impact
4. **Benchmark** → Validate in CI

## Resources

- [Valgrind Documentation](https://valgrind.org/docs/manual/manual.html)
- [Cachegrind Manual](https://valgrind.org/docs/manual/cg-manual.html)
- [Massif Manual](https://valgrind.org/docs/manual/ms-manual.html)
- [Callgrind Manual](https://valgrind.org/docs/manual/cl-manual.html)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Contributing

If you find a performance issue via Valgrind:

1. Open an issue with:
   - Component affected
   - Valgrind output excerpts
   - Proposed optimization

2. Submit PR with:
   - Before/after Valgrind metrics
   - Benchmark improvements
   - Explanation of changes

3. Maintainers will validate with full workflow run
