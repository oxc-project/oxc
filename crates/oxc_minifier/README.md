# Oxc Minifier

JavaScript minifier focused on small output and performance. Transform TypeScript,
TSX, and JSX to JavaScript before minification.

## Inspiration

- **Closure Compiler**: Advanced size optimizations
- **Terser/UglifyJS**: Comprehensive battle-tested transforms
- **esbuild**: Efficient algorithms and architecture
- **SWC**: Modern Rust performance

## Key Features

- Maximum compression through exhaustive optimizations
- Correctness testing under the [documented assumptions](./docs/ASSUMPTIONS.md)
- Fixed-point iteration for optimal size
- Arena allocation for performance

## Current Performance

See [`tasks/minsize`](../../tasks/minsize) for compression benchmarks.

- Matching/beating esbuild on many libraries
- Minifier idempotency checks on selected test262 and Babel cases

## Usage

```rust
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

let allocator = Allocator::default();
let parsed = Parser::new(&allocator, "const x = 1 + 1; console.log(x);", SourceType::mjs())
    .parse();
assert!(parsed.diagnostics.is_empty());
let mut program = parsed.program;

let result = Minifier::new(MinifierOptions::default()).minify(&allocator, &mut program);
let output = Codegen::new()
    .with_options(CodegenOptions { minify: true, ..CodegenOptions::default() })
    .with_scoping(result.scoping)
    .with_private_member_mappings(result.class_private_mappings)
    .build(&program);
println!("{}", output.code);
```

## Testing Infrastructure

- `just minsize` - Track compression benchmarks
- `cargo coverage minifier` - Minifier idempotency tests (test262 and Babel)
- `cargo coverage runtime` - Execute eligible test262 cases with Node.js
- `tasks/e2e` - Real-world E2E testing

## Development

- `just test` - Run all tests
- `cargo run -p oxc_minifier --example minifier -- test.js` - Try the minifier

## Key Dependencies

- [`oxc_ecmascript`](../oxc_ecmascript) - ECMAScript operations and constant evaluation
- [`oxc_semantic`](../oxc_semantic) - Scope and symbol analysis
- [`oxc_mangler`](../oxc_mangler) - Variable renaming

## Documentation

- [Architecture](./docs/ARCHITECTURE.md) - Design and components
- [Optimizations](./docs/OPTIMIZATIONS.md) - Optimization overview
- [Assumptions](./docs/ASSUMPTIONS.md) - Code assumptions for optimization
- [Correctness](./docs/CORRECTNESS.md) - Testing and validation
- [Roadmap](./docs/ROADMAP.md) - Development plan
- [Agent Guide](./AGENTS.md) - AI assistant reference
