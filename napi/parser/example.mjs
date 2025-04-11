import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { parseSync } from './index.js';

// usage:
// node napi/parser/example.mjs test.ts --experimentalRawTransfer

process.chdir(path.join(import.meta.dirname, '../..'));

function main() {
  const args = parseArgs({
    args: process.argv.slice(2),
    allowPositionals: true,
    options: {
      lang: {
        type: 'string',
      },
      astType: {
        type: 'string',
      },
      experimentalRawTransfer: {
        type: 'boolean',
      },
    },
  });
  const file = args.positionals[0] ?? 'test.js';
  const code = fs.readFileSync(file, 'utf-8');
  const result = parseSync(file, code, args.values);
  console.dir({ ...result }, { depth: Infinity });
}

main();
