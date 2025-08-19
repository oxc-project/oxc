# MCP Boilerplate for Oxc

This is a Model Context Protocol (MCP) server boilerplate for the oxc project. It provides a simple echo command as an example.

## Features

- TypeScript-based MCP server
- Single `echo` command that echoes back any message
- **New:** `parse` command that parses JavaScript/TypeScript code using the Oxc parser
- Built using @modelcontextprotocol/sdk

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

### echo

Echoes back the provided message.

**Parameters:**

- `message` (string, required): The message to echo back

**Example:**

```json
{
  "name": "echo",
  "arguments": {
    "message": "Hello, MCP!"
  }
}
```

**Response:**

```json
{
  "content": [
    {
      "type": "text",
      "text": "Hello, MCP!"
    }
  ]
}
```

### parse

Parses JavaScript or TypeScript code using the Oxc parser and displays the results.

**Parameters:**

- `sourceCode` (string, required): The JavaScript or TypeScript source code to parse
- `filename` (string, optional): The filename (used to determine file type). Defaults to "input.js"
- `showAst` (boolean, optional): Whether to include the parsed AST in the output. Defaults to false
- `showEstree` (boolean, optional): Whether to include the ESTree representation in the output. Defaults to false
- `showComments` (boolean, optional): Whether to include extracted comments in the output. Defaults to false

**Example:**

```json
{
  "name": "parse",
  "arguments": {
    "sourceCode": "console.log('Hello World!'); // Comment",
    "filename": "test.js",
    "showComments": true
  }
}
```

**Response:**

```json
{
  "content": [
    {
      "type": "text",
      "text": "Comments:\n  Line:  Comment\n\nParsed Successfully."
    }
  ]
}
```

## Development

The MCP server is structured as follows:

- `src/index.ts` - Main server implementation
- `dist/` - Compiled JavaScript output
- `test/test.mjs` - Simple test script

To add new tools:

1. Add the tool definition to the `ListToolsRequestSchema` handler
2. Add the tool implementation to the `CallToolRequestSchema` handler
3. Update this README with documentation for the new tool

The `parse` tool demonstrates how to integrate Oxc's parser functionality into the MCP server, allowing clients to parse JavaScript and TypeScript code and inspect the resulting AST, comments, and parsing errors.

## MCP Integration

This server can be used with any MCP-compatible client. The server communicates over stdio using JSON-RPC messages.

For more information about MCP, see: https://modelcontextprotocol.io/
