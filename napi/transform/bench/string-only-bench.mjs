/**
 * Benchmark ONLY string creation overhead, not transformation
 * This measures the difference between returning String vs FastString
 */
import { transformSync, transformSyncFast } from '../index.js';

const ITERATIONS = 1000;
const WARMUP = 100;

// Small code that transforms quickly - focus is on string return overhead
const smallCode = 'const x: number = 1;';
const mediumCode = `const x: string = "${'a'.repeat(1000)}";`;
const largeCode = `const x: string = "${'a'.repeat(100000)}";`;

function benchmark(name, fn) {
  // Warmup
  for (let i = 0; i < WARMUP; i++) fn();

  const times = [];
  for (let i = 0; i < ITERATIONS; i++) {
    const start = performance.now();
    fn();
    times.push(performance.now() - start);
  }

  const sorted = [...times].sort((a, b) => a - b);
  const mean = times.reduce((a, b) => a + b) / times.length;
  const p50 = sorted[Math.floor(times.length * 0.5)];

  return { mean, p50, name };
}

console.log('String Creation Overhead Benchmark');
console.log('='.repeat(80));
console.log('Measuring time to transform AND return string to JS\n');

// First verify output sizes
const smallOut = transformSync('t.ts', smallCode).code;
const mediumOut = transformSync('t.ts', mediumCode).code;
const largeOut = transformSync('t.ts', largeCode).code;

console.log(`Small code:  input=${smallCode.length}B, output=${smallOut.length}B`);
console.log(`Medium code: input=${mediumCode.length}B, output=${mediumOut.length}B`);
console.log(`Large code:  input=${largeCode.length}B, output=${largeOut.length}B`);
console.log('');

const results = [];

for (const [name, code] of [['small', smallCode], ['medium', mediumCode], ['large', largeCode]]) {
  console.log(`--- ${name} code ---`);

  const sync = benchmark(`transformSync (${name})`, () => transformSync('t.ts', code));
  const fast = benchmark(`transformSyncFast (${name})`, () => transformSyncFast('t.ts', code));

  const diff = ((fast.mean - sync.mean) / sync.mean * 100);
  const absDiff = (fast.mean - sync.mean) * 1000; // in microseconds

  console.log(`transformSync:     ${sync.mean.toFixed(4)} ms (p50: ${sync.p50.toFixed(4)} ms)`);
  console.log(`transformSyncFast: ${fast.mean.toFixed(4)} ms (p50: ${fast.p50.toFixed(4)} ms)`);
  console.log(`Difference: ${diff.toFixed(2)}% (${absDiff.toFixed(1)} µs) - FastString is ${diff > 0 ? 'SLOWER' : 'FASTER'}`);
  console.log('');

  results.push({ name, sync: sync.mean, fast: fast.mean, diff, absDiff });
}

console.log('='.repeat(80));
console.log('Summary:');
console.log('');
console.log('| Output Size | transformSync | transformSyncFast | Diff (µs) | Diff (%) |');
console.log('|-------------|---------------|-------------------|-----------|----------|');
for (const r of results) {
  const outSize = r.name === 'small' ? smallOut.length : r.name === 'medium' ? mediumOut.length : largeOut.length;
  console.log(`| ${String(outSize).padStart(9)}B | ${r.sync.toFixed(4).padStart(13)} | ${r.fast.toFixed(4).padStart(17)} | ${r.absDiff.toFixed(1).padStart(9)} | ${(r.diff > 0 ? '+' : '') + r.diff.toFixed(1).padStart(7)}% |`);
}
