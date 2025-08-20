#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

import { parseCode } from './parser.js';
import { spawnCommand } from './spawn.js';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

/**
 * Write source code to a temporary file and return the path
 */
function writeTempFile(sourceCode: string, filename: string): string {
  // Create temp directory if it doesn't exist
  mkdirSync('/tmp/oxc-mcp', { recursive: true });
  
  const tempPath = join('/tmp/oxc-mcp', filename);
  writeFileSync(tempPath, sourceCode, 'utf8');
  return tempPath;
}

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

      // Handle example-based tools that require writing to temp files
      const sourceCode = args?.sourceCode as string;
      const filename = (args?.filename as string) || 'input.js';
      
      if (!sourceCode) {
        throw new Error('sourceCode is required');
      }

      const tempFilePath = writeTempFile(sourceCode, filename);

      try {
        let result: string;

        switch (name) {
          case 'linter':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_linter', '--example', 'linter', tempFilePath]);
            break;

          case 'formatter':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_formatter', '--example', 'formatter', tempFilePath]);
            break;

          case 'semantic':
            const showSymbols = args?.showSymbols as boolean;
            const semanticArgs = ['run', '-p', 'oxc_semantic', '--example', 'semantic', tempFilePath];
            if (showSymbols) {
              semanticArgs.push('--symbols');
            }
            result = await spawnCommand('cargo', semanticArgs);
            break;

          case 'transformer':
            const targets = args?.targets as string;
            const transformerArgs = ['run', '-p', 'oxc_transformer', '--example', 'transformer', tempFilePath];
            if (targets) {
              transformerArgs.push('--targets', targets);
            }
            result = await spawnCommand('cargo', transformerArgs);
            break;

          case 'compiler':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc', '--example', 'compiler', '--features=full', tempFilePath]);
            break;

          case 'codegen':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_codegen', '--example', 'codegen', tempFilePath]);
            break;

          case 'minifier':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_minifier', '--example', 'minifier', tempFilePath]);
            break;

          case 'dce':
            const nospace = args?.nospace as boolean;
            const twice = args?.twice as boolean;
            const dceArgs = ['run', '-p', 'oxc_minifier', '--example', 'dce', tempFilePath];
            if (nospace) dceArgs.push('--nospace');
            if (twice) dceArgs.push('--twice');
            result = await spawnCommand('cargo', dceArgs);
            break;

          case 'mangler':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_minifier', '--example', 'mangler', tempFilePath]);
            break;

          case 'cfg':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_semantic', '--example', 'cfg', tempFilePath]);
            break;

          case 'isolated_declarations':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_isolated_declarations', '--example', 'isolated_declarations', tempFilePath]);
            break;

          case 'define':
            const sourcemap = args?.sourcemap as boolean;
            const defineArgs = ['run', '-p', 'oxc_transformer_plugins', '--example', 'define', tempFilePath];
            if (sourcemap) defineArgs.push('--sourcemap');
            result = await spawnCommand('cargo', defineArgs);
            break;

          case 'visitor':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_parser', '--example', 'visitor', tempFilePath]);
            break;

          case 'parser_tsx':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_parser', '--example', 'parser_tsx', tempFilePath]);
            break;

          case 'regular_expression':
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_parser', '--example', 'regular_expression', tempFilePath]);
            break;

          case 'regex_visitor':
            const pattern = args?.pattern as string;
            const flags = (args?.flags as string) || '';
            if (!pattern) {
              throw new Error('pattern is required for regex_visitor');
            }
            // Create a temp file with the regex pattern
            const regexContent = `/${pattern}/${flags}`;
            const regexTempPath = writeTempFile(regexContent, 'regex.txt');
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_regular_expression', '--example', 'regex_visitor', regexTempPath]);
            break;

          case 'parse_literal':
            const literalPattern = args?.pattern as string;
            const literalFlags = (args?.flags as string) || '';
            if (!literalPattern) {
              throw new Error('pattern is required for parse_literal');
            }
            // Create a temp file with the regex pattern
            const literalContent = `/${literalPattern}/${literalFlags}`;
            const literalTempPath = writeTempFile(literalContent, 'regex.txt');
            result = await spawnCommand('cargo', ['run', '-p', 'oxc_regular_expression', '--example', 'parse_literal', literalTempPath]);
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
