# OXC JavaScript Minifier

A high-performance JavaScript minifier written in Rust that reduces code size through various optimization techniques.

## Architecture

The JavaScript minifier consists of three main components that work together:

1. **Compressor** - Rewrites statements and expressions for minimal output
2. **Mangler** - Shortens variable and function names 
3. **Printer** - Removes whitespace and formats the final output

```
JavaScript Source → Parser → AST → Compressor → Mangler → Printer → Minified Output
```

## Usage

### Basic Usage

```rust
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_minifier::{Minifier, MinifierOptions};

let allocator = Allocator::default();
let mut program: Program = /* your parsed AST */;

let minifier = Minifier::new(MinifierOptions::default());
let result = minifier.build(&allocator, &mut program);
```

### Custom Options

```rust
use oxc_minifier::{CompressOptions, MangleOptions, MinifierOptions};

let options = MinifierOptions {
    mangle: Some(MangleOptions::default()),
    compress: Some(CompressOptions {
        drop_console: true,
        dead_code_elimination: true,
        ..Default::default()
    }),
};

let minifier = Minifier::new(options);
```

## Components

### Compressor

The compressor applies various optimizations to reduce code size:

- **Peephole Optimizations**: Local code transformations (constant folding, dead code elimination)
- **Control Flow Optimization**: Simplifies conditional statements and loops
- **Expression Minimization**: Reduces complex expressions to simpler forms
- **Dead Code Elimination**: Removes unreachable code

The compressor is heavily inspired by [Terser](https://github.com/terser/terser) and implements similar optimization strategies.

### Mangler

The mangler is integrated with `oxc_semantic` and provides:

- Variable name shortening with scope awareness
- Gzip-friendly name generation
- Preservation of semantic correctness
- Support for keep lists to preserve specific names

### Printer

The printer handles the final output formatting:
- Whitespace removal
- Comment stripping  
- Optimal character encoding
- Source map generation (when enabled)

## Optimization Techniques

### Peephole Optimizations

The minifier applies numerous peephole optimizations in multiple passes:

- **Constant Folding**: Evaluates constant expressions at compile time
- **Dead Code Elimination**: Removes unreachable or unused code
- **Expression Minimization**: Simplifies boolean expressions and conditions
- **Statement Fusion**: Combines related statements for better compression
- **Method Replacement**: Replaces known methods with shorter equivalents

### Control Flow Optimization

- **Conditional Simplification**: Reduces complex if/else chains
- **Loop Optimization**: Simplifies for/while loop structures
- **Early Exit Minimization**: Optimizes return/break/continue statements

## Safety Assumptions

The minifier makes several assumptions about the input code to enable aggressive optimizations:

- [Properties of the global object defined in the ECMAScript spec](https://tc39.es/ecma262/multipage/global-object.html#sec-global-object) behaves the same as in the spec
  - Examples of properties: `Infinity`, `parseInt`, `Object`, `Promise.resolve`
  - Examples that breaks this assumption: `globalThis.Object = class MyObject {}`
- The code does not rely on the `name` property of `Function` or `Class`
  - Examples that breaks this assumption: `function fn() {}; console.log(f.name === 'fn')`
- [`document.all`](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-IsHTMLDDA-internal-slot) is not used or behaves as a normal object
  - Examples that breaks this assumption: `console.log(typeof document.all === 'undefined')`
- TDZ violation does not happen
  - Examples that breaks this assumption: `(() => { console.log(v); let v; })()`
- `with` statement is not used
  - Examples that breaks this assumption: `with (Math) { console.log(PI); }`
- `.toString()`, `.valueOf()`, `[Symbol.toPrimitive]()` are side-effect free
  - Examples that breaks this assumption: `{ toString() { console.log('sideeffect') } }`
- Errors thrown when creating a String or an Array that exceeds the maximum length can disappear or moved
  - Examples that breaks this assumption: `try { new Array(Number(2n**53n)) } catch { console.log('log') }`
- Invalid super class error does not happen
  - Examples that breaks this assumption: `const v = []; class A extends v {}`

## Testing

The minifier includes comprehensive tests based on:

- **Terser Test Suite**: Fixtures copied from [Terser v5.9.0](https://github.com/terser/terser/tree/v5.9.0/test/compress)
- **Custom Test Cases**: OXC-specific optimizations and edge cases
- **Integration Tests**: End-to-end minification workflows

Run tests with:
```bash
cargo test --package oxc_minifier
```

## Contributing

When adding new optimizations:

1. Add comprehensive test cases covering edge cases
2. Document the optimization with clear examples
3. Ensure the optimization is safe and doesn't change semantics
4. Follow the existing code patterns and naming conventions

See the individual peephole optimization modules for examples of well-documented optimizations.
