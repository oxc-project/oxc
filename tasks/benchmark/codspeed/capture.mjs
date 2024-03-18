/*
 * HTTP server to intercept benchmark data from Codspeed runner.
 * 
 * Codspeed runner makes 2 API calls:
 * 1. Uploading metadata
 * 2. Uploading archive of CPU profile files
 * 
 * Server starts on an available port, saves the files sent by Codspeed runner to a directory,
 * then shuts itself down.
 */

import fs from 'fs';
import {pipeline} from 'stream/promises';
import express from 'express';

const DEFAULT_PORT = 3000,
    LISTEN_ATTEMPTS = 10;

// Create directory for saving assets
const rand = Math.round(Math.random() * 1000000000000000000).toString(16),
    dataDir = `/tmp/oxc_bench_data_${rand}`;
fs.mkdirSync(dataDir);

let component = process.env.COMPONENT;
if (process.env.FIXTURE) component += process.env.FIXTURE;

const app = express();

app.post('/upload', (req, res, next) => {
    saveBody(req, 'metadata.json', next, () => {
        res.json({
            status: 'success',
            uploadUrl: `http://localhost:${port}/upload_archive`,
            runId: 'dummy_value',
        });
    });
});

app.put('/upload_archive', (req, res, next) => {
    saveBody(req, 'archive.tar.gz', next, () => {
        res.send('OK');
        server.close(() => {});
    });
});

function saveBody(req, filename, onError, done) {
    (async () => {
        const stream = fs.createWriteStream(`${dataDir}/${component}_${filename}`);
        await pipeline(req, stream);
        done();
    })().catch(onError);
}

// Open server on a port which is not already in use
let server,
    port = DEFAULT_PORT;
for (let i = 0; i < LISTEN_ATTEMPTS; i++) {
    console.log(`Starting server on port ${port}`);
    try {
        await new Promise((resolve, reject) => {
            server = app.listen(port, resolve);
            server.on('error', reject);
        });
        break;
    } catch (err) {
        if (err?.code !== 'EADDRINUSE') throw err;
        console.log(`Port ${port} in use. Trying again.`);
        port = DEFAULT_PORT + Math.round(Math.random() * 5000);
    }
}
console.log(`Server listening on port ${port}`);

// Output data dir path + port to env vars
fs.appendFileSync(process.env.GITHUB_ENV, `DATA_DIR=${dataDir}\nINTERCEPT_PORT=${port}\n`);
