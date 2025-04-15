import { parentPort, workerData } from 'node:worker_threads';
import run from './impl.js';

const { iterations, shouldWalk, sourceBuffer } = workerData;
const sourceText = new TextDecoder().decode(sourceBuffer);

run(sourceText, sourceBuffer, iterations, shouldWalk);

parentPort.postMessage(null);
