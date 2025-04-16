import { join as pathJoin } from 'node:path';
import { Worker } from 'node:worker_threads';
import { getSourceBuffer } from './util.js';
import { NO_DESER, NO_WALK, variantName, WALK } from './variants.js';

const WORKER_PATH = pathJoin(import.meta.dirname, 'worker.js');

const sourceBuffer = await getSourceBuffer();

await runWorkers(sourceBuffer, NO_WALK, 1, 1);
await runWorkers(sourceBuffer, NO_WALK, 8, 8);
await runWorkers(sourceBuffer, NO_WALK, 256, 4);
await runWorkers(sourceBuffer, NO_WALK, 384, 6);
await runWorkers(sourceBuffer, NO_WALK, 512, 8);
await runWorkers(sourceBuffer, NO_WALK, 768, 12);
await runWorkers(sourceBuffer, WALK, 512, 8);
await runWorkers(sourceBuffer, NO_DESER, 256, 1);
await runWorkers(sourceBuffer, NO_DESER, 1024, 4);
await runWorkers(sourceBuffer, NO_DESER, 1536, 6);
await runWorkers(sourceBuffer, NO_DESER, 2048, 8);
await runWorkers(sourceBuffer, NO_DESER, 3072, 12);

async function runWorkers(sourceBuffer, variant, iterations, threads) {
  const startTime = performance.now();

  const iterationsPerThread = iterations / threads;

  const promises = [];
  for (let i = 0; i < threads; i++) {
    const worker = new Worker(WORKER_PATH, {
      workerData: { iterations: iterationsPerThread, variant, sourceBuffer },
    });

    promises.push(
      new Promise((resolve, reject) => {
        worker.on('message', () => resolve());
        worker.on('error', reject);
        worker.on('exit', (code) => {
          if (code !== 0) reject(new Error(`Worker stopped with exit code ${code}`));
        });
      }),
    );
  }

  await Promise.all(promises);

  const endTime = performance.now();
  const totalTime = endTime - startTime;
  const iterationTime = totalTime / iterationsPerThread;

  console.log('----------');
  console.log(iterations, 'iterations in', threads, 'threads (', iterationsPerThread, 'iterations per thread )');
  console.log(variantName(variant));
  console.log('Total time:', totalTime, 'ms');
  console.log('Time per iteration:', iterationTime, 'ms');
  console.log('Iteration speed:', 1000 / iterationTime, 'Hz');
}
