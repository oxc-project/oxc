import {join as pathJoin} from 'path';
import {writeFile} from 'fs/promises';
import {Bench} from 'tinybench';
import {parseSyncRaw} from './index.js';
import deserialize from './deserialize.js';
import fixtures from './fixtures.mjs';

const IS_CI = !!process.env.CI,
    ACCURATE = IS_CI || process.env.ACCURATE;

const bench = new Bench(
    ACCURATE
    ? {
        warmupIterations: 20, // Default is 5
        time: 5000, // 5 seconds, default is 500 ms
        iterations: 100, // Default is 10
    }
    : undefined
);

for (const {filename, sourceBuff, sourceStr, allocSize} of fixtures) {
    bench.add(`parser_napi[${filename}]`, () => {
        const buff = parseSyncRaw(sourceBuff, {sourceFilename: filename}, allocSize);
        const sourceIsAscii = sourceBuff.length === sourceStr.length;
        deserialize(buff, sourceIsAscii ? sourceStr : sourceBuff, sourceIsAscii);
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
