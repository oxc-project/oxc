#!/usr/bin/env node

/**
 * More comprehensive test script for the parser tool
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function testMcpParser(testName, request) {
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

    server.stdin.write(JSON.stringify(request) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        try {
          const lines = output.trim().split('\n');
          const lastLine = lines[lines.length - 1];
          const response = JSON.parse(lastLine);
          console.log(`✓ ${testName}: Success`);
          console.log(`  Response: ${response.result.content[0].text.slice(0, 100)}...`);
          resolve(response);
        } catch (error) {
          console.error(`✗ ${testName}: Failed to parse response`);
          reject(error);
        }
      } else {
        console.error(`✗ ${testName}: Server exited with code ${code}`);
        reject(new Error(`Server exited with code ${code}`));
      }
    });

    server.on('error', (error) => {
      console.error(`✗ ${testName}: Failed to start server`);
      reject(error);
    });

    setTimeout(() => {
      server.kill();
      reject(new Error(`${testName}: Test timeout`));
    }, 5000);
  });
}

async function runTests() {
  console.log('Running comprehensive parser tests...\n');

  // Test 1: Parse TypeScript with types
  await testMcpParser('TypeScript parsing', {
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'parse',
      arguments: {
        sourceCode: 'interface User { name: string; age: number; } const user: User = { name: "John", age: 30 };',
        filename: 'test.ts',
      },
    },
  });

  // Test 2: Parse JSX
  await testMcpParser('JSX parsing', {
    jsonrpc: '2.0',
    id: 2,
    method: 'tools/call',
    params: {
      name: 'parse',
      arguments: {
        sourceCode: 'const element = <div className="container"><h1>Hello React!</h1></div>;',
        filename: 'test.jsx',
      },
    },
  });

  // Test 3: Parse with syntax error
  await testMcpParser('Syntax error handling', {
    jsonrpc: '2.0',
    id: 3,
    method: 'tools/call',
    params: {
      name: 'parse',
      arguments: {
        sourceCode: 'const x = {; // Intentional syntax error',
        filename: 'test.js',
      },
    },
  });

  // Test 4: Parse with comments
  await testMcpParser('Comments extraction', {
    jsonrpc: '2.0',
    id: 4,
    method: 'tools/call',
    params: {
      name: 'parse',
      arguments: {
        sourceCode: '/* Block comment */ const x = 5; // Line comment',
        filename: 'test.js',
        showComments: true,
      },
    },
  });

  console.log('\n✓ All comprehensive tests passed!');
}

runTests().catch((error) => {
  console.error('✗ Tests failed:', error);
  process.exit(1);
});
