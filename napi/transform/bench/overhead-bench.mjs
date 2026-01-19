/**
 * Benchmark to measure overhead of ASCII check
 */

const ITERATIONS = 1000;

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
  return lines.join('');
}

const ascii_large = generateAsciiCode(100000);
const buffer = Buffer.from(ascii_large);

console.log(`String size: ${ascii_large.length} bytes\n`);

// Benchmark ASCII check in JS (similar to what Rust does)
function isAscii(str) {
  for (let i = 0; i < str.length; i++) {
    if (str.charCodeAt(i) > 127) return false;
  }
  return true;
}

function isAsciiBuffer(buf) {
  for (let i = 0; i < buf.length; i++) {
    if (buf[i] > 127) return false;
  }
  return true;
}

// Warmup
for (let i = 0; i < 100; i++) {
  isAscii(ascii_large);
  isAsciiBuffer(buffer);
}

// Benchmark string check
let start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  isAscii(ascii_large);
}
let elapsed = performance.now() - start;
console.log(`isAscii (string):  ${(elapsed / ITERATIONS).toFixed(4)} ms per check`);

// Benchmark buffer check
start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  isAsciiBuffer(buffer);
}
elapsed = performance.now() - start;
console.log(`isAscii (buffer):  ${(elapsed / ITERATIONS).toFixed(4)} ms per check`);

// For comparison - transform time is ~12ms, so if ASCII check is ~0.5ms that's significant
console.log(`\nFor reference: transform takes ~12ms, so ASCII check overhead could be ~${(elapsed / ITERATIONS / 12 * 100).toFixed(1)}% of total time`);
