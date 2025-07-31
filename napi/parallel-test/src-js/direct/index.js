import { Worker } from 'node:worker_threads';

import { run } from '../bindings.js';

const WORKER_URL = new URL('./worker.js', import.meta.url);

let workers;
const success = await run(startWorkers);
await stopWorkers();

console.log('> Success:', success);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;

/**
 * @param {number} count - Number of workers
 * @returns {Promise<undefined>}
 */
async function startWorkers(count) {
  console.log('> Starting', count, 'workers');
  workers = await Promise.all(Array.from({ length: count }, (_, id) => {
    const worker = new Worker(WORKER_URL, { workerData: { id } });
    return new Promise(resolve => worker.addListener('message', () => resolve(worker)));
  }));
  console.log('> Started', count, 'workers');
}

function stopWorkers() {
  return Promise.all(workers.map(worker => worker.terminate()));
}
