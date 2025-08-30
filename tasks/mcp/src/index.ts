#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

import { minify } from './minifier.js';
import { parse } from './parser.js';

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
            name: 'minify',
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
                mangle: {
                  type: 'boolean',
                  description: 'Mangle names',
                  default: false,
                },
                nospace: {
                  type: 'boolean',
                  description: 'Remove whitespace',
                  default: false,
                },
              },
              required: ['sourceCode'],
            },
          },
        ],
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      const sourceCode = args?.sourceCode as string;
      const filename = (args?.filename as string) || 'input.js';
      const options = {
        sourceCode,
        filename,
        ...args,
      };

      let fn;
      switch (name) {
        case 'parse': {
          fn = parse;
          break;
        }
        case 'minify': {
          fn = minify;
          break;
        }
        default:
          throw new Error(`Unknown tool: ${name}`);
      }

      let result;
      try {
        result = await fn(options);
      } catch (error) {
        throw new Error(`${name} error: ${error instanceof Error ? error.message : String(error)}`);
      }

      return {
        content: [
          {
            type: 'text',
            text: result,
          },
        ],
      };
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
