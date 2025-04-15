import path from 'node:path';
import { callThreadsafeFunction } from './index.js';

// usage:
// node napi/parser/example.mjs test.ts

process.chdir(path.join(import.meta.dirname, '../..'));

async function main() {
  const data = [];
  callThreadsafeFunction((_, n) => {
    data.push(n);
  });
  console.log(data);
}

main();
