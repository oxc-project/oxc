import { parentPort, workerData } from 'node:worker_threads';
import { registerWorker } from '../bindings.js';
import { setWorkerIdAndLog, storeBuffer, workload } from './workload.js';

const { id: workerId, log } = workerData;

if (log) console.log('> Booting worker', workerId);

setWorkerIdAndLog(workerId, log);
registerWorker(workerId, storeBuffer, workload);

parentPort.postMessage('');
