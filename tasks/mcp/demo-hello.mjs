#!/usr/bin/env node

/**
 * Simple demonstration of echoing "hello" using the MCP server
 * This demonstrates the solution to the problem statement: echo 'hello'
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log("🔧 Starting MCP server to demonstrate 'echo hello'...");

const serverPath = path.join(__dirname, 'dist/index.js');
const server = spawn('node', [serverPath], {
  stdio: ['pipe', 'pipe', 'pipe'],
});

let output = '';

server.stdout.on('data', (data) => {
  output += data.toString();
});

server.stderr.on('data', (data) => {
  console.error('Server error:', data.toString());
});

// Send the echo "hello" request
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

console.log('📤 Sending request to echo "hello"...');
console.log('Request:', JSON.stringify(echoHelloRequest, null, 2));

server.stdin.write(JSON.stringify(echoHelloRequest) + '\n');
server.stdin.end();

server.on('close', (code) => {
  if (code === 0) {
    console.log('\n📥 Server response:');
    try {
      const lines = output.trim().split('\n');
      const response = JSON.parse(lines[0]);
      console.log(JSON.stringify(response, null, 2));
      
      const echoedText = response.result?.content?.[0]?.text;
      if (echoedText === 'hello') {
        console.log('\n✅ SUCCESS: Successfully echoed "hello"!');
        console.log(`💬 Echo result: "${echoedText}"`);
      } else {
        console.log(`❌ UNEXPECTED: Got "${echoedText}" instead of "hello"`);
      }
    } catch (error) {
      console.error('❌ Failed to parse response:', error);
      console.log('Raw output:', output);
    }
  } else {
    console.error(`❌ Server exited with code ${code}`);
  }
});

server.on('error', (error) => {
  console.error('❌ Failed to start server:', error);
});