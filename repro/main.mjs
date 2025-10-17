import { dirname, join } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { Worker } from 'node:worker_threads';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const WORKER_COUNT = 16;
let done = 0;

console.log(`Starting ${ WORKER_COUNT } workers...\n`);

for (let i = 1; i <= WORKER_COUNT; i++) {
   const worker = new Worker(join(__dirname, 'worker.mjs'), {
      workerData: { workerId: i },
   });

   const id = String(i).padStart(2);

   worker.on('error', (error) => {
      console.error(`[Main] Worker ${ id } error:`, error);
   });

   worker.on('exit', (code) => {
      done++;
      if (code !== 0) {
         console.error(`[Main] Worker ${ id } stopped with exit code ${ code }`);
      }
      console.log(`Workers done: ${ done }/${ WORKER_COUNT }`);
      if (done === WORKER_COUNT) {
         console.log(`All workers done!`);
      }
   });
}

process.on('exit', (code) => {
   console.log(`About to exit with code: ${ code }`);
});

console.log(`\n${ WORKER_COUNT } workers created and running!\n`);
