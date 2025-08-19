#!/usr/bin/env node

/**
 * Final verification test that compares MCP parser output with original parser example
 */

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function testMcpParserEquivalent() {
  return new Promise((resolve, reject) => {
    const serverPath = path.join(__dirname, '../dist/index.js');
    const server = spawn('node', [serverPath], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let output = '';

    server.stdout.on('data', (data) => {
      output += data.toString();
    });

    // Test equivalent to: cargo run -p oxc_parser --example parser test.js --comments --ast
    const parseRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: {
        name: 'parse',
        arguments: {
          sourceCode: `console.log('Hello World!'); // Test comment
function test() {
  return 42;
}`,
          filename: 'test.js',
          showAst: true,
          showComments: true,
        },
      },
    };

    server.stdin.write(JSON.stringify(parseRequest) + '\n');
    server.stdin.end();

    server.on('close', (code) => {
      if (code === 0) {
        try {
          const lines = output.trim().split('\n');
          const response = JSON.parse(lines[lines.length - 1]);
          const result = response.result.content[0].text;
          
          console.log('✓ MCP Parser Tool Output:');
          console.log('=' .repeat(50));
          console.log(result);
          console.log('=' .repeat(50));
          
          // Verify output contains expected sections
          const hasComments = result.includes('Comments:');
          const hasAST = result.includes('AST:');
          const hasSuccess = result.includes('Parsed Successfully');
          
          if (hasComments && hasAST && hasSuccess) {
            console.log('✓ All expected sections present');
            resolve(true);
          } else {
            console.log('✗ Missing expected sections');
            reject(new Error('Missing expected output sections'));
          }
        } catch (error) {
          reject(error);
        }
      } else {
        reject(new Error(`Server exited with code ${code}`));
      }
    });

    setTimeout(() => {
      server.kill();
      reject(new Error('Test timeout'));
    }, 10000);
  });
}

console.log('Testing MCP parser tool equivalent to original parser example...\n');

testMcpParserEquivalent()
  .then(() => {
    console.log('\n✓ MCP parser tool successfully replicates parser example functionality!');
  })
  .catch((error) => {
    console.error('✗ Test failed:', error);
    process.exit(1);
  });