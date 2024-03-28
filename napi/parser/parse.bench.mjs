import {join as pathJoin} from 'path';
import {writeFile} from 'fs/promises';
import {Bench} from 'tinybench';
import {parseSyncRaw, createBuffer} from './index.js';
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
const buff = createBuffer();

for (const {filename, sourceBuff, sourceStr} of fixtures) {
    bench.add(
        `parser_napi[${filename}]`,
        () => {
            parseSyncRaw(buff, sourceBuff.length, {sourceFilename: filename});
            deserialize(buff, sourceStr, sourceBuff.length);
        },
        {
            beforeAll() {
                // Writing source into buffer is not done in the bench loop,
                // as presumably would need to load source from a file anyway,
                // and we'll provide an API to load it direct into buffer
                buff.set(sourceBuff);
            }
        }
    );
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
