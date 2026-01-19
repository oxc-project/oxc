/**
 * Split Benchmark: ASCII-only vs Real-world scenarios
 *
 * 1. ASCII-only: Pure ASCII code (identifiers, keywords, operators)
 * 2. Real-world: Actual source files with comments, strings, unicode
 */
import { transformSync, transformSyncFast } from '../index.js';

const ITERATIONS = 500;
const WARMUP = 50;

// ============================================================================
// Test Case 1: Pure ASCII (best case for optimization)
// ============================================================================

const asciiOnly = `
// Pure ASCII TypeScript code
import { useState, useEffect } from 'react';
import { fetchData, processResult } from './utils';

interface User {
  id: number;
  name: string;
  email: string;
  active: boolean;
}

interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

async function fetchUsers(): Promise<ApiResponse<User[]>> {
  const response = await fetch('/api/users');
  const data = await response.json();
  return { data, status: 200, message: 'OK' };
}

export class UserService {
  private cache: Map<number, User> = new Map();

  async getUser(id: number): Promise<User | null> {
    if (this.cache.has(id)) {
      return this.cache.get(id) || null;
    }
    const response = await fetchUsers();
    const user = response.data.find(u => u.id === id);
    if (user) {
      this.cache.set(id, user);
    }
    return user || null;
  }
}

${Array.from({length: 100}, (_, i) => `const var${i}: string = "${'x'.repeat(50)}";`).join('\n')}

export { fetchUsers };
`.trim();

// ============================================================================
// Test Case 2: Real-world code with Unicode (comments, strings, i18n)
// ============================================================================

const realWorld = `
/**
 * User Authentication Module
 * 用户认证模块 - Módulo de autenticación
 *
 * @author 开发团队
 * @license MIT
 */
import { createHash } from 'crypto';
import { User, AuthToken, LoginResult } from './types';

// 错误消息 - Error messages - Mensajes de error
const ERROR_MESSAGES = {
  invalidCredentials: '用户名或密码错误 / Invalid credentials',
  accountLocked: 'アカウントがロックされています / Account locked',
  sessionExpired: 'Сессия истекла / Session expired',
  networkError: '네트워크 오류 / Network error',
} as const;

// Emoji constants for status 📊
const STATUS_ICONS = {
  success: '✅',
  error: '❌',
  warning: '⚠️',
  info: 'ℹ️',
};

interface AuthConfig {
  /** Maximum login attempts - 最大登录尝试次数 */
  maxAttempts: number;
  /** Lock duration in ms - ロック期間（ミリ秒） */
  lockDuration: number;
  /** Token expiry - Срок действия токена */
  tokenExpiry: number;
}

/**
 * Authenticate user with credentials
 * 使用凭据验证用户
 *
 * @param username - 用户名
 * @param password - パスワード
 * @returns Authentication result - 认证结果
 */
export async function authenticate(
  username: string,
  password: string,
  config: AuthConfig
): Promise<LoginResult> {
  console.log(\`🔐 Authenticating user: \${username}\`);

  // Validate input - 验证输入
  if (!username || !password) {
    return {
      success: false,
      error: ERROR_MESSAGES.invalidCredentials,
      icon: STATUS_ICONS.error,
    };
  }

  // Hash password - 密码哈希化
  const hashedPassword = createHash('sha256')
    .update(password + '盐值_salt_ソルト')
    .digest('hex');

  return {
    success: true,
    token: \`token_\${Date.now()}\`,
    message: '登录成功 ✨ Login successful!',
    icon: STATUS_ICONS.success,
  };
}

// 更多代码... More code... もっとコード...
${Array.from({length: 50}, (_, i) => `const msg${i} = "消息 ${i} - Message ${i} - メッセージ ${i}";`).join('\n')}

export { ERROR_MESSAGES, STATUS_ICONS };
`.trim();

// ============================================================================
// Test Case 3: Large ASCII (stress test)
// ============================================================================

function generateLargeAscii(sizeKB) {
  const lines = [];
  let size = 0;
  let i = 0;
  while (size < sizeKB * 1024) {
    const line = `const variable${i}: string = "${'abcdefghij'.repeat(5)}";\n`;
    lines.push(line);
    size += line.length;
    i++;
  }
  return lines.join('') + `export { ${Array.from({length: Math.min(i, 100)}, (_, j) => `variable${j}`).join(', ')} };`;
}

const largeAscii = generateLargeAscii(100); // 100KB

// ============================================================================
// Benchmark Function
// ============================================================================

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
  const p95 = sorted[Math.floor(times.length * 0.95)];

  return { name, mean, p50, p95 };
}

function runComparison(label, code) {
  const inputSize = Buffer.byteLength(code, 'utf8');
  const output = transformSync('test.ts', code);
  const outputSize = output.code.length;
  const isAscii = Buffer.from(code).every(b => b < 128);
  const outputIsAscii = Buffer.from(output.code).every(b => b < 128);

  console.log(`\n${'='.repeat(70)}`);
  console.log(`${label}`);
  console.log(`${'='.repeat(70)}`);
  console.log(`Input:  ${inputSize} bytes (${isAscii ? 'ASCII' : 'Unicode'})`);
  console.log(`Output: ${outputSize} bytes (${outputIsAscii ? 'ASCII' : 'Unicode'})`);
  console.log(`${'─'.repeat(70)}`);

  const baseline = benchmark('transformSync', () => transformSync('test.ts', code));
  const fast = benchmark('transformSyncFast', () => transformSyncFast('test.ts', code));

  const diffMean = ((fast.mean - baseline.mean) / baseline.mean * 100);
  const diffP50 = ((fast.p50 - baseline.p50) / baseline.p50 * 100);

  console.log(`transformSync:      mean=${baseline.mean.toFixed(4)}ms  p50=${baseline.p50.toFixed(4)}ms  p95=${baseline.p95.toFixed(4)}ms`);
  console.log(`transformSyncFast:  mean=${fast.mean.toFixed(4)}ms  p50=${fast.p50.toFixed(4)}ms  p95=${fast.p95.toFixed(4)}ms`);
  console.log(`${'─'.repeat(70)}`);
  console.log(`Difference: mean=${diffMean > 0 ? '+' : ''}${diffMean.toFixed(2)}%  p50=${diffP50 > 0 ? '+' : ''}${diffP50.toFixed(2)}%`);
  console.log(`Result: FastString is ${diffMean > 1 ? 'SLOWER ❌' : diffMean < -1 ? 'FASTER ✅' : 'SAME ➖'}`);

  return { label, inputSize, outputSize, isAscii, outputIsAscii, baseline, fast, diffMean, diffP50 };
}

// ============================================================================
// Main
// ============================================================================

console.log('Split Benchmark: ASCII-only vs Real-world');
console.log(`Iterations: ${ITERATIONS} (warmup: ${WARMUP})`);

const results = [];

// Run benchmarks
results.push(runComparison('1. ASCII-ONLY (Typical TypeScript)', asciiOnly));
results.push(runComparison('2. REAL-WORLD (Unicode: Chinese/Japanese/Korean/Emoji)', realWorld));
results.push(runComparison('3. LARGE ASCII (100KB stress test)', largeAscii));

// Summary
console.log(`\n${'='.repeat(70)}`);
console.log('SUMMARY');
console.log(`${'='.repeat(70)}`);
console.log('| Scenario              | Input    | Output   | ASCII? | Diff (mean) |');
console.log('|-----------------------|----------|----------|--------|-------------|');
for (const r of results) {
  const scenario = r.label.substring(3, 25).padEnd(21);
  const input = `${(r.inputSize/1024).toFixed(1)}KB`.padStart(8);
  const output = `${(r.outputSize/1024).toFixed(1)}KB`.padStart(8);
  const ascii = r.outputIsAscii ? 'Yes' : 'No ';
  const diff = `${r.diffMean > 0 ? '+' : ''}${r.diffMean.toFixed(1)}%`.padStart(11);
  console.log(`| ${scenario} | ${input} | ${output} | ${ascii}    | ${diff} |`);
}

console.log(`\n${'='.repeat(70)}`);
console.log('CONCLUSION');
console.log(`${'='.repeat(70)}`);
const asciiResult = results[0];
const unicodeResult = results[1];
console.log(`
ASCII-only code:    ${asciiResult.diffMean > 1 ? '❌ No benefit' : asciiResult.diffMean < -1 ? '✅ Faster' : '➖ No difference'}
Real-world Unicode: ${unicodeResult.diffMean > 1 ? '❌ No benefit' : unicodeResult.diffMean < -1 ? '✅ Faster' : '➖ No difference'} (expected: falls back to UTF-8)

The FastString optimization ${Math.abs(asciiResult.diffMean) < 2 ? 'shows no measurable benefit' : asciiResult.diffMean < -2 ? 'provides improvement' : 'adds overhead'} for this use case.
`);
