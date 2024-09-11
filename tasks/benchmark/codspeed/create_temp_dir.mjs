/*
 * Create temp dir for downloading artefacts to.
 */

import fs from 'fs';

// Create directory for saving assets
const rand = Math.round(Math.random() * 1000000000000000000).toString(16),
  dataDir = `/tmp/oxc_bench_data_${rand}`;
fs.mkdirSync(dataDir);

// Output dir path to env var
fs.appendFileSync(process.env.GITHUB_ENV, `DATA_DIR=${dataDir}\n`);
