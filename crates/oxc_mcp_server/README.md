# Oxc Model Context Protocol Server

A Model Context Protocol (MCP) server that provides AI models with access to Oxc's JavaScript/TypeScript tooling capabilities.

## Features

The Oxc MCP server exposes the following capabilities:

### Resources
- **`oxc://linter/rules`**: Information about available linter rules and their descriptions
- **`oxc://project/info`**: Overview of the Oxc project structure and capabilities  
- **`oxc://ast/schema`**: Schema definition for Oxc's Abstract Syntax Tree

### Tools
- **`lint_code`**: Lint JavaScript/TypeScript code using the Oxc linter (simplified version)
- **`parse_code`**: Parse JavaScript/TypeScript code and return AST information
- **`analyze_code`**: Perform semantic analysis on JavaScript/TypeScript code
- **`format_code`**: Format JavaScript/TypeScript code using Oxc's codegen

### Prompts
- **`analyze_js_code`**: Template for analyzing JavaScript/TypeScript code for issues
- **`explain_linter_rules`**: Template for explaining linter rules and their purpose
- **`code_quality_review`**: Template for comprehensive code quality reviews

## Usage

### Running the Server

The server communicates over stdio using JSON-RPC 2.0:

```bash
cargo run -p oxc_mcp_server
```

### Example Requests

#### Initialize the server
```json
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
```

#### List available tools
```json
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
```

#### Parse JavaScript code
```json
{
  "jsonrpc": "2.0", 
  "id": 3, 
  "method": "tools/call", 
  "params": {
    "name": "parse_code", 
    "arguments": {
      "code": "const x = 1; console.log(x);",
      "filename": "example.js"
    }
  }
}
```

#### Read project information
```json
{
  "jsonrpc": "2.0", 
  "id": 4, 
  "method": "resources/read", 
  "params": {
    "uri": "oxc://project/info"
  }
}
```

#### Get a code analysis prompt
```json
{
  "jsonrpc": "2.0", 
  "id": 5, 
  "method": "prompts/get", 
  "params": {
    "name": "analyze_js_code",
    "arguments": {
      "code": "var x = 1; x = 2;"
    }
  }
}
```

## Integration with MCP Clients

This server can be used with any MCP-compatible client, such as:
- Claude Desktop
- Custom MCP clients
- AI development environments

Add the following configuration to your MCP client:

```json
{
  "mcpServers": {
    "oxc": {
      "command": "cargo",
      "args": ["run", "-p", "oxc_mcp_server"],
      "cwd": "/path/to/oxc"
    }
  }
}
```

## Architecture

The server is implemented as a simple JSON-RPC server that:
- Reads JSON-RPC requests from stdin
- Processes them using Oxc's parsing, linting, and analysis capabilities
- Returns JSON-RPC responses to stdout

This design makes it compatible with the standard MCP protocol while keeping the implementation simple and focused.

## Current Limitations

- Linting functionality is simplified and doesn't use the full Oxc linter configuration system
- Some advanced semantic analysis features are not exposed
- Error handling could be more detailed
- No support for streaming or batch operations

## Future Enhancements

- Full linter integration with proper configuration support
- More detailed AST traversal and analysis tools
- Integration with Oxc's transformer and minifier
- Support for TypeScript-specific analysis
- Batch processing capabilities
- Enhanced error reporting and diagnostics