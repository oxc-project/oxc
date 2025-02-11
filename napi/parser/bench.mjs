import { writeFile } from 'fs/promises';
import { join as pathJoin } from 'path';
import { Bench } from 'tinybench';
import { parseSync } from './index.js';

// Same fixtures as used in Rust parser benchmarks
const fixtureUrls = [
  'https://raw.githubusercontent.com/microsoft/TypeScript/v5.3.3/src/compiler/checker.ts',
  'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/cal.com.tsx',
  'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/RadixUIAdoptionSection.jsx',
  'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
  'https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js',
];

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
const bench = new Bench();
for (const { filename, code } of fixtures) {
  bench.add(
    `parser_napi[${filename}]`,
    () => {
      parseSync(filename, code);
    },
  );
}

await bench.run();
console.table(bench.table());
