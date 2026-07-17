# oxc-graphql-parser

A spec-compliant, error-resilient GraphQL lexer and parser for Rust.

## Features

- Typed GraphQL AST based on the [October 2021 specification]
- Error-resilient lexing and parsing
- GraphQL schema and query parsing
- Standalone lexer API

## Installation

```bash
cargo add oxc-graphql-parser
```

Or add it manually:

```toml
[dependencies]
oxc-graphql-parser = "0.0.1"
```

The Cargo package name uses hyphens. Import it from Rust as `oxc_graphql_parser`.

## Usage

```rust
use oxc_graphql_parser::{Allocator, Parser};

let input = "union SearchResult = Photo | Person | Cat | Dog";
let allocator = Allocator::default();
let parser = Parser::new(&allocator, input);
let ast = parser.parse();

assert_eq!(0, ast.errors().len());
```

`Parser::parse` always returns an AST, even when lexing or parsing reports
errors. Check `ast.errors()` before walking the document:

```rust
use oxc_graphql_parser::{Allocator, Parser};

let input = "union SearchResult = Photo | Person | Cat | Dog";
let allocator = Allocator::default();
let parser = Parser::new(&allocator, input);
let ast = parser.parse();

assert_eq!(0, ast.errors().len());

let document = ast.document();
for definition in &document.definitions {
    println!("{definition:?}");
}
```

## Examples

The [examples directory] contains integrations for diagnostics and analysis:

- [using oxc-graphql-parser with ariadne to display error diagnostics]
- [using oxc-graphql-parser with annotate_snippets to display error diagnostics]
- [checking for unused variables]

### Get Field Names In An Object

```rust
use oxc_graphql_parser::{Allocator, ast, Parser};

let input = "
type ProductDimension {
  size: String
  weight: Float @tag(name: \"hi from inventory value type field\")
}
";

let allocator = Allocator::default();
let parser = Parser::new(&allocator, input);
let ast = parser.parse();

assert_eq!(0, ast.errors().len());

let document = ast.document();
for definition in &document.definitions {
    if let ast::Definition::ObjectType(object_type) = definition {
        assert_eq!(object_type.name.as_str(), "ProductDimension");

        for field in &object_type.fields {
            println!("{}", field.name);
        }
    }
}
```

### Get Variables Used In A Query

```rust
use oxc_graphql_parser::{Allocator, ast, Parser};

let input = "
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
";

let allocator = Allocator::default();
let parser = Parser::new(&allocator, input);
let ast = parser.parse();

assert_eq!(0, ast.errors().len());

let document = ast.document();
for definition in &document.definitions {
    if let ast::Definition::Operation(operation) = definition {
        assert_eq!(operation.name.as_ref().unwrap().as_str(), "GraphQuery");

        let variables: Vec<String> = operation
            .variable_definitions
            .iter()
            .map(|definition| definition.variable.name.to_string())
            .collect();

        assert_eq!(
            variables.as_slice(),
            ["graph_id".to_string(), "variant".to_string()]
        );
    }
}
```

## Rust Versions

`oxc-graphql-parser` is tested on the latest stable version of Rust.
Older versions may or may not be compatible.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

[examples directory]: https://github.com/oxc-project/oxc-graphql-parser/tree/main/crates/oxc_graphql_parser/examples
[using oxc-graphql-parser with ariadne to display error diagnostics]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/ariadne.rs
[using oxc-graphql-parser with annotate_snippets to display error diagnostics]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/annotate_snippet.rs
[checking for unused variables]: https://github.com/oxc-project/oxc-graphql-parser/blob/main/crates/oxc_graphql_parser/examples/unused_vars.rs
[October 2021 specification]: https://spec.graphql.org/October2021
