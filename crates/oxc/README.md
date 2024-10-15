<p align="center">
  <img alt="OXC Logo" src="https://cdn.jsdelivr.net/gh/oxc-project/oxc-assets/preview-universal.png" width="700">
</p>

<div align="center">

[![Crate][crate-oxc-badge]][crate-oxc-url]
[![Docs][docs-oxc-badge]][docs-oxc-url]
[![GitHub][github-badge]][github-url]
[![Website][website-badge]][website-url]
[![Playground][playground-badge]][playground-url]

</div>

## ⚓ Oxc

The Oxidation Compiler is a high-performance web toolchain. This is an umbrella
crate re-exporting all of oxc's different tools. It also adds higher-level APIs
for stitching various components together that are not found in other oxc
crates.

## ⚡️ Quick Start

The easiest way to get started with `oxc` is by adding this to your `Cargo.toml`:

```toml
[dependencies]
oxc = { version = "*", features = ["full"] }
```

In most cases, code using oxc will follow this general pipeline:

1. Parse source code into an AST
2. Run semantic analysis on the AST
3. Use the AST and semantic data in one or more other tools
4. Generate new code for the final processed program

### Example

This example performs the first two steps of this pipeline:

```rust
use std::path::Path;

use oxc::{
    allocator::Allocator,
    parser::{Parser, ParserReturn},
    span::SourceType,
    semantic::{SemanticBuilder, SemanticBuilderReturn}
};

// In real code, this will likely come from a file read from disk.
let source_path = Path::new("test.tsx");
let source_text = "
import React from 'react';
export interface Props {
    count: number;
    onInc: () => void;
    onDec: () => void;
}
export const Counter: React.FC<Props> = props => {
    return (
        <div>
            <button onClick={props.onInc}>+</button>
            <span id='count'>{props.count}</span>
            <button onClick={props.onDec}>-</button>
        </div>
    );
};
";

// Memory arena where AST nodes are allocated.
let allocator = Allocator::default();
// Infer source type (TS/JS/ESM/JSX/etc) based on file extension
let source_type = SourceType::from_path(source_path).unwrap();
let mut errors = Vec::new();

// Step 1: Parsing
// Parse the TSX file into an AST. The root AST node is a `Program` struct.
let ParserReturn { program, trivias, errors: parser_errors, panicked } =
    Parser::new(&allocator, source_text, source_type).parse();
errors.extend(parser_errors);

// Parsing failed completely. `program` is empty and `errors` isn't. If the
// parser could recover from errors, `program` will be a valid AST and
// `errors` will be populated. We can still perform semantic analysis in
// such cases (if we want).
if panicked {
    for error in &errors {
        eprintln!("{error:?}");
        panic!("Parsing failed.");
    }
}

// Step 2: Semantic analysis.
// Some of the more expensive syntax checks are deferred to this stage, and are
// enabled using `with_check_syntax_error`. You are not required to enable
// these, and they are disabled by default.
let SemanticBuilderReturn {
    semantic,
    errors: semantic_errors,
} = SemanticBuilder::new()
    .with_check_syntax_error(true) // Enable extra syntax error checking
    .with_build_jsdoc(true)        // Enable JSDoc parsing
    .with_cfg(true)                // Build a Control Flow Graph
    .build(&program);              // Produce the `Semantic`

errors.extend(semantic_errors);
if errors.is_empty() {
    println!("parsing and semantic analysis completed successfully.");
} else {
    for error in errors {
        eprintln!("{error:?}");
        panic!("Failed to build Semantic for Counter component.");
    }
}

// From here, you can now pass `program` and `semantic` to other tools.
```

## 💡 Features

These feature flags enable/disable various tools in oxc's toolchain:

- `full`: Enable all features that provide access to a tool.
- `semantic`: Enable the `semantic` module for semantic analysis on ASTs.
- `transformer`: Enable the `transform` module for babel-like transpiling.
- `minifier`: Enable the `minifier` and `mangler` modules for terser-like minification.
- `codegen`: Enable the `codegen` module, which prints ASTs to source code.
- `mangler`: Enable the `mangler` module without enabling `minifier`.
- `cfg`: Expose the `cfg` module. CFGs may still be created in `semantic`
  without turning this on.
- `sourcemap`: Enable the `sourcemap` module. Useful when using `codegen` to
  print both source code and source maps.
- `isolated_declarations`: enable the `isolated_declarations` module for
  generating typescript type declarations

These feature flags modify the behavior of oxc's tools. None of them are enabled
by the `full` feature.

- `serialize`: Implements `Serialize` and `Deserialize` for various oxc data
  structures.
- `sourcemap_concurrent`: Generate source maps in parallel. Only useful when
  the `sourcemap` feature is also enabled.
- `wasm`: Enable WASM bindings for the transformer/transpiler. Only useful when
  the `transformer` feature is enabled.

[crate-oxc-badge]: https://img.shields.io/crates/v/oxc?style=flat-square&logo=rust
[crate-oxc-url]: https://crates.io/crates/oxc
[docs-oxc-badge]: https://img.shields.io/badge/docs.rs-oxc-66c2a5?style=flat-square&logo=rust
[docs-oxc-url]: https://docs.rs/oxc
[github-badge]: https://img.shields.io/badge/GitHub-oxc--project%2Foxc-8da0cb?style=flat-square&logo=github
[github-url]: https://github.com/oxc-project/oxc
[playground-badge]: https://img.shields.io/badge/Playground-blue?color=9BE4E0?style=flat-square
[playground-url]: https://playground.oxc.rs/
[website-badge]: https://img.shields.io/badge/Website-blue?style=flat-square
[website-url]: https://oxc.rs
