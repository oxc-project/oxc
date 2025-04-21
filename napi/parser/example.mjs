import path from 'node:path';
import { linterMain } from './index.js';

// usage:
// node napi/parser/example.mjs test.ts

process.chdir(path.join(import.meta.dirname, '../..'));

async function main() {
  const data = [];
  await linterMain(async (_, n) => {
    data.push(n.program);
    await new Promise(r => setTimeout(r, 500));
    return 1;
  });
  console.log(data);
}

main();
