import {readFile, writeFile} from 'fs/promises';
import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

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
const cacheDirPath = pathJoin(__dirname, '../../target');

export default await Promise.all(urls.map(async (url) => {
    const filename = url.split('/').at(-1),
        path = pathJoin(cacheDirPath, filename);

    let sourceStr;
    try {
        sourceStr = await readFile(path, 'utf8');
    } catch {
        const res = await fetch(url);
        sourceStr = await res.text();
        await writeFile(path, sourceStr);
    }

    // Remove a few Unicode characters
    // TODO: Deserialization works without this, but is a bit slower.
    // Make this unnecessary by encoding UTF-16 offset into `Atom`.
    sourceStr = sourceStr.replace(/৹/, 'x').replace(/ç/g, 'c').replace(/[–—]/g, '-')
        .replace(/[“”]/g, '"').replace(/’/g, "'")
        .replace(/•/g, '*').replace(/[🏝️😄😴]/g, '_').replace(/ﬅ|ſt/g, 'ft');

    const sourceBuff = Buffer.from(sourceStr);

    return {filename, sourceBuff, sourceStr};
}));
