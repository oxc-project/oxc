/*
 * Combine benchmark data from different jobs, and upload to Codspeed.
 */

import {createReadStream} from 'fs';
import fs from 'fs/promises';
import {join as pathJoin} from 'path';
import {createHash} from 'crypto';
import assert from 'assert';
import tar from 'tar';
import axios from 'axios';

const METADATA_SUFFIX = 'metadata.json',
    ARCHIVE_SUFFIX = `archive.tar.gz`,
    CODSPEED_UPLOAD_URL = 'https://api.codspeed.io/upload';

const dataDir = process.env.DATA_DIR,
    token = process.env.CODSPEED_TOKEN;

// Get list of components
const components = (await fs.readdir(dataDir))
    .filter(filename => filename.endsWith(METADATA_SUFFIX))
    .map(filename => filename.slice(0, -METADATA_SUFFIX.length - 1));

// Unzip tarballs
const unzipDir = pathJoin(dataDir, 'unzip');
await fs.mkdir(unzipDir);

for (const component of components) {
    console.log(`Unzipping profile data: ${component}`);
    const archivePath = pathJoin(dataDir, `${component}_${ARCHIVE_SUFFIX}`);
    const componentUnzipDir = pathJoin(unzipDir, component);
    await fs.mkdir(componentUnzipDir);
    await tar.extract({file: archivePath, cwd: componentUnzipDir});
    await fs.rm(archivePath);
}

// Move all `.out` files to one directory
console.log('Combining profiles');

const outDir = pathJoin(dataDir, 'out');
await fs.mkdir(outDir);

const pids = new Set(),
    duplicates = [];
let highestPid = -1;
for (const component of components) {
    const componentDir = pathJoin(unzipDir, component);
    const outFiles = await fs.readdir(componentDir);
    for (const filename of outFiles) {
        if (!filename.endsWith('.out')) continue;
        let pid = filename.slice(0, -4);
        assert(/^\d+$/.test(pid), `Unexpected file: ${component}/${filename}`);
        pid *= 1;

        const path = pathJoin(componentDir, filename);
        if (pids.has(pid)) {
            // Duplicate PID
            duplicates.push({pid, path});
        } else {
            pids.add(pid);
            if (pid > highestPid) highestPid = pid;

            await fs.rename(path, pathJoin(outDir, `${pid}.out`));
        }
    }
}

// Alter PIDs for `.out` files with duplicate filenames
for (let {pid, path} of duplicates) {
    let content = await fs.readFile(path, 'utf8');

    const pidLine = `\npid: ${pid}\n`;
    const index = content.indexOf(pidLine);
    assert(index !== -1, `Could not locate PID in ${path}`);
    const before = content.slice(0, index);
    assert(before.split('\n').length === 3, `Unexpected formatting in ${path}`);

    pid = ++highestPid;
    content = `${before}\npid: ${pid}\n${content.slice(index + pidLine.length)}`;

    await fs.writeFile(pathJoin(outDir, `${pid}.out`), content);
    await fs.rm(path);
}

// Add log files to output dir
for (const filename of ['runner.log', 'valgrind.log']) {
    await fs.rename(pathJoin(unzipDir, components[0], filename), pathJoin(outDir, filename));
}

// ZIP combined profile directory
console.log('Zipping combined profile directory');
const archivePath = pathJoin(dataDir, 'archive.tar.gz');
await tar.create({file: archivePath, gzip: true, cwd: outDir}, ['./']);

// Get size + MD5 hash of archive
console.log('Hashing ZIP');
const {size} = await fs.stat(archivePath);

const hash = createHash('md5');
const inputStream = createReadStream(archivePath);
for await (const chunk of inputStream) {
    hash.update(chunk);
}
const md5 = hash.digest('base64');

// Alter MD5 hash in metadata object
const metadata = JSON.parse(
    await fs.readFile(pathJoin(dataDir, `${components[0]}_${METADATA_SUFFIX}`), 'utf8')
);
metadata.profileMd5 = md5;

// Upload metadata to CodSpeed
console.log('Uploading metadata to CodSpeed');
const {data} = await axios({
    method: 'post',
    url: CODSPEED_UPLOAD_URL,
    data: metadata,
    headers: {Authorization: token},
});
assert(data?.status === 'success', 'Failed to upload metadata to Codspeed');
const {uploadUrl} = data;

// Upload profile ZIP to Codspeed
console.log('Uploading profile ZIP to CodSpeed');
await axios({
    method: 'put',
    url: uploadUrl,
    data: createReadStream(archivePath),
    headers: {
        'Content-Type': 'application/gzip',
        'Content-Length': size,
        'Content-MD5': md5,
    }
});
