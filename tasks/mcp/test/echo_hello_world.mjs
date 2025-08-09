#!/usr/bin/env node

/**
 * Test script to specifically run the MCP server echo command with "hello world" message
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function runEchoHelloWorld() {
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

    // Send a JSON-RPC request to call echo with "hello world"
    const echoRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: {
        name: 'echo',
        arguments: {
          message: 'hello world',
        },
      },
    };

    console.log('Sending echo request with message: "hello world"');
    server.stdin.write(JSON.stringify(echoRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        console.log('✓ MCP server echo command executed successfully');
        
        // Parse the JSON response
        try {
          const lines = output.trim().split('\n');
          for (const line of lines) {
            if (line.trim()) {
              const response = JSON.parse(line);
              if (response.id === 1 && response.result) {
                const echoedMessage = response.result.content[0].text;
                console.log(`✓ Echo response: "${echoedMessage}"`);
                
                if (echoedMessage === 'hello world') {
                  console.log('✓ Successfully echoed "hello world" message!');
                  resolve({ success: true, message: echoedMessage });
                  return;
                } else {
                  reject(new Error(`Expected "hello world", got "${echoedMessage}"`));
                  return;
                }
              }
            }
          }
          reject(new Error('No valid echo response found'));
        } catch (error) {
          console.error('Error parsing response:', error);
          console.log('Raw output:', output);
          reject(error);
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

    // Timeout after 5 seconds
    setTimeout(() => {
      server.kill();
      reject(new Error('Test timeout'));
    }, 5000);
  });
}

// Run the echo hello world test
console.log('Running MCP server echo command with "hello world" message...');
runEchoHelloWorld()
  .then((result) => {
    console.log('✓ MCP server echo command completed successfully');
    console.log(`✓ Result: ${JSON.stringify(result)}`);
    process.exit(0);
  })
  .catch((error) => {
    console.error('✗ Echo hello world test failed:', error);
    process.exit(1);
  });