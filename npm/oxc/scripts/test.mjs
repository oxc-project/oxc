#!/usr/bin/env node

// Simple test to validate the MCP server basic functionality
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

function testMcpServer() {
  return new Promise((resolve, reject) => {
    const serverPath = join(__dirname, '..', 'dist', 'bin', 'server.js');
    console.log(`Starting server: ${serverPath}`);
    
    const server = spawn('node', [serverPath], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let output = '';
    let errorOutput = '';

    server.stdout.on('data', (data) => {
      output += data.toString();
    });

    server.stderr.on('data', (data) => {
      errorOutput += data.toString();
    });

    // Send a simple initialize request
    const initRequest = {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: "2024-11-05",
        capabilities: {},
        clientInfo: {
          name: "test-client",
          version: "1.0.0"
        }
      }
    };

    setTimeout(() => {
      server.stdin.write(JSON.stringify(initRequest) + '\n');
    }, 500);

    setTimeout(() => {
      server.kill();
      if (errorOutput.includes('Oxc MCP Server is running')) {
        console.log('✅ MCP Server started successfully');
        console.log('✅ Server logged startup message correctly');
        resolve(true);
      } else {
        console.log('❌ MCP Server failed to start properly');
        console.log('Error output:', errorOutput);
        console.log('Standard output:', output);
        reject(new Error('Server failed to start'));
      }
    }, 2000);

    server.on('error', (err) => {
      console.log('❌ Server process error:', err);
      reject(err);
    });
  });
}

console.log('🧪 Testing Oxc MCP Server...');
console.log('');
testMcpServer()
  .then(() => {
    console.log('');
    console.log('✅ All tests passed!');
    console.log('');
    console.log('💡 To test interactively, run:');
    console.log('   npm start');
    console.log('');
    console.log('💡 To use with MCP Inspector:');
    console.log('   npx @modelcontextprotocol/inspector');
    process.exit(0);
  })
  .catch((err) => {
    console.error('❌ Tests failed:', err.message);
    process.exit(1);
  });