/**
 * Benchmark script to compare sync and async transform variants for oxc-transform.
 *
 * Run with: node bench/string-perf.mjs
 *
 * This script benchmarks all 4 transform variants:
 * - transformSync (baseline sync)
 * - transformSyncFast (optimized sync)
 * - transform (baseline async)
 * - transformFast (optimized async)
 *
 * with various input types:
 * - ascii_small: Small ASCII-only TypeScript code (~100 bytes)
 * - ascii_large: Large ASCII-only TypeScript code (~100KB)
 * - unicode_small: Small code with Unicode characters
 * - unicode_large: Large code with Unicode characters
 *
 * The optimized versions (transformSyncFast, transformFast) use FastString which can
 * avoid copying string data for ASCII-only output. Unicode inputs may not see improvement.
 */

import { transformSync, transformSyncFast, transform, transformFast } from '../index.js';

// ============================================================================
// Configuration
// ============================================================================

const WARMUP_ITERATIONS = 10;
const BENCHMARK_ITERATIONS = 100;

// Target sizes
const SMALL_SIZE = 100;       // ~100 bytes
const LARGE_SIZE = 100 * 1024; // ~100KB

// ============================================================================
// Test Input Generators
// ============================================================================

/**
 * Generate valid TypeScript/JavaScript code that is purely ASCII.
 * Uses patterns like: const varN = "string"; export { varN };
 * @param {number} targetSize - Target size in bytes
 * @returns {string} Generated ASCII-only code
 */
function generateAsciiCode(targetSize) {
  const lines = [];
  const exports = [];
  let currentSize = 0;
  let varIndex = 0;

  while (currentSize < targetSize) {
    const varName = `var${varIndex}`;
    // Create a line like: const var0: string = "some string value here";
    const stringValue = `value_${varIndex}_${'x'.repeat(20)}`;
    const line = `const ${varName}: string = "${stringValue}";\n`;

    lines.push(line);
    exports.push(varName);
    currentSize += line.length;
    varIndex++;
  }

  // Add export statement
  const exportLine = `export { ${exports.join(', ')} };\n`;
  lines.push(exportLine);

  return lines.join('');
}

/**
 * Generate code with Unicode characters: const msg = "Hello 世界";
 * @param {number} targetSize - Target size in bytes
 * @returns {string} Generated code with Unicode
 */
function generateUnicodeCode(targetSize) {
  const lines = [];
  const exports = [];
  let currentSize = 0;
  let varIndex = 0;

  // Unicode strings to include (mix of different scripts and emojis)
  const unicodeStrings = [
    'Hello 世界',           // Chinese
    'Hola Mundo',          // Spanish with accents potentially
    'Bonjour le monde',    // French
    'Hallo Welt',          // German with umlauts potentially
    'Ciao Mondo',          // Italian
    'Olá Mundo',           // Portuguese
    'Привет мир',          // Russian
    'مرحبا بالعالم',       // Arabic
    'שלום עולם',           // Hebrew
    'こんにちは世界',       // Japanese
    '안녕하세요 세계',       // Korean
    'Γειά σου Κόσμε',      // Greek
    'สวัสดีโลก',            // Thai
    'Xin chào Thế giới',   // Vietnamese
    'नमस्ते दुनिया',         // Hindi
  ];

  while (currentSize < targetSize) {
    const varName = `msg${varIndex}`;
    const unicodeStr = unicodeStrings[varIndex % unicodeStrings.length];
    // Create a line like: const msg0: string = "Hello 世界 value_0";
    const line = `const ${varName}: string = "${unicodeStr} value_${varIndex}";\n`;

    lines.push(line);
    exports.push(varName);
    // Use Buffer.byteLength for accurate byte count with Unicode
    currentSize += Buffer.byteLength(line, 'utf8');
    varIndex++;
  }

  // Add export statement
  const exportLine = `export { ${exports.join(', ')} };\n`;
  lines.push(exportLine);

  return lines.join('');
}

// ============================================================================
// Benchmark Utilities
// ============================================================================

/**
 * Run a synchronous benchmark function with warmup and measurement phases.
 * @param {string} name - Name of the benchmark
 * @param {Function} fn - Function to benchmark (should be synchronous)
 * @param {number} warmupIterations - Number of warmup iterations
 * @param {number} iterations - Number of measured iterations
 * @returns {Object} Benchmark results
 */
function benchmark(name, fn, warmupIterations = WARMUP_ITERATIONS, iterations = BENCHMARK_ITERATIONS) {
  // Warmup phase
  for (let i = 0; i < warmupIterations; i++) {
    fn();
  }

  // Measurement phase
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    fn();
    const end = performance.now();
    times.push(end - start);
  }

  // Calculate statistics
  const total = times.reduce((a, b) => a + b, 0);
  const mean = total / iterations;
  const sorted = [...times].sort((a, b) => a - b);
  const median = sorted[Math.floor(iterations / 2)];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  // Standard deviation
  const squaredDiffs = times.map(t => Math.pow(t - mean, 2));
  const avgSquaredDiff = squaredDiffs.reduce((a, b) => a + b, 0) / iterations;
  const stdDev = Math.sqrt(avgSquaredDiff);

  return {
    name,
    iterations,
    mean,
    median,
    min,
    max,
    stdDev,
    total,
  };
}

/**
 * Run an async benchmark function with warmup and measurement phases.
 * @param {string} name - Name of the benchmark
 * @param {Function} fn - Async function to benchmark (should return a Promise)
 * @param {number} warmupIterations - Number of warmup iterations
 * @param {number} iterations - Number of measured iterations
 * @returns {Promise<Object>} Benchmark results
 */
async function benchmarkAsync(name, fn, warmupIterations = WARMUP_ITERATIONS, iterations = BENCHMARK_ITERATIONS) {
  // Warmup phase
  for (let i = 0; i < warmupIterations; i++) {
    await fn();
  }

  // Measurement phase
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    const end = performance.now();
    times.push(end - start);
  }

  // Calculate statistics
  const total = times.reduce((a, b) => a + b, 0);
  const mean = total / iterations;
  const sorted = [...times].sort((a, b) => a - b);
  const median = sorted[Math.floor(iterations / 2)];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  // Standard deviation
  const squaredDiffs = times.map(t => Math.pow(t - mean, 2));
  const avgSquaredDiff = squaredDiffs.reduce((a, b) => a + b, 0) / iterations;
  const stdDev = Math.sqrt(avgSquaredDiff);

  return {
    name,
    iterations,
    mean,
    median,
    min,
    max,
    stdDev,
    total,
  };
}

/**
 * Format benchmark results for display.
 * @param {Object} result - Benchmark result object
 * @returns {string} Formatted result string
 */
function formatResult(result) {
  return [
    `${result.name}:`,
    `  Iterations: ${result.iterations}`,
    `  Mean:       ${result.mean.toFixed(4)} ms/iter`,
    `  Median:     ${result.median.toFixed(4)} ms/iter`,
    `  Min:        ${result.min.toFixed(4)} ms`,
    `  Max:        ${result.max.toFixed(4)} ms`,
    `  Std Dev:    ${result.stdDev.toFixed(4)} ms`,
    `  Total:      ${result.total.toFixed(2)} ms`,
  ].join('\n');
}

/**
 * Calculate and format the comparison between baseline and optimized.
 * @param {Object} baseline - Baseline benchmark result
 * @param {Object} optimized - Optimized benchmark result
 * @param {string} inputType - Type of input (ascii/unicode)
 * @returns {string} Formatted comparison string
 */
function formatComparison(baseline, optimized, inputType) {
  const improvement = ((baseline.mean - optimized.mean) / baseline.mean) * 100;
  const speedup = baseline.mean / optimized.mean;

  const isUnicode = inputType.startsWith('unicode');
  let note = '';
  if (isUnicode && improvement < 5) {
    note = ' (expected: FastString optimization does not apply to Unicode output)';
  }

  if (improvement > 0) {
    return `  Improvement: ${improvement.toFixed(2)}% faster (${speedup.toFixed(2)}x speedup)${note}`;
  } else {
    return `  Improvement: ${(-improvement).toFixed(2)}% slower${note}`;
  }
}

/**
 * Print a separator line.
 */
function printSeparator() {
  console.log('='.repeat(80));
}

// ============================================================================
// Main Benchmark
// ============================================================================

async function main() {
  console.log('String Performance Benchmark: Sync and Async Transform Variants');
  console.log('');
  printSeparator();

  // Check if all transform functions are available
  if (typeof transformSyncFast !== 'function') {
    console.error('ERROR: transformSyncFast is not available.');
    console.error('Make sure the native binding is built with the fast string API.');
    process.exit(1);
  }
  if (typeof transform !== 'function') {
    console.error('ERROR: transform (async) is not available.');
    console.error('Make sure the native binding is built with the async API.');
    process.exit(1);
  }
  if (typeof transformFast !== 'function') {
    console.error('ERROR: transformFast (async) is not available.');
    console.error('Make sure the native binding is built with the async fast string API.');
    process.exit(1);
  }

  // Generate test inputs
  console.log('Generating test inputs...');

  const inputs = {
    ascii_small: generateAsciiCode(SMALL_SIZE),
    ascii_large: generateAsciiCode(LARGE_SIZE),
    unicode_small: generateUnicodeCode(SMALL_SIZE),
    unicode_large: generateUnicodeCode(LARGE_SIZE),
  };

  // Report input sizes
  console.log('');
  console.log('Input sizes:');
  for (const [name, code] of Object.entries(inputs)) {
    const byteSize = Buffer.byteLength(code, 'utf8');
    const charCount = code.length;
    console.log(`  ${name}: ${byteSize} bytes, ${charCount} chars`);
  }
  console.log('');
  printSeparator();

  // Verify all functions work
  console.log('Verifying all transform functions work with test inputs...');
  for (const [name, code] of Object.entries(inputs)) {
    try {
      const result1 = transformSync('test.ts', code);
      const result2 = transformSyncFast('test.ts', code);
      const result3 = await transform('test.ts', code);
      const result4 = await transformFast('test.ts', code);

      const results = [
        { name: 'transformSync', result: result1 },
        { name: 'transformSyncFast', result: result2 },
        { name: 'transform', result: result3 },
        { name: 'transformFast', result: result4 },
      ];

      let hasErrors = false;
      for (const { name: fnName, result } of results) {
        if (result.errors && result.errors.length > 0) {
          console.error(`  ${name} (${fnName}): FAILED - ${result.errors.length} errors`);
          console.error(`    First error: ${JSON.stringify(result.errors[0])}`);
          hasErrors = true;
        }
      }

      if (!hasErrors) {
        // Verify all outputs match
        const allMatch = results.every(r => r.result.code === result1.code);
        if (allMatch) {
          console.log(`  ${name}: OK (output: ${result1.code.length} chars, all outputs match)`);
        } else {
          console.warn(`  ${name}: WARNING - outputs differ!`);
          for (const { name: fnName, result } of results) {
            console.warn(`    ${fnName}: ${result.code.length} chars`);
          }
        }
      }
    } catch (err) {
      console.error(`  ${name}: ERROR - ${err.message}`);
    }
  }
  console.log('');
  printSeparator();

  // Run benchmarks
  console.log('Running benchmarks...');
  console.log(`  Warmup iterations: ${WARMUP_ITERATIONS}`);
  console.log(`  Benchmark iterations: ${BENCHMARK_ITERATIONS}`);
  console.log('');

  const results = {
    transformSync: [],
    transformSyncFast: [],
    transform: [],
    transformFast: [],
  };

  for (const [name, code] of Object.entries(inputs)) {
    console.log(`Benchmarking: ${name}`);
    console.log('-'.repeat(40));

    // transformSync (baseline sync)
    const syncResult = benchmark(
      `transformSync (${name})`,
      () => transformSync('test.ts', code),
      WARMUP_ITERATIONS,
      BENCHMARK_ITERATIONS
    );
    results.transformSync.push({ ...syncResult, inputName: name });
    console.log(formatResult(syncResult));
    console.log('');

    // transformSyncFast (optimized sync)
    const syncFastResult = benchmark(
      `transformSyncFast (${name})`,
      () => transformSyncFast('test.ts', code),
      WARMUP_ITERATIONS,
      BENCHMARK_ITERATIONS
    );
    results.transformSyncFast.push({ ...syncFastResult, inputName: name });
    console.log(formatResult(syncFastResult));
    console.log('');

    // transform (baseline async)
    const asyncResult = await benchmarkAsync(
      `transform (${name})`,
      () => transform('test.ts', code),
      WARMUP_ITERATIONS,
      BENCHMARK_ITERATIONS
    );
    results.transform.push({ ...asyncResult, inputName: name });
    console.log(formatResult(asyncResult));
    console.log('');

    // transformFast (optimized async)
    const asyncFastResult = await benchmarkAsync(
      `transformFast (${name})`,
      () => transformFast('test.ts', code),
      WARMUP_ITERATIONS,
      BENCHMARK_ITERATIONS
    );
    results.transformFast.push({ ...asyncFastResult, inputName: name });
    console.log(formatResult(asyncFastResult));
    console.log('');

    // Comparison (sync vs sync fast)
    console.log('Sync comparison:');
    console.log(formatComparison(syncResult, syncFastResult, name));
    console.log('');

    // Comparison (async vs async fast)
    console.log('Async comparison:');
    console.log(formatComparison(asyncResult, asyncFastResult, name));
    console.log('');
  }

  printSeparator();

  // Summary table
  console.log('Summary Table (ms per iteration):');
  console.log('');
  console.log('| Input         | transformSync | transformSyncFast | Sync Improv | transform     | transformFast   | Async Improv |');
  console.log('|---------------|---------------|-------------------|-------------|---------------|-----------------|--------------|');

  for (let i = 0; i < results.transformSync.length; i++) {
    const syncResult = results.transformSync[i];
    const syncFastResult = results.transformSyncFast[i];
    const asyncResult = results.transform[i];
    const asyncFastResult = results.transformFast[i];

    const syncImprovement = ((syncResult.mean - syncFastResult.mean) / syncResult.mean) * 100;
    const asyncImprovement = ((asyncResult.mean - asyncFastResult.mean) / asyncResult.mean) * 100;

    const name = syncResult.inputName.padEnd(13);
    const syncMean = syncResult.mean.toFixed(4).padStart(13);
    const syncFastMean = syncFastResult.mean.toFixed(4).padStart(17);
    const syncImprovementStr = syncImprovement > 0
      ? `+${syncImprovement.toFixed(1)}%`.padStart(11)
      : `${syncImprovement.toFixed(1)}%`.padStart(11);
    const asyncMean = asyncResult.mean.toFixed(4).padStart(13);
    const asyncFastMean = asyncFastResult.mean.toFixed(4).padStart(15);
    const asyncImprovementStr = asyncImprovement > 0
      ? `+${asyncImprovement.toFixed(1)}%`.padStart(12)
      : `${asyncImprovement.toFixed(1)}%`.padStart(12);

    console.log(`| ${name} | ${syncMean} | ${syncFastMean} | ${syncImprovementStr} | ${asyncMean} | ${asyncFastMean} | ${asyncImprovementStr} |`);
  }

  console.log('');
  printSeparator();

  // Detailed comparison
  console.log('Detailed Comparison:');
  console.log('');

  for (let i = 0; i < results.transformSync.length; i++) {
    const syncResult = results.transformSync[i];
    const syncFastResult = results.transformSyncFast[i];
    const asyncResult = results.transform[i];
    const asyncFastResult = results.transformFast[i];
    const name = syncResult.inputName;
    const isUnicode = name.startsWith('unicode');

    const syncImprovement = ((syncResult.mean - syncFastResult.mean) / syncResult.mean) * 100;
    const syncSpeedup = syncResult.mean / syncFastResult.mean;
    const asyncImprovement = ((asyncResult.mean - asyncFastResult.mean) / asyncResult.mean) * 100;
    const asyncSpeedup = asyncResult.mean / asyncFastResult.mean;

    console.log(`${name}:`);
    console.log(`  Sync:`);
    console.log(`    transformSync:     ${syncResult.mean.toFixed(4)} ms (median: ${syncResult.median.toFixed(4)} ms)`);
    console.log(`    transformSyncFast: ${syncFastResult.mean.toFixed(4)} ms (median: ${syncFastResult.median.toFixed(4)} ms)`);
    if (syncImprovement > 0) {
      console.log(`    Result: ${syncImprovement.toFixed(2)}% faster (${syncSpeedup.toFixed(2)}x speedup)`);
    } else {
      console.log(`    Result: ${(-syncImprovement).toFixed(2)}% slower`);
    }

    console.log(`  Async:`);
    console.log(`    transform:         ${asyncResult.mean.toFixed(4)} ms (median: ${asyncResult.median.toFixed(4)} ms)`);
    console.log(`    transformFast:     ${asyncFastResult.mean.toFixed(4)} ms (median: ${asyncFastResult.median.toFixed(4)} ms)`);
    if (asyncImprovement > 0) {
      console.log(`    Result: ${asyncImprovement.toFixed(2)}% faster (${asyncSpeedup.toFixed(2)}x speedup)`);
    } else {
      console.log(`    Result: ${(-asyncImprovement).toFixed(2)}% slower`);
    }

    if (isUnicode) {
      console.log(`  Note: Unicode input - FastString optimization may not apply to output`);
    } else if (syncImprovement > 5 || asyncImprovement > 5) {
      console.log(`  Note: ASCII input - FastString zero-copy optimization is effective`);
    }
    console.log('');
  }

  printSeparator();

  // Throughput analysis
  console.log('Throughput (MB/s):');
  console.log('');
  console.log('| Input         | transformSync | transformSyncFast | transform     | transformFast   |');
  console.log('|---------------|---------------|-------------------|---------------|-----------------|');

  for (const [name, code] of Object.entries(inputs)) {
    const syncResult = results.transformSync.find(r => r.inputName === name);
    const syncFastResult = results.transformSyncFast.find(r => r.inputName === name);
    const asyncResult = results.transform.find(r => r.inputName === name);
    const asyncFastResult = results.transformFast.find(r => r.inputName === name);

    if (syncResult && syncFastResult && asyncResult && asyncFastResult) {
      const byteSize = Buffer.byteLength(code, 'utf8');

      const syncBytesPerMs = byteSize / syncResult.mean;
      const syncMbPerSec = (syncBytesPerMs * 1000) / (1024 * 1024);

      const syncFastBytesPerMs = byteSize / syncFastResult.mean;
      const syncFastMbPerSec = (syncFastBytesPerMs * 1000) / (1024 * 1024);

      const asyncBytesPerMs = byteSize / asyncResult.mean;
      const asyncMbPerSec = (asyncBytesPerMs * 1000) / (1024 * 1024);

      const asyncFastBytesPerMs = byteSize / asyncFastResult.mean;
      const asyncFastMbPerSec = (asyncFastBytesPerMs * 1000) / (1024 * 1024);

      const nameStr = name.padEnd(13);
      const syncStr = syncMbPerSec.toFixed(2).padStart(13);
      const syncFastStr = syncFastMbPerSec.toFixed(2).padStart(17);
      const asyncStr = asyncMbPerSec.toFixed(2).padStart(13);
      const asyncFastStr = asyncFastMbPerSec.toFixed(2).padStart(15);

      console.log(`| ${nameStr} | ${syncStr} | ${syncFastStr} | ${asyncStr} | ${asyncFastStr} |`);
    }
  }

  console.log('');
  console.log('Benchmark complete.');
}

// Run the benchmark
main().catch(console.error);
