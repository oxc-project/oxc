import { parentPort, workerData } from 'node:worker_threads';
import { registerWorker } from '../bindings.js';
import { setLog, workload } from './workload.js';

const { id: workerId, log } = workerData;

if (log) console.log('> Booting worker', workerId);

setLog(log);
registerWorker(workerId, workload);

parentPort.postMessage('');
