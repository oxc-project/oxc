import {fileURLToPath} from 'url';
import {join as pathJoin} from 'path';
import {readFile, writeFile} from 'fs/promises';
import {Bench} from 'tinybench';
import {parseSync} from './index.js';

const IS_CI = !!process.env.CI;

const urls = [
    // TypeScript syntax (2.81MB)
    'https://raw.githubusercontent.com/microsoft/TypeScript/v5.3.3/src/compiler/checker.ts',
    // Real world app tsx (1.0M)
    'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/cal.com.tsx',
    // Real world content-heavy app jsx (3K)
    'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/RadixUIAdoptionSection.jsx',
    // Heavy with classes (554K)
    'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
    // ES5 (3.9M)
    'https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js',
];

// Same directory as Rust benchmarks use for downloaded files
const cacheDirPath = pathJoin(fileURLToPath(import.meta.url), '../../../target');

const files = await Promise.all(urls.map(async (url) => {
    const filename = url.split('/').at(-1),
        path = pathJoin(cacheDirPath, filename);

    let code;
    try {
        code = await readFile(path, 'utf8');
        if (IS_CI) console.log('Found cached file:', filename);
    } catch {
        if (IS_CI) console.log('Downloading:', filename);
        const res = await fetch(url);
        code = await res.text();
        await writeFile(path, code);
    }

    return {filename, code};
}));

const bench = new Bench();

for (const {filename, code} of files) {
    bench.add(`parser_napi[${filename}]`, () => {
        const res = parseSync(code, {sourceFilename: filename});
        JSON.parse(res.program);
    });
}

console.log('Warming up');
await bench.warmup();
console.log('Running benchmarks');
await bench.run();
console.table(bench.table());

// If running on CI, save results to file
if (IS_CI) {
    const dataDir = process.env.DATA_DIR;
    const results = bench.tasks.map(task => ({
        filename: task.name.match(/\[(.+)\]$/)[1],
        duration: task.result.period / 1000, // In seconds
    }));
    await writeFile(pathJoin(dataDir, 'results.json'), JSON.stringify(results));
}
