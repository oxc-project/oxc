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

import express from 'express';
import { createWriteStream } from 'fs';
import fs from 'fs/promises';
import { join as pathJoin } from 'path';
import { pipeline } from 'stream/promises';
import { extract } from 'tar';

const DEFAULT_PORT = 3000,
  LISTEN_ATTEMPTS = 10;

// Create directory for saving assets
const rand = Math.round(Math.random() * 100000000000000).toString(16),
  dataDir = `/tmp/oxc_bench_data_${rand}`;
await fs.mkdir(dataDir);

let component = process.env.COMPONENT;
if (process.env.FIXTURE) component += process.env.FIXTURE;

const app = express();

const wrapHandler = fn => (req, res, next) => {
  fn(req, res).catch(next);
};
const getFilePath = filename => pathJoin(dataDir, `${component}_${filename}`);

app.post(
  '/upload',
  wrapHandler(async (req, res) => {
    const stream = createWriteStream(getFilePath('metadata.json'));
    await pipeline(req, stream);

    res.json({
      status: 'success',
      uploadUrl: `http://localhost:${port}/upload_archive`,
      runId: 'dummy_value',
    });
  }),
);

app.put(
  '/upload_archive',
  wrapHandler(async (req, res) => {
    // Stream uploaded tarball to file
    const path = getFilePath('archive.tar.gz');
    const stream = createWriteStream(path);
    await pipeline(req, stream);

    // Untar contents + delete tarball
    await extract({ file: path, cwd: dataDir });
    await fs.rm(path);

    // Rename `.out` files + delete `.log` files
    const filenames = await fs.readdir(dataDir);
    for (const filename of filenames) {
      if (filename.endsWith('.log')) {
        await fs.rm(pathJoin(dataDir, filename));
      } else if (filename.endsWith('.out')) {
        await fs.rename(pathJoin(dataDir, filename), getFilePath(filename));
      }
    }

    // Send response
    res.send('');
    server.close(() => {});
  }),
);

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
await fs.appendFile(process.env.GITHUB_ENV, `DATA_DIR=${dataDir}\nINTERCEPT_PORT=${port}\n`);
