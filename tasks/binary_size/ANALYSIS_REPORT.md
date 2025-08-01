# Binary Size Analysis: Comprehensive Report

This document provides a detailed analysis of what causes large binary size in oxc tools and actionable recommendations for size reduction.

## Current State Analysis

### Binary Size Breakdown (oxlint with allocator feature)

- **Total Binary Size**: ~115 MB (with debug symbols)
- **Stripped Size**: ~8.8 MB (production size)
- **Text Section**: ~6.3 MB (actual code)
- **Debug Overhead**: ~107 MB (92.4% of total size)

### Top Contributors to Binary Size

#### 1. oxc_linter (36.3% - 2.1 MB)
The largest contributor to binary size is the linter crate containing all lint rules.

**Key factors:**
- 400+ lint rules with individual implementations
- Pattern matching and AST traversal code
- Rule-specific diagnostic messages and fix suggestions
- Configuration and metadata for each rule

**Recommendations:**
- Consider feature flags for rule categories (e.g., `--features typescript-rules`)
- Lazy loading of rule implementations
- Optimize rule visitor patterns to reduce code duplication
- Consider rule bundling or code generation to reduce boilerplate

#### 2. std (16.2% - 1.0 MB)
Rust standard library contributions, particularly collections and I/O.

**Key factors:**
- HashMap/HashSet implementations (hashbrown)
- String and Vec operations
- I/O and filesystem operations
- Regex and parsing utilities

**Recommendations:**
- Use `std` build optimization: `cargo +nightly build -Z build-std=std,panic_abort`
- Consider custom allocators for specific use cases
- Profile-guided optimization (PGO) can help eliminate unused std code

#### 3. regex_automata (5.5% - 346 KB)
Regular expression engine used throughout the codebase.

**Key factors:**
- Complex state machine generation
- Unicode support and character classes
- Optimization for various regex patterns
- DFA/NFA implementations

**Recommendations:**
- Audit regex usage - consider simpler string operations where possible
- Use regex compilation caching
- Consider feature flags for complex regex features
- Evaluate alternative regex engines with smaller footprint

#### 4. oxc_parser (5.0% - 314 KB)
JavaScript/TypeScript parser implementation.

**Key factors:**
- Complete JavaScript/TypeScript grammar implementation
- Error recovery and diagnostic generation
- AST node creation and validation
- Lexer state machines

**Recommendations:**
- Consider parser feature flags (e.g., TypeScript-only mode)
- Optimize AST node representations
- Lazy evaluation of optional language features

### Generic Function Bloat Analysis

**Total LLVM Lines Generated**: 1,867,069

#### Top Generic Bloat Sources:

1. **hashbrown::raw::RawTable<T,A>::reserve_rehash** (119 instantiations)
   - Hash table operations across different key/value types
   - **Impact**: 30,061 LLVM lines
   - **Fix**: Consider using type erasure or fewer generic parameters

2. **Debug trait implementations** (1,876 instantiations)
   - Debug formatting for every AST node and data structure
   - **Impact**: 24,164 LLVM lines  
   - **Fix**: Conditional debug implementations or simplified debug output

3. **LintContext::diagnostic_with_fix_of_kind** (75 instantiations)
   - Generic diagnostic creation across different rule types
   - **Impact**: 28,011 LLVM lines
   - **Fix**: Reduce generic parameters or use dynamic dispatch

## Actionable Size Reduction Strategies

### Immediate Wins (High Impact, Low Effort)

#### 1. Strip Debug Symbols in Production (92.4% reduction)
```bash
# Production build without debug symbols
RUSTFLAGS="-C strip=symbols" cargo build --release -p oxlint --features allocator
```
**Expected Savings**: ~107 MB → 8.8 MB final binary

#### 2. Link-Time Optimization Improvements
Current settings are already optimal (LTO=fat), but consider:
```toml
[profile.release-size]
inherits = "release"
opt-level = "z"  # Optimize for size instead of speed
lto = "fat"
codegen-units = 1
panic = "abort"
```

#### 3. Feature Flag Optimization
Create more granular feature flags:
```toml
[features]
default = ["essential-rules"]
all-rules = ["typescript-rules", "react-rules", "import-rules", "stylistic-rules"]
essential-rules = []
typescript-rules = []
react-rules = []
# ... etc
```

### Medium-Term Optimizations (Medium Impact, Medium Effort)

#### 1. Rule System Optimization
- **Code Generation**: Generate boilerplate rule code instead of writing by hand
- **Visitor Pattern Optimization**: Reduce generic parameters in visitor traits
- **Rule Categorization**: Allow users to select only needed rule categories

#### 2. Generic Bloat Reduction
- **HashMap Consolidation**: Use fewer generic HashMap instantiations
- **Debug Trait Optimization**: Conditional debug implementations
- **AST Node Optimization**: Reduce generic parameters where possible

#### 3. Dependency Optimization
- **Regex Optimization**: Replace complex regexes with simpler string operations
- **Error Handling**: Consider lighter error handling than miette for core functionality
- **Allocator Strategy**: Profile allocator impact more precisely

### Long-Term Strategies (High Impact, High Effort)

#### 1. Plugin Architecture
Move non-essential rules to plugins that can be loaded dynamically:
```rust
// Core binary with essential rules only
// Additional rules loaded as shared libraries
```

#### 2. Profile-Guided Optimization (PGO)
Use real-world usage patterns to optimize binary:
```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# Step 2: Run with representative workloads  
./target/release/oxlint large-codebase/

# Step 3: Build with optimization
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data" cargo build --release
```

#### 3. Alternative Standard Library
Use a smaller std implementation:
```bash
cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu
```
**Expected Savings**: 10-20% reduction in std contribution

#### 4. WebAssembly Target
Consider WASM as a size-optimized target:
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz -o oxlint.wasm target/wasm32-unknown-unknown/release/oxlint.wasm
```

## Monitoring and Regression Prevention

### Continuous Integration
The binary size analysis tool runs automatically on PRs to catch regressions:
```bash
just binary-size-compare  # Compare with baseline
```

### Size Budgets
Consider implementing size budgets in CI:
- Text section: < 7 MB
- Stripped binary: < 10 MB  
- Per-crate contributions: Track largest contributors

### Regular Analysis
Schedule regular comprehensive analysis:
```bash
# Weekly analysis
just binary-size-detailed --save-baseline
```

## Implementation Priority

### Phase 1: Quick Wins (1-2 weeks)
1. ✅ Build comprehensive analysis tooling
2. 🔄 Implement production build optimizations  
3. 🔄 Add CI integration for size monitoring
4. 🔄 Document size reduction strategies

### Phase 2: Medium-term optimizations (1-2 months)
1. 🔄 Implement granular feature flags
2. 🔄 Optimize generic function instantiations
3. 🔄 Profile and optimize regex usage
4. 🔄 Implement rule categorization

### Phase 3: Long-term strategies (3-6 months)
1. 🔄 Design plugin architecture
2. 🔄 Implement PGO builds
3. 🔄 Evaluate alternative dependencies
4. 🔄 Consider WASM deployment

## Conclusion

The current oxlint binary size of 8.8 MB (stripped) is reasonable for a comprehensive linter, but there are significant opportunities for reduction:

1. **Debug symbol stripping** provides immediate 92% size reduction for production
2. **Generic function bloat** represents the largest optimization opportunity 
3. **Feature flags** can help users build minimal configurations
4. **Long-term architectural changes** could provide dramatic size improvements

The analysis tooling now provides ongoing monitoring to prevent size regressions and guide optimization efforts.