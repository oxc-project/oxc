#!/usr/bin/env node

/**
 * Test script to specifically test echoing "hello" as requested
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function testEchoHello() {
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

    // Send a JSON-RPC request to echo "hello"
    const echoHelloRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: {
        name: 'echo',
        arguments: {
          message: 'hello',
        },
      },
    };

    // Send request
    server.stdin.write(JSON.stringify(echoHelloRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        console.log('✓ MCP server echoed "hello" successfully');
        
        // Parse the response to verify it contains "hello"
        try {
          const lines = output.trim().split('\n');
          const response = JSON.parse(lines[0]);
          
          if (response.result?.content?.[0]?.text === 'hello') {
            console.log('✓ Echo response correctly returned "hello"');
            console.log('Response:', JSON.stringify(response, null, 2));
            resolve({ output, errorOutput, response });
          } else {
            console.error('✗ Echo response did not return "hello"');
            console.error('Got:', response.result?.content?.[0]?.text);
            reject(new Error('Incorrect echo response'));
          }
        } catch (parseError) {
          console.error('✗ Failed to parse server response');
          console.error('Raw output:', output);
          reject(parseError);
        }
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
testEchoHello()
  .then(() => {
    console.log('✓ "echo hello" test passed successfully');
    process.exit(0);
  })
  .catch((error) => {
    console.error('✗ "echo hello" test failed:', error);
    process.exit(1);
  });