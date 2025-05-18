import { writeFile } from 'fs/promises';
import { join as pathJoin } from 'path';
import { bench } from 'vitest';
import { parseSync } from './index.js';

// Same fixtures as used in Rust parser benchmarks
let fixtureUrls = [
  'https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts',
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx',
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx',
  'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
  'https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js',
];

// For sharding in CI - specify single fixture to run benchmarks on
let benchStandard = bench, benchRaw = bench;
let shard = process.env.SHARD;
if (shard) {
  shard *= 1;
  if (shard % 2 === 0) {
    benchRaw = bench.skip;
  } else {
    benchStandard = bench.skip;
    shard--;
  }
  fixtureUrls = [fixtureUrls[shard / 2]];
}

// Same directory as Rust benchmarks use for downloaded files
// to avoid re-downloading if Rust benchmarks already downloaded
const cacheDirPath = pathJoin(import.meta.dirname, '../../target');

// Load fixtures
const fixtures = await Promise.all(fixtureUrls.map(async (url) => {
  const filename = url.split('/').at(-1),
    path = pathJoin(cacheDirPath, filename);

  let code;
  try {
    code = await readFile(path, 'utf8');
  } catch {
    const res = await fetch(url);
    code = await res.text();
    await writeFile(path, code);
  }

  return { filename, code };
}));

// Run benchmarks
for (const { filename, code } of fixtures) {
  benchStandard(`parser_napi[${filename}]`, () => {
    const ret = parseSync(filename, code);
    // Read returned object's properties to execute getters which deserialize
    const { program, comments, module, errors } = ret;
  });

  benchRaw(`parser_napi_raw[${filename}]`, () => {
    const ret = parseSync(filename, code, { experimentalRawTransfer: true });
    // Read returned object's properties to execute getters
    const { program, comments, module, errors } = ret;
  });
}
