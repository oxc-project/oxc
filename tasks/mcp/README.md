# Oxc MCP Server Boilerplate

This crate provides a boilerplate implementation of a Model Context Protocol (MCP) server using the [rmcp](https://docs.rs/rmcp/0.5.0/rmcp/) Rust SDK. It demonstrates how to create and integrate MCP servers within the Oxc project ecosystem.

## Overview

The Model Context Protocol (MCP) is a protocol for enabling AI assistants to securely connect to and interact with local and remote resources. This boilerplate provides a foundation for building MCP servers that can integrate with Oxc's JavaScript/TypeScript tooling capabilities.

## Features

- **Basic MCP Server**: A simple server implementation with metadata and initialization
- **Example Tools**: 
  - `EchoTool`: Simple echo functionality for testing
  - `TextAnalyzerTool`: Basic text analysis (word count, character count, etc.)
- **Interactive Mode**: Command-line interface for testing tools
- **Extensible Design**: Easy to add new tools and capabilities

## Usage

### As a Library

```rust
use oxc_mcp::{OxcMcpServer, EchoTool, TextAnalyzerTool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = OxcMcpServer::new();
    server.initialize().await?;
    
    // Use tools
    let echo_tool = EchoTool::new("Hello MCP!".to_string());
    println!("{}", echo_tool.execute());
    
    Ok(())
}
```

### Running the Example

```bash
# From the oxc root directory
cargo run --bin oxc_mcp
```

This will start an interactive demo that showcases the MCP server capabilities.

## Extending the Boilerplate

### Adding New Tools

1. Create a new tool struct that implements `serde::Serialize` and `serde::Deserialize`
2. Implement an `execute` method that returns the tool's result
3. Add the tool to the server's tool registry

Example:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MyCustomTool {
    pub input: String,
}

impl MyCustomTool {
    pub fn new(input: String) -> Self {
        Self { input }
    }

    pub fn execute(&self) -> String {
        // Your tool logic here
        format!("Processed: {}", self.input)
    }
}
```

### Integrating with Oxc Tools

To integrate with Oxc's parsing, linting, or transformation capabilities, uncomment the oxc dependencies in `Cargo.toml`:

```toml
oxc_allocator = { workspace = true }
oxc_parser = { workspace = true }
oxc_ast = { workspace = true }
```

Then you can create tools that leverage Oxc's functionality:

```rust
use oxc_parser::Parser;
use oxc_allocator::Allocator;

// Tool that parses JavaScript/TypeScript code
pub struct ParserTool {
    pub source_code: String,
}

impl ParserTool {
    pub fn execute(&self) -> Result<String, String> {
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, &self.source_code, SourceType::tsx());
        let result = parser.parse();
        
        if result.errors.is_empty() {
            Ok("Parse successful".to_string())
        } else {
            Err(format!("Parse errors: {:?}", result.errors))
        }
    }
}
```

## Dependencies

- [rmcp](https://docs.rs/rmcp/0.5.0/rmcp/): Rust SDK for Model Context Protocol
- [tokio](https://tokio.rs/): Async runtime
- [serde](https://serde.rs/): Serialization/deserialization
- [serde_json](https://docs.rs/serde_json/): JSON support

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.