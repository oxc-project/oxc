import { parentPort, workerData } from 'node:worker_threads';
import { registerWorker } from '../bindings.js';
import workload from './workload.js';

const { id: workerId, log } = workerData;

if (log) console.log('> Booting worker', workerId);

registerWorker(workerId, workload.bind(null, workerId));

parentPort.postMessage('');
