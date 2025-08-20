# MCP Server for Oxc

This is a Model Context Protocol (MCP) server for the Oxc project that provides access to all Oxc examples as tools. It allows you to parse, lint, format, transform, compile, and analyze JavaScript/TypeScript code using Oxc's powerful toolchain.

## Features

- TypeScript-based MCP server
- All 18 Oxc examples exposed as MCP tools
- Built using @modelcontextprotocol/sdk
- Comprehensive coverage of Oxc functionality:
  - **Parsing**: AST generation, TSX support, regular expressions
  - **Linting**: Code quality analysis
  - **Formatting**: Code beautification
  - **Semantic Analysis**: Symbol resolution, control flow graphs
  - **Transformation**: Code transformation, environment targeting
  - **Compilation**: Complete pipeline processing
  - **Minification**: Code compression, dead code elimination, mangling
  - **Type Processing**: Isolated declarations

## Installation

From the root of the oxc repository:

```bash
pnpm install
```

## Building

```bash
cd tasks/mcp
pnpm build
```

## Usage

Start the MCP server:

```bash
cd tasks/mcp
pnpm start
```

Or run in development mode (builds and starts):

```bash
cd tasks/mcp
pnpm dev
```

## Testing

Run the included test to verify the server works:

```bash
cd tasks/mcp
pnpm test
```

## Available Tools

### Parser Tools

#### parse

Parse JavaScript or TypeScript code using the Oxc parser.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to parse
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"
- `showAst` (boolean, optional): Whether to include the parsed AST in the output, default: false
- `showEstree` (boolean, optional): Whether to include the ESTree representation in the output, default: false
- `showComments` (boolean, optional): Whether to include extracted comments in the output, default: false

#### parser_tsx

Parse TSX files using the Oxc parser.

**Parameters:**

- `sourceCode` (string, required): The TSX source code to parse
- `filename` (string, optional): The filename (should be .tsx), default: "input.tsx"

#### visitor

Demonstrate AST visitor pattern using the Oxc parser.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to visit
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

#### regular_expression

Parse regular expressions using the Oxc parser.

**Parameters:**

- `sourceCode` (string, required): The JavaScript source code containing regular expressions
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Linter Tools

#### linter

Lint JavaScript or TypeScript code using the Oxc linter example.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to lint
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Formatter Tools

#### formatter

Format JavaScript or TypeScript code using the Oxc formatter.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to format
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Semantic Analysis Tools

#### semantic

Perform semantic analysis on JavaScript or TypeScript code.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to analyze
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"
- `showSymbols` (boolean, optional): Whether to display symbol table and reference information, default: false

#### cfg

Generate control flow graph visualizations from JavaScript/TypeScript code.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to analyze
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Transformation Tools

#### transformer

Transform JavaScript or TypeScript code using the Oxc transformer.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to transform
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"
- `targets` (string, optional): Browser/environment targets for transformation

#### define

Replace global defines in JavaScript code using transformer plugins.

**Parameters:**

- `sourceCode` (string, required): The JavaScript source code to transform
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"
- `sourcemap` (boolean, optional): Generate source maps, default: false

### Compilation Tools

#### compiler

Run the complete Oxc compilation pipeline (parse, transform, codegen).

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to compile
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

#### codegen

Generate code from an AST using the Oxc code generator.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to process
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Minification Tools

#### minifier

Minify JavaScript code using the Oxc minifier.

**Parameters:**

- `sourceCode` (string, required): The JavaScript source code to minify
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

#### dce

Perform dead code elimination using the Oxc compressor.

**Parameters:**

- `sourceCode` (string, required): The JavaScript source code to optimize
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"
- `nospace` (boolean, optional): Remove extra whitespace, default: false
- `twice` (boolean, optional): Test idempotency by running twice, default: false

#### mangler

Mangle JavaScript code using the Oxc mangler.

**Parameters:**

- `sourceCode` (string, required): The JavaScript source code to mangle
- `filename` (string, optional): The filename (used to determine file type), default: "input.js"

### Type Processing Tools

#### isolated_declarations

Generate isolated TypeScript declarations.

**Parameters:**

- `sourceCode` (string, required): The TypeScript source code to process
- `filename` (string, optional): The filename (should be .ts or .tsx), default: "input.ts"

### Regular Expression Tools

#### regex_visitor

Demonstrate regular expression AST visitor pattern.

**Parameters:**

- `pattern` (string, required): The regular expression pattern to analyze
- `flags` (string, optional): Regular expression flags (e.g., "gi"), default: ""

#### parse_literal

Parse regular expression literals.

**Parameters:**

- `pattern` (string, required): The regular expression pattern to parse
- `flags` (string, optional): Regular expression flags (e.g., "gi"), default: ""

## Example Usage

Here are some example tool calls:

### Parse and analyze code

```json
{
  "name": "parse",
  "arguments": {
    "sourceCode": "const hello = 'world'; console.log(hello);",
    "filename": "test.js",
    "showAst": true
  }
}
```

### Format code

```json
{
  "name": "formatter",
  "arguments": {
    "sourceCode": "const x=1;if(x){console.log('hello');}"
  }
}
```

### Lint code

```json
{
  "name": "linter",
  "arguments": {
    "sourceCode": "debugger; const {} = {};"
  }
}
```

### Minify code

```json
{
  "name": "minifier",
  "arguments": {
    "sourceCode": "function hello() { return 'world'; }"
  }
}
```

### Transform modern JavaScript

```json
{
  "name": "transformer",
  "arguments": {
    "sourceCode": "const arrow = () => 42;",
    "targets": "es5"
  }
}
```

## Development

The MCP server is structured as follows:

- `src/index.ts` - Main server implementation with all tool definitions
- `src/parser.ts` - Parser utility for the parse tool
- `src/spawn.ts` - Utility for executing cargo commands
- `dist/` - Compiled JavaScript output
- `test/test.mjs` - Test script

The server exposes all Oxc examples as MCP tools by:

1. Writing source code to temporary files
2. Executing the corresponding `cargo run -p <crate> --example <example>` command
3. Returning the output to the client

To add new tools:

1. Add the tool definition to the `ListToolsRequestSchema` handler
2. Add the tool implementation to the `CallToolRequestSchema` handler
3. Update this README with documentation for the new tool

## MCP Integration

This server can be used with any MCP-compatible client. The server communicates over stdio using JSON-RPC messages.

For more information about MCP, see: https://modelcontextprotocol.io/
