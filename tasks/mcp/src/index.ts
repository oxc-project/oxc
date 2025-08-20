#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { parseCode } from './parser.js';

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
                additionalArgs: {
                  type: 'array',
                  items: {
                    type: 'string',
                  },
                  description: 'Additional command line arguments to pass to the parser',
                  default: [],
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

      throw new Error(`Unknown tool: ${name}`);
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
