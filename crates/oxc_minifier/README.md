# Oxc Minifier

Next-generation JavaScript/TypeScript minifier achieving best-in-class compression.

## Inspiration

- **Closure Compiler**: Advanced size optimizations
- **Terser/UglifyJS**: Comprehensive battle-tested transforms
- **esbuild**: Efficient algorithms and architecture
- **SWC**: Modern Rust performance

## Key Features

- Maximum compression through exhaustive optimizations
- 100% correctness with comprehensive testing
- Fixed-point iteration for optimal size
- Arena allocation for performance

## Current Performance

See [`tasks/minsize`](../../tasks/minsize) for compression benchmarks.

- Matching/beating esbuild on many libraries
- Full test262, Babel, TypeScript conformance

## Usage

```rust
use oxc_minifier::{Minifier, MinifierOptions};

let options = MinifierOptions::default();
let minifier = Minifier::new(options);
let result = minifier.minify(&mut program);
```

## Testing Infrastructure

- `just minsize` - Track compression benchmarks
- `cargo coverage` - Conformance tests (test262, Babel, TypeScript)
- `tasks/e2e` - Real-world E2E testing

## Development

- `just test` - Run all tests
- `cargo run -p oxc_minifier --example minifier` - Try the minifier

## Key Dependencies

- [`oxc_ecmascript`](../oxc_ecmascript) - ECMAScript operations and constant evaluation
- [`oxc_semantic`](../oxc_semantic) - Scope and symbol analysis
- [`oxc_mangler`](../oxc_mangler) - Variable renaming

## Documentation

- [Architecture](./docs/ARCHITECTURE.md) - Design and components
- [Optimizations](./docs/OPTIMIZATIONS.md) - Complete optimization catalog
- [Assumptions](./docs/ASSUMPTIONS.md) - Code assumptions for optimization
- [Correctness](./docs/CORRECTNESS.md) - Testing and validation
- [Roadmap](./docs/ROADMAP.md) - Development plan
- [Claude Guide](./docs/CLAUDE.md) - AI assistant reference
