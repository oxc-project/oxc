import { Worker } from 'node:worker_threads';

import { run } from '../bindings.js';

const WORKER_URL = new URL('./worker.js', import.meta.url);

let log = false;

const workers = [];
const success = await run(startWorkers);
await stopWorkers();

if (log) console.log('> Success:', success);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;

/**
 * @param {number} count - Number of workers
 * @param {boolean} shouldLog - `true` if logging is enabled
 * @returns {Promise<undefined>}
 */
function startWorkers(count, shouldLog) {
  log = shouldLog;

  if (log) console.log('> Starting', count, 'workers');

  return new Promise((resolve) => {
    let remainingCount = count;
    function done(_dummy) {
      if (--remainingCount === 0) {
        resolve();
        if (log) console.log('> Started', count, 'workers');
      }
    }

    for (let id = 0; id < count; id++) {
      const worker = new Worker(WORKER_URL, { workerData: { id, log } });
      worker.addListener('message', done);
      workers.push(worker);
    }
  });
}

function stopWorkers() {
  return Promise.all(workers.map(worker => worker.terminate()));
}
