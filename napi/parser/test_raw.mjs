import {writeFile, mkdir, rm} from 'fs/promises';
import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';
import {spawnSync} from 'child_process';
import {expect} from 'expect';
import oxc from './index.js';
import deserialize from './deserialize.js';
import fixtures from './fixtures.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));

for (const {filename, sourceBuff, sourceStr, allocSize} of fixtures) {
    await test(filename, sourceBuff, sourceStr, allocSize);
}

async function test(filename, sourceBuff, sourceText, allocSize) {
    console.log('Testing:', filename);

    const astViaJson = JSON.parse(oxc.parseSync(sourceText, {sourceFilename: filename}).program);
    // console.dir(astViaJson, {depth: 10});

    const buff = oxc.parseSyncRaw(sourceBuff, {sourceFilename: filename}, allocSize);
    const astRaw = deserialize(buff, sourceBuff);
    // console.dir(astRaw, {depth: 10});

    if (JSON.stringify(astRaw) === JSON.stringify(astViaJson)) {
        console.log('> Pass');
    } else {
        console.log('> Fail');

        const diffPath = pathJoin(__dirname, 'diffTemp');
        await mkdir(diffPath, {recursive: true});
        const pathJson = pathJoin(diffPath, `${filename}.json.json`),
            pathRaw = pathJoin(diffPath, `${filename}.raw.json`);
        await writeFile(pathJson, JSON.stringify(astViaJson, null, 2));
        await writeFile(pathRaw, JSON.stringify(astRaw, null, 2));
        const diff = spawnSync('diff', [pathJson, pathRaw]).stdout;
        await writeFile(pathJoin(__dirname, `${filename}.diff`), diff);
        await rm(diffPath, {recursive: true});
    }

    // assertEqual(astRaw, astViaJson);
}

function assertEqual(val1, val2) {
    try {
        expect(val1).toEqual(val2);
    } catch (err) {
        delete err.matcherResult;
        throw err;
    }
}
