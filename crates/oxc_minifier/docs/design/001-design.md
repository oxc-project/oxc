# Oxc Minifier Design

Next-generation JavaScript/TypeScript minifier achieving best-in-class compression.

## Inspiration

- **[Closure Compiler](https://github.com/google/closure-compiler)**: Advanced size optimizations
- **[Terser](https://github.com/terser/terser)/[UglifyJS](https://github.com/mishoo/UglifyJS)**: Comprehensive battle-tested transforms
- **[esbuild](https://github.com/evanw/esbuild)**: Efficient algorithms and architecture
- **[SWC](https://github.com/swc-project/swc)**: Modern Rust performance

## Key Features

- Maximum compression through exhaustive optimizations
- 100% correctness with comprehensive testing
- Fixed-point iteration for optimal size
- Arena allocation for performance

## Not Included

The following optimizations were considered but are deliberately excluded from scope:

- **Property Flattening (Namespace Collapsing)** — Flatten `a.b.c` → `a$b$c`. Closure-only (`CollapseProperties.java`). Requires global namespace assumptions that don't hold for most modern JS.
- **Cross-Module Code Motion** — Move definitions across module boundaries. Closure-only (`CrossModuleCodeMotion.java`). Requires bundler-level integration and whole-program module graph.
- **Type-based Property Disambiguation** — Use type info to rename same-named properties on different types independently. Closure-only (`DisambiguateProperties.java`, `AmbiguateProperties.java`). Requires full type system.
- **Method Devirtualization** — Convert instance methods to static functions. Closure-only (`DevirtualizeMethods.java`). Requires type info and class hierarchy analysis.
- **Expression Decomposition** — Break complex expressions into temp variables to enable other passes. Closure-only (`ExpressionDecomposer.java`). Internal technique, not a user-facing optimization.
- **Inline Properties (type-based)** — Inline property accesses using type information. Closure-only (`InlineProperties.java`). Requires full type system.
- **Function Rewriter** — Rewrite functions for optimization. Closure-only (`FunctionRewriter.java`). Off by default due to performance regressions.
- **Replace Strings** — Replace string literals with shorter aliases. Closure-only (`ReplaceStrings.java`). Application-specific, not generally applicable.
- **Rescope Global Symbols** — Move global declarations into a scope wrapper. Closure-only (`RescopeGlobalSymbols.java`). Changes module semantics.
- **Private Field Mangling** — Rename `#privateField` to `#a`. No major minifier implements this. Complexity-to-benefit ratio is poor: private fields are rare, `#` prefix is always present, edge cases with `in` checks and subclass inheritance.
- **Regex Optimization** — Shorten character classes, remove redundant escapes. No major JS minifier implements significant regex optimization. High complexity, minimal savings.

## Correctness Invariants

### Multi-pass diminishing returns

The fixed-point optimization loop should compare output size between iterations and stop when further passes increase it. More passes can sometimes produce _worse_ output due to interaction effects between transforms. ([Terser #1554](https://github.com/terser/terser/issues/1554))

### Statement execution order preservation

Optimization passes must never change statement evaluation order. Reordering statements can alter observable behavior when expressions have side effects. ([SWC #8437](https://github.com/swc-project/swc/issues/8437), [SWC #9485](https://github.com/swc-project/swc/issues/9485))
