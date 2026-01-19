/**
 * Focused benchmark with more iterations for statistical significance
 */
import { transformSync, transformSyncFast, transform, transformFast } from '../index.js';

const ITERATIONS = 500;
const WARMUP = 50;

// Generate ~100KB ASCII code
function generateAsciiCode(size) {
  const lines = [];
  let currentSize = 0;
  let i = 0;
  while (currentSize < size) {
    const line = `const v${i}: string = "${'x'.repeat(30)}";\n`;
    lines.push(line);
    currentSize += line.length;
    i++;
  }
  const exports = [];
  for (let j = 0; j < i; j++) {
    exports.push(`v${j}`);
  }
  return lines.join('') + `export { ${exports.join(', ')} };`;
}

const ascii_large = generateAsciiCode(100000);
console.log(`Input size: ${ascii_large.length} bytes`);

function benchmarkSync(name, fn) {
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
  const p95 = sorted[Math.floor(times.length * 0.95)];
  const p99 = sorted[Math.floor(times.length * 0.99)];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  console.log(`${name.padEnd(22)} mean: ${mean.toFixed(3)}ms  p50: ${p50.toFixed(3)}ms  p95: ${p95.toFixed(3)}ms  min: ${min.toFixed(3)}ms  max: ${max.toFixed(3)}ms`);
  return { mean, p50, p95, min, max };
}

async function benchmarkAsync(name, fn) {
  // Warmup
  for (let i = 0; i < WARMUP; i++) await fn();

  const times = [];
  for (let i = 0; i < ITERATIONS; i++) {
    const start = performance.now();
    await fn();
    times.push(performance.now() - start);
  }

  const sorted = [...times].sort((a, b) => a - b);
  const mean = times.reduce((a, b) => a + b) / times.length;
  const p50 = sorted[Math.floor(times.length * 0.5)];
  const p95 = sorted[Math.floor(times.length * 0.95)];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  console.log(`${name.padEnd(22)} mean: ${mean.toFixed(3)}ms  p50: ${p50.toFixed(3)}ms  p95: ${p95.toFixed(3)}ms  min: ${min.toFixed(3)}ms  max: ${max.toFixed(3)}ms`);
  return { mean, p50, p95, min, max };
}

console.log(`\n${'='.repeat(100)}`);
console.log(`Running ${ITERATIONS} iterations (${WARMUP} warmup)...`);
console.log(`${'='.repeat(100)}\n`);

console.log('SYNC BENCHMARKS:');
console.log('-'.repeat(100));
const syncBase = benchmarkSync('transformSync', () => transformSync('test.ts', ascii_large));
const syncFast = benchmarkSync('transformSyncFast', () => transformSyncFast('test.ts', ascii_large));

const syncDiff = ((syncFast.mean - syncBase.mean) / syncBase.mean * 100);
console.log(`\n=> Sync difference: ${syncDiff.toFixed(2)}% (FastString is ${syncDiff > 0 ? 'SLOWER' : 'FASTER'})`);
console.log(`   p50 diff: ${((syncFast.p50 - syncBase.p50) / syncBase.p50 * 100).toFixed(2)}%`);

console.log(`\n${'='.repeat(100)}\n`);

console.log('ASYNC BENCHMARKS:');
console.log('-'.repeat(100));

async function runAsync() {
  const asyncBase = await benchmarkAsync('transform', async () => await transform('test.ts', ascii_large));
  const asyncFast = await benchmarkAsync('transformFast', async () => await transformFast('test.ts', ascii_large));

  const asyncDiff = ((asyncFast.mean - asyncBase.mean) / asyncBase.mean * 100);
  console.log(`\n=> Async difference: ${asyncDiff.toFixed(2)}% (FastString is ${asyncDiff > 0 ? 'SLOWER' : 'FASTER'})`);
  console.log(`   p50 diff: ${((asyncFast.p50 - asyncBase.p50) / asyncBase.p50 * 100).toFixed(2)}%`);
}

await runAsync();

console.log(`\n${'='.repeat(100)}`);
console.log('Benchmark complete.');
