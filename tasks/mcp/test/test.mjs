#!/usr/bin/env node

/**
 * Simple test script to verify the MCP server works
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function testMcpServer() {
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

    const echoRequest = {
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: {
        name: 'echo',
        arguments: {
          message: 'Hello, MCP!',
        },
      },
    };

    // Send requests
    server.stdin.write(JSON.stringify(listToolsRequest) + '\n');
    server.stdin.write(JSON.stringify(echoRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        console.log('✓ MCP server started successfully');
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
testMcpServer()
  .then(() => {
    console.log('✓ All tests passed');
    process.exit(0);
  })
  .catch((error) => {
    console.error('✗ Test failed:', error);
    process.exit(1);
  });
