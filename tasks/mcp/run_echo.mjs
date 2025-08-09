#!/usr/bin/env node

/**
 * Simple CLI script to run the MCP server echo command
 * Usage: node run_echo.mjs [message]
 * Example: node run_echo.mjs "hello world"
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Get message from command line arguments or use default
const message = process.argv[2] || 'hello world';

function runEcho(message) {
  return new Promise((resolve, reject) => {
    const serverPath = path.join(__dirname, 'dist/index.js');
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

    // Send a JSON-RPC request to call echo
    const echoRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: {
        name: 'echo',
        arguments: {
          message: message,
        },
      },
    };

    server.stdin.write(JSON.stringify(echoRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        try {
          const lines = output.trim().split('\n');
          for (const line of lines) {
            if (line.trim()) {
              const response = JSON.parse(line);
              if (response.id === 1 && response.result) {
                const echoedMessage = response.result.content[0].text;
                resolve(echoedMessage);
                return;
              }
            }
          }
          reject(new Error('No valid echo response found'));
        } catch (error) {
          reject(error);
        }
      } else {
        reject(new Error(`Server exited with code ${code}: ${errorOutput}`));
      }
    });

    server.on('error', (error) => {
      reject(error);
    });

    setTimeout(() => {
      server.kill();
      reject(new Error('Timeout'));
    }, 5000);
  });
}

console.log(`Running MCP echo command with message: "${message}"`);
runEcho(message)
  .then((result) => {
    console.log(`Echo result: "${result}"`);
  })
  .catch((error) => {
    console.error('Error:', error.message);
    process.exit(1);
  });