#!/usr/bin/env node

/**
 * Test script to verify custom parser integration with linting
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('Testing Custom Parser + Linting Integration');
console.log('='.repeat(80));

// Test file with intentional linting issues
const testCode = `
const unused = 'not used';
var badVar = 'use const';

function test() {
  debugger;
  console.log('test');
}
`;

console.log('\n📝 Test Code:');
console.log(testCode);

console.log('\n🔍 Expected Violations:');
console.log('  - no-unused-vars: unused variable');
console.log('  - no-var: use of var instead of const/let');
console.log('  - no-debugger: debugger statement');
console.log('  - no-console: console.log statement');

console.log('\n📋 Running oxlint...\n');

// Write test file
import { writeFileSync } from 'fs';
const testFile = join(__dirname, 'temp-test.js');
writeFileSync(testFile, testCode);

// Run oxlint
import { execSync } from 'child_process';
try {
  const result = execSync(
    `node ../../apps/oxlint/dist/cli.js ${testFile} --disable-nested-config`,
    { encoding: 'utf8', cwd: __dirname }
  );
  console.log(result);
} catch (error) {
  console.log(error.stdout);
  console.log(error.stderr);
}

// Clean up
import { unlinkSync } from 'fs';
unlinkSync(testFile);

console.log('\n' + '='.repeat(80));
console.log('✅ Test Complete');
