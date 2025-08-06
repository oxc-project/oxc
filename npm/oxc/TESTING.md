# Testing the Oxc MCP Server

This guide shows you how to test the Oxc MCP server using the MCP Inspector.

## Prerequisites

1. Install Node.js 18 or later
2. Build the project: `npm run build`

## Using MCP Inspector

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is a debugging tool for MCP servers.

### Installation

```bash
npx @modelcontextprotocol/inspector
```

### Configuration

1. Start the inspector
2. Add a new server configuration:
   - **Server Name**: `oxc`
   - **Command**: `node`
   - **Arguments**: `["/absolute/path/to/npm/oxc/dist/bin/server.js"]`
   - **Environment Variables**: (none required)

### Testing Tools

Once connected, you can test the available tools:

#### analyze-code
```json
{
  "code": "function hello(name) { return `Hello, ${name}!`; }",
  "language": "javascript"
}
```

#### lint-code
```json
{
  "code": "var x = 5; if (x == 5) console.log('hello');",
  "rules": ["no-var", "eqeqeq", "no-console"]
}
```

### Testing Resources

You can browse the available resources:

- `docs://getting-started` - Oxc introduction
- `docs://parser` - Parser documentation
- `docs://linter` - Linter documentation
- `docs://ast` - AST documentation
- `ast://example` - AST examples

### Testing Prompts

Try the available prompts:

#### code-examples
```json
{
  "pattern": "async function",
  "language": "typescript"
}
```

#### analyze-prompt
```json
{
  "focusArea": "performance",
  "codeType": "function"
}
```

## Manual Testing

You can also test the server manually using stdio:

```bash
# Start the server
node dist/bin/server.js

# Send an initialize request (paste this and press Enter)
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}

# List available tools
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}

# Call the analyze-code tool
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"analyze-code","arguments":{"code":"const x = 42;","language":"javascript"}}}
```

## Expected Responses

The server should respond with appropriate JSON-RPC 2.0 messages. For example, listing tools should return the `analyze-code` and `lint-code` tools with their schemas.