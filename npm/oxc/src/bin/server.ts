#!/usr/bin/env node

import { McpServer, ResourceTemplate } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

// Create MCP server
const server = new McpServer({
  name: "@oxc-project/oxc",
  version: "0.1.0"
});

// Code Analysis Tool
server.registerTool(
  "analyze-code",
  {
    title: "Analyze JavaScript/TypeScript Code",
    description: "Parse and analyze JavaScript/TypeScript code using Oxc parser",
    inputSchema: {
      code: z.string().describe("JavaScript or TypeScript code to analyze"),
      language: z.enum(["javascript", "typescript"]).optional().describe("Language type (auto-detected if not provided)")
    }
  },
  async ({ code, language }) => {
    // This is a placeholder implementation
    // In a real implementation, this would use the oxc parser
    const analysis = {
      language: language || "auto-detected",
      codeLength: code.length,
      lineCount: code.split('\n').length,
      hasImports: /import\s+/.test(code),
      hasExports: /export\s+/.test(code),
      functions: (code.match(/function\s+\w+/g) || []).length,
      arrows: (code.match(/=>\s*/g) || []).length
    };

    return {
      content: [{
        type: "text",
        text: `Code Analysis Results:\n${JSON.stringify(analysis, null, 2)}`
      }]
    };
  }
);

// Lint Code Tool
server.registerTool(
  "lint-code",
  {
    title: "Lint JavaScript/TypeScript Code",
    description: "Run Oxc linter on JavaScript/TypeScript code",
    inputSchema: {
      code: z.string().describe("JavaScript or TypeScript code to lint"),
      rules: z.array(z.string()).optional().describe("Specific linting rules to apply")
    }
  },
  async ({ code, rules }) => {
    // Placeholder implementation
    // In a real implementation, this would use oxlint
    const issues = [];
    
    // Simple pattern-based checks as examples
    if (code.includes("console.log")) {
      issues.push("Warning: console.log found (no-console)");
    }
    if (code.includes("var ")) {
      issues.push("Warning: 'var' usage found, prefer 'let' or 'const' (no-var)");
    }
    if (/==\s*[^=]/.test(code)) {
      issues.push("Warning: Use '===' instead of '==' (eqeqeq)");
    }

    return {
      content: [{
        type: "text",
        text: issues.length > 0 
          ? `Linting Issues Found:\n${issues.join('\n')}`
          : "No linting issues found!"
      }]
    };
  }
);

// AST Resource
server.registerResource(
  "ast",
  new ResourceTemplate("ast://{type}", { 
    list: undefined 
  }),
  {
    title: "AST Examples",
    description: "Abstract Syntax Tree examples and documentation"
  },
  async (uri, { type }) => {
    const examples = {
      "example": `// AST Example
function greet(name) {
  return "Hello, " + name + "!";
}

// This function would be parsed into an AST with nodes like:
// - FunctionDeclaration
//   - Identifier (name: "greet")
//   - Parameter (Identifier: "name")
//   - BlockStatement
//     - ReturnStatement
//       - BinaryExpression (+)
//         - BinaryExpression (+)
//           - StringLiteral ("Hello, ")
//           - Identifier ("name")
//         - StringLiteral ("!")
`
    };

    const content = examples[type as keyof typeof examples] || "Unknown AST type";
    
    return {
      contents: [{
        uri: uri.href,
        text: content,
        mimeType: "text/plain"
      }]
    };
  }
);

// Documentation Resource
server.registerResource(
  "docs",
  new ResourceTemplate("docs://{section}", { 
    list: undefined 
  }),
  {
    title: "Oxc Documentation",
    description: "Documentation sections for Oxc tools"
  },
  async (uri, { section }) => {
    const docs = {
      "getting-started": `# Getting Started with Oxc

Oxc (The Oxidation Compiler) is a collection of high-performance tools for JavaScript and TypeScript written in Rust.

## Features
- Ultra-fast JavaScript/TypeScript parser
- High-performance linter (oxlint)
- Code formatting capabilities
- AST manipulation tools

## Installation
\`\`\`bash
npm install oxlint
# or
cargo install oxlint
\`\`\`
`,
      "parser": `# Oxc Parser

The Oxc parser is a high-performance JavaScript/TypeScript parser that generates ASTs compatible with ESTree.

## Features
- Full JavaScript support (ES2023+)
- TypeScript support
- JSX support
- High performance (3x faster than alternatives)
- Memory efficient

## Usage
The parser can handle both JavaScript and TypeScript files with automatic detection.
`,
      "linter": `# Oxlint - The Oxc Linter

Oxlint is a fast JavaScript/TypeScript linter that aims to be compatible with ESLint.

## Features
- 50-100x faster than ESLint
- Compatible with ESLint rules
- Zero configuration required
- Supports TypeScript out of the box

## Usage
\`\`\`bash
oxlint src/
\`\`\`
`,
      "ast": `# Abstract Syntax Trees (AST)

Oxc generates ESTree-compatible ASTs that can be used for code analysis and transformation.

## Node Types
- Statements (ExpressionStatement, IfStatement, etc.)
- Expressions (BinaryExpression, CallExpression, etc.)  
- Declarations (FunctionDeclaration, VariableDeclaration, etc.)
- Literals (StringLiteral, NumericLiteral, etc.)

## AST Exploration
Use the analyze-code tool to explore AST structures of your code.
`
    };

    const content = docs[section as keyof typeof docs] || "Documentation section not found";
    
    return {
      contents: [{
        uri: uri.href,
        text: content,
        mimeType: "text/markdown"
      }]
    };
  }
);

// Code Examples Prompt
server.registerPrompt(
  "code-examples",
  {
    title: "Generate Code Examples",
    description: "Generate JavaScript/TypeScript code examples for specific patterns",
    argsSchema: {
      pattern: z.string().describe("Code pattern to generate examples for"),
      language: z.string().describe("Programming language (javascript or typescript)")
    }
  },
  ({ pattern, language }) => ({
    messages: [{
      role: "user",
      content: {
        type: "text",
        text: `Generate ${language} code examples for: ${pattern}\n\nPlease provide clean, well-commented examples that demonstrate best practices.`
      }
    }]
  })
);

// Analysis Prompt
server.registerPrompt(
  "analyze-prompt",
  {
    title: "Code Analysis Prompt",
    description: "Generate prompts for analyzing code with specific focus areas",
    argsSchema: {
      focusArea: z.enum(["performance", "security", "maintainability", "readability"]).describe("Area to focus analysis on"),
      codeType: z.enum(["function", "class", "module", "component"]).describe("Type of code being analyzed")
    }
  },
  ({ focusArea, codeType }) => ({
    messages: [{
      role: "user",
      content: {
        type: "text",
        text: `Please analyze this ${codeType} with a focus on ${focusArea}. 

Consider the following aspects:
${focusArea === "performance" ? "- Time complexity\n- Memory usage\n- Algorithmic efficiency\n- Bottlenecks" : ""}
${focusArea === "security" ? "- Input validation\n- XSS vulnerabilities\n- Injection attacks\n- Data exposure" : ""}
${focusArea === "maintainability" ? "- Code organization\n- Documentation\n- Testing\n- Modularity" : ""}
${focusArea === "readability" ? "- Naming conventions\n- Code clarity\n- Comments\n- Structure" : ""}

Provide specific recommendations for improvement.`
      }
    }]
  })
);

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Oxc MCP Server is running...");
}

main().catch((error) => {
  console.error("Server error:", error);
  process.exit(1);
});