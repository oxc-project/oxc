# Conformance Runner Architecture

## Overview

The conformance runner validates Oxc's toolchain (parser, semantic analyzer, codegen, formatter, transformer, minifier) against industry-standard test suites (Test262, Babel, TypeScript, etc.). This document describes the architecture optimizations introduced to improve performance and reduce resource usage.

## Problem Statement

### Before Optimization

The original implementation had significant inefficiencies:

1. **Redundant Directory Traversals**: Each tool (parser, semantic, codegen, formatter, transformer, minifier, estree) independently walked the test directory tree
2. **Redundant File I/O**: Each tool independently read every test file
3. **Memory Inefficiency**: Test results were kept in memory across all suites simultaneously

### Impact

For a typical test suite like Test262 with ~40,000 files and 7 tools:

- **280,000 directory stat calls** (40,000 files × 7 tools)
- **280,000 file read operations** (40,000 files × 7 tools)
- **High memory footprint** from loading all suites at once

## Architecture Solution

### Key Principles

1. **Discover Once, Use Many**: Perform file discovery once per suite, share results across tools
2. **Sequential Processing**: Process suites sequentially to minimize peak memory usage
3. **Centralized Discovery**: Extract file discovery logic into dedicated module

### Components

#### 1. File Discovery Layer (`file_discovery.rs`)

**Purpose**: Centralize file system operations

**Components**:

- `DiscoveredFiles`: Container for all files from a test suite
- `DiscoveredFile`: Single file with path and pre-loaded content
- `FileDiscoveryConfig`: Configuration for discovery process

**Key Features**:

- Single directory walk per suite
- Single file read per file per suite
- Handles UTF-8 and UTF-16LE encoding
- Automatic submodule initialization

#### 2. Suite Trait Enhancement (`suite.rs`)

**New Method**: `run_with_discovered_files()`

Allows suites to accept pre-discovered files instead of discovering them independently.

**Backward Compatibility**: Original `run()` method retained for legacy use cases.

#### 3. Sequential Suite Processing (`lib.rs`)

**Pattern**:

```rust
fn process_test262_suite(&self) {
    // 1. Discover files once
    let files = DiscoveredFiles::discover(&config);

    // 2. Run all tools with same files
    ParserSuite::new().run_with_discovered_files("parser", &args, &files);
    SemanticSuite::new().run_with_discovered_files("semantic", &args, &files);
    CodegenSuite::new().run_with_discovered_files("codegen", &args, &files);
    // ... more tools

    // 3. Drop files, reclaim memory
}
```

**Benefits**:

- Files loaded once, used 7 times
- Memory freed between suites
- Predictable resource usage

## Performance Improvements

### I/O Reduction

| Operation       | Before      | After       | Improvement      |
| --------------- | ----------- | ----------- | ---------------- |
| Directory Walks | 7 per suite | 1 per suite | **7x reduction** |
| File Reads      | 7 per file  | 1 per file  | **7x reduction** |

For Test262 (~40,000 files):

- **240,000 fewer stat calls** (6 × 40,000)
- **240,000 fewer file reads** (6 × 40,000)

### Memory Optimization

**Before**: All suites loaded simultaneously

```
Peak Memory = Suite1 + Suite2 + Suite3 + Suite4
```

**After**: Sequential processing with explicit drops

```
Peak Memory = max(Suite1, Suite2, Suite3, Suite4)
```

For typical suite sizes:

- Test262: ~40,000 files
- Babel: ~10,000 files
- TypeScript: ~8,000 files
- Misc: ~100 files

Peak memory reduced by processing largest suite in isolation.

## Data Flow

### Old Flow

```
run_parser()
  └─> Test262Suite::run()
      ├─> Walk directory (1st time)
      ├─> Read files (1st time)
      └─> Process with parser

run_semantic()
  └─> Test262Suite::run()
      ├─> Walk directory (2nd time)
      ├─> Read files (2nd time)
      └─> Process with semantic

... (repeated for all 7 tools)
```

### New Flow

```
process_test262_suite()
  ├─> DiscoveredFiles::discover()
  │   ├─> Walk directory (once)
  │   └─> Read files (once)
  │
  ├─> ParserSuite::run_with_discovered_files(&files)
  ├─> SemanticSuite::run_with_discovered_files(&files)
  ├─> CodegenSuite::run_with_discovered_files(&files)
  ├─> FormatterSuite::run_with_discovered_files(&files)
  ├─> TransformerSuite::run_with_discovered_files(&files)
  ├─> MinifierSuite::run_with_discovered_files(&files)
  └─> EstreeSuite::run_with_discovered_files(&files)
```

## Implementation Details

### File Discovery

**Encoding Handling**:

- UTF-8: Direct `fs::read_to_string()`
- UTF-16LE: Fallback with `encoding_rs`
- BOM removal: Strip `\u{feff}` if present

**Filtering**:

- Path-based: `skip_test_path` closure
- Substring: `filter` option
- Extension: `.DS_Store` automatically excluded

### Memory Management

**Lifecycle**:

1. `DiscoveredFiles::discover()` - Allocate and load
2. Multiple `run_with_discovered_files()` calls - Share
3. End of `process_*_suite()` - Drop and reclaim

**Clone Semantics**:

- `DiscoveredFiles` and `DiscoveredFile` are `Clone`
- Allows passing by reference to avoid moves
- Suites can own temporary clones if needed

## Future Optimizations

### Potential Improvements

1. **Parallel File Reading**
   - Current: Sequential with `map()`
   - Future: Parallel with `par_iter()` from rayon
   - Trade-off: Memory for speed

2. **Lazy File Loading**
   - Current: All files loaded upfront
   - Future: Load on first access
   - Trade-off: Complexity for memory

3. **File Content Sharing**
   - Current: Cloned per tool
   - Future: `Arc<str>` for zero-copy sharing
   - Trade-off: Complexity for memory

4. **Suite-Level Parallelism**
   - Current: Sequential suite processing
   - Future: Parallel with bounded memory
   - Trade-off: Coordination for speed

### Measurement Needs

Before implementing future optimizations:

- Baseline performance metrics (timing + memory)
- Profiling to identify bottlenecks
- Benchmarks to validate improvements

## Testing Strategy

### Verification

1. **Functional Correctness**:
   - All existing tests must pass
   - No changes to test results
   - Snapshot files remain unchanged

2. **Performance Validation**:
   - Measure I/O operations (strace/dtruss)
   - Track memory usage (time -v / /usr/bin/time -l)
   - Compare before/after timings

3. **Compatibility**:
   - Legacy `run()` method still works
   - Filter functionality preserved
   - Submodule init still triggers

## Migration Guide

### For New Tools

Use the optimized path:

```rust
let files = DiscoveredFiles::discover(&FileDiscoveryConfig {
    test_root: suite.get_test_root(),
    filter: args.filter.as_deref(),
    skip_test_path: Box::new(|path| suite.skip_test_path(path)),
    skip_test_crawl: suite.skip_test_crawl(),
    suite_name: "my_tool",
});

suite.run_with_discovered_files("my_tool", &args, &files);
```

### For Existing Code

Two options:

1. **Keep using `run()`**: Works unchanged, but not optimized
2. **Migrate to `run_with_discovered_files()`**: Requires passing discovered files

## Related Files

- `tasks/coverage/src/file_discovery.rs` - File discovery implementation
- `tasks/coverage/src/suite.rs` - Suite trait with new method
- `tasks/coverage/src/lib.rs` - Main orchestration logic
- `tasks/coverage/ARCHITECTURE.md` - This document

## References

- [Test262](https://github.com/tc39/test262) - ECMAScript conformance suite
- [Babel](https://github.com/babel/babel) - JavaScript compiler test suite
- [TypeScript](https://github.com/microsoft/TypeScript) - TypeScript compiler test suite
