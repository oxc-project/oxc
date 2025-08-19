#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { parseSync } from 'oxc-parser';

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
            name: 'echo',
            description: 'Echo back the provided message',
            inputSchema: {
              type: 'object',
              properties: {
                message: {
                  type: 'string',
                  description: 'The message to echo back',
                },
              },
              required: ['message'],
            },
          },
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
        ],
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      if (name === 'echo') {
        const message = args?.message;
        if (typeof message !== 'string') {
          throw new Error('Message must be a string');
        }

        return {
          content: [
            {
              type: 'text',
              text: message,
            },
          ],
        };
      }

      if (name === 'parse') {
        const sourceCode = args?.sourceCode as string;
        const filename = (args?.filename as string) || 'input.js';
        const showAst = (args?.showAst as boolean) || false;
        const showEstree = (args?.showEstree as boolean) || false;
        const showComments = (args?.showComments as boolean) || false;

        if (typeof sourceCode !== 'string') {
          throw new Error('sourceCode must be a string');
        }

        try {
          // Parse the source code using Oxc parser
          const result = parseSync(filename, sourceCode);
          const output: string[] = [];

          // Display comments if requested
          if (showComments) {
            output.push('Comments:');
            if (result.comments.length === 0) {
              output.push('  (no comments found)');
            } else {
              for (const comment of result.comments) {
                output.push(`  ${comment.type}: ${comment.value}`);
              }
            }
            output.push('');
          }

          // Display AST if requested
          if (showAst) {
            output.push('AST:');
            output.push(JSON.stringify(result.program, null, 2));
            output.push('');
          }

          // Display ESTree representation if requested
          if (showEstree) {
            // Get the ESTree representation
            // Note: The NAPI bindings already return ESTree-compatible format
            output.push('ESTree AST:');
            output.push(JSON.stringify(result.program, null, 2));
            output.push('');
          }

          // Report parsing results
          if (result.errors.length === 0) {
            output.push('Parsed Successfully.');
          } else {
            output.push('Parsing Errors:');
            for (const error of result.errors) {
              output.push(`  ${error.severity}: ${error.message}`);
              if (error.labels && error.labels.length > 0) {
                for (const label of error.labels) {
                  output.push(`    at ${label.start}-${label.end}: ${label.message || 'error'}`);
                }
              }
            }
            output.push('Parsed with Errors.');
          }

          return {
            content: [
              {
                type: 'text',
                text: output.join('\n'),
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
