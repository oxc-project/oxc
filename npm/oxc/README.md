# @oxc-project/oxc

> MCP Server for Oxc - JavaScript/TypeScript parsing, linting, and code analysis

[![npm version](https://badge.fury.io/js/@oxc-project%2Foxc.svg)](https://badge.fury.io/js/@oxc-project%2Foxc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This package provides a [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server for [Oxc](https://oxc.rs), enabling AI assistants to interact with Oxc's high-performance JavaScript/TypeScript parsing, linting, and analysis capabilities.

## Features

- **Fast Code Analysis**: Parse and analyze JavaScript/TypeScript code using Oxc's high-performance parser
- **Smart Linting**: Run Oxc linter with intelligent rule detection and suggestions
- **AST Exploration**: Generate and explore Abstract Syntax Trees for code understanding
- **Documentation Resources**: Access comprehensive Oxc documentation and examples
- **Interactive Prompts**: Pre-configured prompts for common code analysis tasks

## Installation

```bash
npm install @oxc-project/oxc
```

## Usage

### As an MCP Server

The package includes a standalone MCP server that can be used with any MCP-compatible client:

```bash
# Start the MCP server (stdio transport)
npx oxc-mcp

# Or use with a specific transport
node dist/bin/server.js
```

### Configuration for MCP Clients

Add to your MCP client configuration (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "oxc": {
      "command": "npx",
      "args": ["@oxc-project/oxc"]
    }
  }
}
```

### Available Tools

#### `analyze-code`
Parse and analyze JavaScript/TypeScript code using Oxc parser.

**Parameters:**
- `code` (string): JavaScript or TypeScript code to analyze
- `language` (optional): Language type ("javascript" or "typescript")

**Example:**
```typescript
const result = await client.callTool({
  name: "analyze-code",
  arguments: {
    code: "function hello(name) { return `Hello, ${name}!`; }",
    language: "javascript"
  }
});
```

#### `lint-code`
Run Oxc linter on JavaScript/TypeScript code.

**Parameters:**
- `code` (string): JavaScript or TypeScript code to lint
- `rules` (optional): Array of specific linting rules to apply

**Example:**
```typescript
const result = await client.callTool({
  name: "lint-code",
  arguments: {
    code: "var x = 5; if (x == 5) console.log('hello');",
    rules: ["no-var", "eqeqeq", "no-console"]
  }
});
```

### Available Resources

#### AST Examples (`ast://`)
- `ast://example`: Basic AST structure examples

#### Documentation (`docs://`)
- `docs://getting-started`: Introduction to Oxc
- `docs://parser`: Parser documentation
- `docs://linter`: Linter documentation  
- `docs://ast`: AST documentation

### Available Prompts

#### `code-examples`
Generate JavaScript/TypeScript code examples for specific patterns.

**Parameters:**
- `pattern` (string): Code pattern to generate examples for
- `language` (optional): Target language ("javascript" or "typescript")

#### `analyze-prompt`
Generate prompts for analyzing code with specific focus areas.

**Parameters:**
- `focusArea`: Analysis focus ("performance", "security", "maintainability", "readability")
- `codeType`: Type of code ("function", "class", "module", "component")

## Task Definitions

The package includes task definitions in the `files/tasks/` directory:

- `analyze-code.json`: Code analysis task configuration
- `lint-code.json`: Code linting task configuration
- `parse-ast.json`: AST parsing task configuration

These task definitions provide structured metadata for the available tools and can be used by MCP clients to understand capabilities.

## Development

### Building

```bash
npm run build
```

### Development Mode

```bash
npm run dev
```

### Testing with MCP Inspector

1. Install the [MCP Inspector](https://github.com/modelcontextprotocol/inspector)
2. Run the server: `npm start`
3. Connect the inspector to test functionality

## API Reference

### Types

```typescript
interface CodeAnalysis {
  language: string;
  codeLength: number;
  lineCount: number;
  hasImports: boolean;
  hasExports: boolean;
  functions: number;
  arrows: number;
}

interface LintResult {
  issues: string[];
  hasErrors: boolean;
  hasWarnings: boolean;
}

interface OxcServerOptions {
  enableParser?: boolean;
  enableLinter?: boolean;
  enableAnalysis?: boolean;
  customRules?: string[];
}
```

## Integration Examples

### With Claude Desktop

```json
{
  "mcpServers": {
    "oxc": {
      "command": "npx",
      "args": ["@oxc-project/oxc"],
      "env": {
        "NODE_ENV": "production"
      }
    }
  }
}
```

### With Custom MCP Client

```typescript
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const transport = new StdioClientTransport({
  command: "npx",
  args: ["@oxc-project/oxc"]
});

const client = new Client({
  name: "my-client",
  version: "1.0.0"
});

await client.connect(transport);

// Use the tools
const analysis = await client.callTool({
  name: "analyze-code",
  arguments: {
    code: "const x = 42;"
  }
});
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see the [LICENSE](https://github.com/oxc-project/oxc/blob/main/LICENSE) file for details.

## Related Projects

- [Oxc](https://oxc.rs) - The Oxidation Compiler collection
- [Model Context Protocol](https://modelcontextprotocol.io) - Protocol specification
- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk) - MCP SDK for TypeScript