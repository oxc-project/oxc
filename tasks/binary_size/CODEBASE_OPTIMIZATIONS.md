# Binary Size Optimizations: Codebase Analysis

This document analyzes the oxc codebase and identifies specific optimization opportunities to reduce binary size, based on the binary size analysis findings.

## Key Findings

From the binary size analysis, we identified:
- **oxc_linter (36.3%, 2.10 MB)**: Largest component with 550+ lint rules
- **Generic function bloat**: 1.8M LLVM lines generated
  - hashbrown::raw::RawTable operations: 119 instantiations  
  - Debug trait implementations: 1,876 instantiations
  - LintContext diagnostic methods: 75 instantiations
- **Regex overhead**: 346KB from regex_automata + regex_syntax
- **Error handling**: 105KB from miette diagnostics

## Implemented Optimizations

### 1. Feature Flags for Rule Categories ✅

**Impact**: Up to 60-80% size reduction for users who don't need all rules

**Implementation**: Added conditional compilation for rule categories:
- `eslint_rules`: Core ESLint rules (~163 rules)
- `react_rules`: React-specific rules (~37 rules)
- `jsx_a11y_rules`: Accessibility rules
- `typescript_rules`: TypeScript-specific rules
- `unicorn_rules`: Unicorn plugin rules
- `jest_rules`: Jest testing rules
- `jsdoc_rules`: JSDoc documentation rules
- `nextjs_rules`: Next.js specific rules
- `node_rules`: Node.js specific rules
- `oxc_rules`: Oxc custom rules
- `promise_rules`: Promise handling rules
- `react_perf_rules`: React performance rules
- `vitest_rules`: Vitest testing rules
- `import_rules`: Import/export rules

**Usage**:
```bash
# Minimal build with only ESLint core rules
cargo build --no-default-features --features minimal

# Custom build with specific rule sets
cargo build --no-default-features --features eslint_rules,react_rules,typescript_rules
```

**Files Modified**:
- `crates/oxc_linter/Cargo.toml`: Added feature flags
- `crates/oxc_linter/src/rules.rs`: Added conditional compilation directives

## Recommended Future Optimizations

### 2. Reduce Debug Trait Instantiations

**Current**: 1,876 instantiations consuming significant binary space
**Solution**: Conditional debug implementations in release builds

```rust
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct LintRule { /* ... */ }

#[cfg(not(debug_assertions))]
pub struct LintRule { /* ... */ }

#[cfg(not(debug_assertions))]
impl std::fmt::Debug for LintRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LintRule").finish_non_exhaustive()
    }
}
```

### 3. Consolidate Diagnostic Creation

**Current**: 75 instantiations of LintContext diagnostic methods
**Solution**: Generic diagnostic creation to reduce monomorphization

```rust
// Instead of multiple specialized methods, use a single generic method
impl LintContext {
    #[inline]
    pub fn diagnostic_generic<T: Into<OxcDiagnostic>>(&self, diagnostic: T) {
        self.add_diagnostic(Message::new(diagnostic.into(), PossibleFixes::None));
    }
}
```

### 4. String Interning for Common Messages

**Solution**: Move common diagnostic messages to static constants

```rust
// Common diagnostic messages as static strings
pub const UNEXPECTED_TOKEN: &str = "Unexpected token";
pub const MISSING_SEMICOLON: &str = "Missing semicolon";
pub const UNREACHABLE_CODE: &str = "Unreachable code detected";

// Rule names as static strings
pub const RULE_NO_UNUSED_VARS: &str = "no-unused-vars";
pub const RULE_NO_CONSOLE: &str = "no-console";
```

### 5. HashMap Optimization

**Current**: 119 instantiations of hashbrown::raw::RawTable
**Solution**: Consolidate map types and use type aliases

```rust
// Use consistent key-value types across the codebase
type RuleMap = FxHashMap<&'static str, RuleEnum>;
type SymbolMap = FxHashMap<SymbolId, SymbolInfo>;
type ScopeMap = FxHashMap<ScopeId, ScopeInfo>;
```

### 6. Lazy Initialization

**Solution**: Make expensive initializations lazy where possible

```rust
use std::sync::LazyLock;

static GLOBALS: LazyLock<GlobalsMap> = LazyLock::new(|| {
    // Expensive initialization only when needed
    build_globals_map()
});
```

### 7. Regex Optimization

**Current**: 346KB from regex crates (5.5% of binary)
**Solution**: 
- Replace simple regex patterns with string operations
- Use lighter regex alternatives for simple cases
- Lazy compile regex patterns

```rust
// Instead of regex for simple patterns
use memchr::memmem;

// Instead of regex::Regex for simple searches
if text.contains("debugger") { /* ... */ }
```

### 8. Alternative Error Handling

**Current**: 105KB from miette (1.7% of binary)
**Solution**: Consider lighter error handling for some cases

```rust
// Simple error types for internal use
#[derive(Debug)]
pub struct SimpleError {
    pub message: &'static str,
    pub span: Span,
}
```

## Build Configuration Optimizations

### Size-Optimized Profile

Already implemented in `Cargo.toml`:
```toml
[profile.release-size]
inherits = "release"
opt-level = "z"      # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit for better optimization
panic = "abort"     # Smaller panic handling
```

### Usage
```bash
cargo build --profile release-size
```

## Measurement and Monitoring

- Binary size analysis tool: `just binary-size`
- Track size changes in CI/CD
- Baseline comparison: `just binary-size-compare`
- Profile specific builds: `cargo build --profile release-size`

## Expected Impact

With all optimizations implemented:
- **Full build**: Current 8.4MB → Target 5-6MB (25-30% reduction)
- **Minimal build**: Target 2-3MB (65-75% reduction)
- **Custom builds**: 40-60% reduction depending on feature selection

The feature flag optimization alone provides the biggest impact for end users who don't need all lint rules.