#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { spawn } from 'child_process';
import { writeFileSync, unlinkSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

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
          // Create a temporary file with the source code
          const tmpFile = join(tmpdir(), `oxc-mcp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}-${filename}`);
          writeFileSync(tmpFile, sourceCode, 'utf8');

          try {
            // Build the cargo command arguments
            const cargoArgs = ['run', '-p', 'oxc_parser', '--example', 'parser', tmpFile];
            
            if (showAst) {
              cargoArgs.push('--ast');
            }
            if (showEstree) {
              cargoArgs.push('--estree');
            }
            if (showComments) {
              cargoArgs.push('--comments');
            }

            // Spawn the cargo command
            const result = await this.spawnCommand('cargo', cargoArgs);
            
            return {
              content: [
                {
                  type: 'text',
                  text: result,
                },
              ],
            };
          } finally {
            // Clean up temporary file
            try {
              unlinkSync(tmpFile);
            } catch (e) {
              // Ignore cleanup errors
            }
          }
        } catch (error) {
          throw new Error(`Parse error: ${error instanceof Error ? error.message : String(error)}`);
        }
      }

      throw new Error(`Unknown tool: ${name}`);
    });
  }

  private spawnCommand(command: string, args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      const process = spawn(command, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        cwd: '/home/runner/work/oxc/oxc',  // Set working directory to the oxc repository root
      });

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      process.on('close', (code) => {
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(`Command failed with exit code ${code}:\n${stderr}`));
        }
      });

      process.on('error', (error) => {
        reject(new Error(`Failed to spawn command: ${error.message}`));
      });
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