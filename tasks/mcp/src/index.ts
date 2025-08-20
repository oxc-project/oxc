#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

import {
  analyzeCode,
  compileCode,
  defineCode,
  eliminateDeadCode,
  formatCode,
  generateCFG,
  generateCode,
  generateIsolatedDeclarations,
  lintCode,
  mangleCode,
  minifyCode,
  parseCode,
  parseLiteral,
  parseRegularExpressions,
  parseTSXCode,
  transformCode,
  visitCode,
  visitRegex,
} from './tools.js';



/**
 * MCP server for oxc project with echo command
 */

class McpOxcServer {
  private server: Server;

  constructor() {
    this.server = new Server(
      {
        name: 'mcp-oxc',
        version: '0.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      },
    );

    this.setupToolHandlers();
  }

  private setupToolHandlers() {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: [
          {
            name: 'parse',
            description: 'Parse JavaScript or TypeScript code using the Oxc parser',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to parse',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
                showAst: {
                  type: 'boolean',
                  description: 'Whether to include the parsed AST in the output',
                  default: false,
                },
                showEstree: {
                  type: 'boolean',
                  description: 'Whether to include the ESTree representation in the output',
                  default: false,
                },
                showComments: {
                  type: 'boolean',
                  description: 'Whether to include extracted comments in the output',
                  default: false,
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'linter',
            description: 'Lint JavaScript or TypeScript code using the Oxc linter example',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to lint',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'formatter',
            description: 'Format JavaScript or TypeScript code using the Oxc formatter',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to format',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'semantic',
            description: 'Perform semantic analysis on JavaScript or TypeScript code',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to analyze',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
                showSymbols: {
                  type: 'boolean',
                  description: 'Whether to display symbol table and reference information',
                  default: false,
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'transformer',
            description: 'Transform JavaScript or TypeScript code using the Oxc transformer',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to transform',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
                targets: {
                  type: 'string',
                  description: 'Browser/environment targets for transformation',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'compiler',
            description: 'Run the complete Oxc compilation pipeline (parse, transform, codegen)',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to compile',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'codegen',
            description: 'Generate code from an AST using the Oxc code generator',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to process',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'minifier',
            description: 'Minify JavaScript code using the Oxc minifier',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript source code to minify',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'dce',
            description: 'Perform dead code elimination using the Oxc compressor',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript source code to optimize',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
                nospace: {
                  type: 'boolean',
                  description: 'Remove extra whitespace',
                  default: false,
                },
                twice: {
                  type: 'boolean',
                  description: 'Test idempotency by running twice',
                  default: false,
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'mangler',
            description: 'Mangle JavaScript code using the Oxc mangler',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript source code to mangle',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'cfg',
            description: 'Generate control flow graph visualizations from JavaScript/TypeScript code',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to analyze',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'isolated_declarations',
            description: 'Generate isolated TypeScript declarations',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The TypeScript source code to process',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (should be .ts or .tsx)',
                  default: 'input.ts',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'define',
            description: 'Replace global defines in JavaScript code using transformer plugins',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript source code to transform',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
                sourcemap: {
                  type: 'boolean',
                  description: 'Generate source maps',
                  default: false,
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'visitor',
            description: 'Demonstrate AST visitor pattern using the Oxc parser',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript or TypeScript source code to visit',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'parser_tsx',
            description: 'Parse TSX files using the Oxc parser',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The TSX source code to parse',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (should be .tsx)',
                  default: 'input.tsx',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'regular_expression',
            description: 'Parse regular expressions using the Oxc parser',
            inputSchema: {
              type: 'object',
              properties: {
                sourceCode: {
                  type: 'string',
                  description: 'The JavaScript source code containing regular expressions',
                },
                filename: {
                  type: 'string',
                  description: 'The filename (used to determine file type)',
                  default: 'input.js',
                },
              },
              required: ['sourceCode'],
            },
          },
          {
            name: 'regex_visitor',
            description: 'Demonstrate regular expression AST visitor pattern',
            inputSchema: {
              type: 'object',
              properties: {
                pattern: {
                  type: 'string',
                  description: 'The regular expression pattern to analyze',
                },
                flags: {
                  type: 'string',
                  description: 'Regular expression flags (e.g., "gi")',
                  default: '',
                },
              },
              required: ['pattern'],
            },
          },
          {
            name: 'parse_literal',
            description: 'Parse regular expression literals',
            inputSchema: {
              type: 'object',
              properties: {
                pattern: {
                  type: 'string',
                  description: 'The regular expression pattern to parse',
                },
                flags: {
                  type: 'string',
                  description: 'Regular expression flags (e.g., "gi")',
                  default: '',
                },
              },
              required: ['pattern'],
            },
          },
        ],
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      if (name === 'parse') {
        const sourceCode = args?.sourceCode as string;
        const filename = (args?.filename as string) || 'input.js';
        const showAst = (args?.showAst as boolean) || false;
        const showEstree = (args?.showEstree as boolean) || false;
        const showComments = (args?.showComments as boolean) || false;

        try {
          const result = await parseCode({
            sourceCode,
            filename,
            showAst,
            showEstree,
            showComments,
          });

          return {
            content: [
              {
                type: 'text',
                text: result,
              },
            ],
          };
        } catch (error) {
          throw new Error(`Parse error: ${error instanceof Error ? error.message : String(error)}`);
        }
      }

      // Handle example-based tools
      const sourceCode = args?.sourceCode as string;
      const filename = (args?.filename as string) || 'input.js';

      try {
        let result: string;

        switch (name) {
          case 'linter':
            result = await lintCode({ sourceCode, filename });
            break;

          case 'formatter':
            result = await formatCode({ sourceCode, filename });
            break;

          case 'semantic':
            const showSymbols = args?.showSymbols as boolean;
            result = await analyzeCode({ sourceCode, filename, showSymbols });
            break;

          case 'transformer':
            const targets = args?.targets as string;
            result = await transformCode({ sourceCode, filename, targets });
            break;

          case 'compiler':
            result = await compileCode({ sourceCode, filename });
            break;

          case 'codegen':
            result = await generateCode({ sourceCode, filename });
            break;

          case 'minifier':
            result = await minifyCode({ sourceCode, filename });
            break;

          case 'dce':
            const nospace = args?.nospace as boolean;
            const twice = args?.twice as boolean;
            result = await eliminateDeadCode({ sourceCode, filename, nospace, twice });
            break;

          case 'mangler':
            result = await mangleCode({ sourceCode, filename });
            break;

          case 'cfg':
            result = await generateCFG({ sourceCode, filename });
            break;

          case 'isolated_declarations':
            result = await generateIsolatedDeclarations({ sourceCode, filename });
            break;

          case 'define':
            const sourcemap = args?.sourcemap as boolean;
            result = await defineCode({ sourceCode, filename, sourcemap });
            break;

          case 'visitor':
            result = await visitCode({ sourceCode, filename });
            break;

          case 'parser_tsx':
            result = await parseTSXCode({ sourceCode, filename });
            break;

          case 'regular_expression':
            result = await parseRegularExpressions({ sourceCode, filename });
            break;

          case 'regex_visitor':
            const pattern = args?.pattern as string;
            const flags = (args?.flags as string) || '';
            if (!pattern) {
              throw new Error('pattern is required for regex_visitor');
            }
            result = await visitRegex({ pattern, flags });
            break;

          case 'parse_literal':
            const literalPattern = args?.pattern as string;
            const literalFlags = (args?.flags as string) || '';
            if (!literalPattern) {
              throw new Error('pattern is required for parse_literal');
            }
            result = await parseLiteral({ pattern: literalPattern, flags: literalFlags });
            break;

          default:
            throw new Error(`Unknown tool: ${name}`);
        }

        return {
          content: [
            {
              type: 'text',
              text: result,
            },
          ],
        };
      } catch (error) {
        throw new Error(`${name} error: ${error instanceof Error ? error.message : String(error)}`);
      }
    });
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
  }
}

// Start the server
const server = new McpOxcServer();
server.run().catch((error) => {
  console.error('Server error:', error);
  process.exit(1);
});
