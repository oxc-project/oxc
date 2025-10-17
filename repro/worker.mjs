import { workerData } from 'node:worker_threads';

const workerId = String(workerData.workerId);

console.log(`Worker ${ workerId.padStart(2) } starting ...`);

try {
   // When running this import, the worker process fails, which leads to a crash of the main process.
   // If you comment out the import, the worker process runs successfully.
   const module = await import('oxc-parser');
} catch (error) {
   console.error(error);
}

setTimeout(() => {
   console.log(`Hello from worker ${ workerId }`);
}, Math.random() * 1000);
