/*
 * Create cache of benchmark profile files.
 */

import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';
import {create as createTar} from 'tar';

const __dirname = dirname(fileURLToPath(import.meta.url));

const filesDir = process.env.DATA_DIR;
const archivePath = pathJoin(__dirname, 'cachedBenches.tar.gz');
await createTar({file: archivePath, gzip: true, cwd: filesDir}, ['./']);
