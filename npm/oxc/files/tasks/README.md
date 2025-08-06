# Oxc MCP Server Tasks

This directory contains task definitions and configurations for the Oxc MCP (Model Context Protocol) server.

## Available Tasks

### Code Analysis Tasks
- **parse-code**: Parse JavaScript/TypeScript code into AST
- **analyze-syntax**: Analyze code syntax and structure
- **detect-patterns**: Detect common code patterns and anti-patterns

### Linting Tasks  
- **lint-file**: Lint a single JavaScript/TypeScript file
- **lint-project**: Lint an entire project directory
- **check-rules**: Validate code against specific linting rules

### Documentation Tasks
- **generate-docs**: Generate documentation from code comments
- **extract-types**: Extract TypeScript type definitions
- **analyze-exports**: Analyze module exports and dependencies

## Task Configuration

Tasks can be configured through the MCP server interface or by modifying the task definitions in this directory.

### Example Task Configuration

```json
{
  "taskName": "analyze-code",
  "description": "Analyze JavaScript/TypeScript code",
  "inputs": {
    "code": "string",
    "language": "javascript|typescript"
  },
  "outputs": {
    "analysis": "CodeAnalysis",
    "warnings": "string[]"
  }
}
```

## Usage

These tasks are available through the MCP server's tool interface. They can be called by compatible MCP clients such as:

- Claude Desktop
- MCP Inspector  
- Custom MCP client applications

## Development

To add new tasks:

1. Define the task schema in a JSON file
2. Implement the task handler in the server code
3. Register the task with the MCP server
4. Test the task using the MCP Inspector