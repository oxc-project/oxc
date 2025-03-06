import { readFileSync, writeFileSync } from 'node:fs';
import { join as pathJoin } from 'node:path';

// Add missing `export * from '@oxc-project/types';` line to `index.d.ts`
const path = pathJoin(import.meta.dirname, '../index.d.ts');
let dts = readFileSync(path, 'utf8');
const lines = dts.split('\n');
lines.splice(2, 0, '', "export * from '@oxc-project/types';");
dts = lines.join('\n');
writeFileSync(path, dts);
