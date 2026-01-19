/**
 * Check if V8 is using external strings or copying
 */
import { transformSyncFast } from '../index.js';
import v8 from 'v8';

// Get heap stats before and after creating strings
function getHeapUsed() {
  if (global.gc) global.gc();
  return v8.getHeapStatistics().used_heap_size;
}

const sizes = [1000, 10000, 100000, 500000];

console.log('Checking if V8 uses external strings or copies...\n');

for (const size of sizes) {
  const code = 'const x = "' + 'a'.repeat(size) + '";';

  // Warmup
  for (let i = 0; i < 10; i++) {
    const result = transformSyncFast('t.ts', code);
  }

  // Measure heap growth
  const heapBefore = getHeapUsed();

  const results = [];
  for (let i = 0; i < 100; i++) {
    results.push(transformSyncFast('t.ts', code));
  }

  const heapAfter = getHeapUsed();
  const heapGrowth = heapAfter - heapBefore;
  const expectedGrowth = size * 100; // If copying, expect ~size * count bytes

  const outputSize = results[0].code.length;

  console.log(`${(size/1000).toString().padStart(3)}KB input → ${(outputSize/1000).toFixed(0)}KB output`);
  console.log(`  Heap growth: ${(heapGrowth/1024/1024).toFixed(2)}MB for 100 strings`);
  console.log(`  Expected if copying: ${(expectedGrowth/1024/1024).toFixed(2)}MB`);
  console.log(`  Ratio: ${(heapGrowth/expectedGrowth).toFixed(2)}x (< 0.5 suggests external strings working)`);
  console.log('');
}

// Also check if strings are actually usable
console.log('Verifying string contents are correct...');
const testCode = 'const x: number = 42;';
const result = transformSyncFast('t.ts', testCode);
console.log(`Input:  "${testCode}"`);
console.log(`Output: "${result.code}"`);
console.log(`Match: ${result.code.includes('const x = 42') ? '✅' : '❌'}`);
