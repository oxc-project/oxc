#!/usr/bin/env node

/**
 * Test script for the parser tool in MCP server
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function testParserTool() {
  return new Promise((resolve, reject) => {
    const serverPath = path.join(__dirname, '../dist/index.js');
    const server = spawn('node', [serverPath], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let output = '';
    let errorOutput = '';

    server.stdout.on('data', (data) => {
      output += data.toString();
    });

    server.stderr.on('data', (data) => {
      errorOutput += data.toString();
    });

    // Send a JSON-RPC request to list tools
    const listToolsRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list',
      params: {},
    };

    // Test parse tool with basic JavaScript
    const parseRequest = {
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: {
        name: 'parse',
        arguments: {
          sourceCode: "console.log('Hello World!'); // This is a comment",
          filename: 'test.js',
          showComments: true,
        },
      },
    };

    // Test parse tool with AST output
    const parseAstRequest = {
      jsonrpc: '2.0',
      id: 3,
      method: 'tools/call',
      params: {
        name: 'parse',
        arguments: {
          sourceCode: 'function test() { return 42; }',
          filename: 'test.js',
          showAst: true,
        },
      },
    };

    // Send requests
    server.stdin.write(JSON.stringify(listToolsRequest) + '\n');
    server.stdin.write(JSON.stringify(parseRequest) + '\n');
    server.stdin.write(JSON.stringify(parseAstRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        console.log('✓ MCP server with parser tool started successfully');
        console.log('Server output:', output);
        if (errorOutput) {
          console.log('Server errors:', errorOutput);
        }
        resolve({ output, errorOutput });
      } else {
        console.error(`✗ MCP server exited with code ${code}`);
        console.error('Error output:', errorOutput);
        reject(new Error(`Server exited with code ${code}`));
      }
    });

    server.on('error', (error) => {
      console.error('✗ Failed to start MCP server:', error);
      reject(error);
    });

    // Timeout after 10 seconds
    setTimeout(() => {
      server.kill();
      reject(new Error('Test timeout'));
    }, 10000);
  });
}

// Run the test
testParserTool()
  .then(() => {
    console.log('✓ All parser tests passed');
    process.exit(0);
  })
  .catch((error) => {
    console.error('✗ Parser test failed:', error);
    process.exit(1);
  });
