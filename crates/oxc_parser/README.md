Oxc Parser for JavaScript and TypeScript

Oxc's [`Parser`] has full support for
- The latest stable ECMAScript syntax
- TypeScript
- JSX and TSX
- [Stage 3 Decorators](https://github.com/tc39/proposal-decorator-metadata)

# Usage

The parser has a minimal API with three inputs (a [memory arena](oxc_allocator::Allocator), a
source string, and a [`SourceType`]) and one return struct (a [ParserReturn]).

```rust
let parser_return = Parser::new(&allocator, &source_text, source_type).parse();
```

# Abstract Syntax Tree (AST)
Oxc's AST is located in a separate [`oxc_ast`] crate. You can find type definitions for AST
nodes [here][`oxc_ast::ast`].

# Performance

The following optimization techniques are used:
* AST is allocated in a memory arena ([bumpalo](https://docs.rs/bumpalo)) for fast AST drop
* [`oxc_span::Span`] offsets uses `u32` instead of `usize`
* Scope binding, symbol resolution and complicated syntax errors are not done in the parser,
they are delegated to the [semantic analyzer](https://docs.rs/oxc_semantic)

<div class="warning">
Because [`oxc_span::Span`] uses `u32` instead of `usize`, Oxc can only parse files up
to 4 GiB in size. This shouldn't be a limitation in almost all cases.
</div>

# Examples

<https://github.com/oxc-project/oxc/blob/main/crates/oxc_parser/examples/parser.rs>

```rust
#![allow(clippy::print_stdout)]
use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `cargo watch -x "run -p oxc_parser --example parser"`

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let show_ast = args.contains("--ast");
    let show_comments = args.contains("--comments");

    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions { parse_regular_expression: true, ..ParseOptions::default() })
        .parse();

    if show_ast {
        println!("AST:");
        println!("{}", serde_json::to_string_pretty(&ret.program).unwrap());
    }

    if show_comments {
        println!("Comments:");
        for comment in ret.trivias.comments() {
            let s = comment.real_span().source_text(&source_text);
            println!("{s}");
        }
    }

    if ret.errors.is_empty() {
        println!("Parsed Successfully.");
    } else {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
            println!("Parsed with Errors.");
        }
    }

    Ok(())
}
```

### Parsing TSX
```rust
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

fn main() {
    let source_text = r#"
import React from 'react';

/**
 * A simple counter component
 */
export const Counter: React.FC = () => {
    const [count, setCount] = React.useState(0);

    return (
        <div>
            <p>Count: {count}</p>
            <button onClick={() => setCount(count + 1)}>Increment</button>
            <button onClick={() => setCount(count - 1)}>Decrement</button>
        </div>
    )
}"#;

    // Memory arena where AST nodes get stored
    let allocator = Allocator::default();
    // Infers TypeScript + JSX + ESM modules
    let source_type = SourceType::from_path("Counter.tsx").unwrap();

    let ParserReturn {
        program,  // AST
        errors,   // Syntax errors
        panicked, // Parser encountered an error it couldn't recover from
        trivias,  // Comments, whitespace, etc.
    } = Parser::new(&allocator, source_text, source_type).parse();

    assert!(!panicked);
    assert!(errors.is_empty());
    assert!(!program.body.is_empty());
    assert_eq!(trivias.comments().count(), 1);
}
```

# Visitor

See [oxc_ast::Visit] and [oxc_ast::VisitMut]

# Visiting without a visitor

For ad-hoc tasks, the semantic analyzer can be used to get a parent pointing tree with untyped nodes,
the nodes can be iterated through a sequential loop.

```rust
for node in semantic.nodes().iter() {
    match node.kind() {
        // check node
    }
}
```

See [full linter example](https://github.com/Boshen/oxc/blob/ab2ef4f89ba3ca50c68abb2ca43e36b7793f3673/crates/oxc_linter/examples/linter.rs#L38-L39)

[`SourceType`]: oxc_span::SourceType
