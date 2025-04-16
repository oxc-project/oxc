import { parentPort, workerData } from 'node:worker_threads';
import run from './impl.js';

const { sourceBuffer, iterations, variant } = workerData;
const sourceText = new TextDecoder().decode(sourceBuffer);

run(sourceText, sourceBuffer, iterations, variant);

parentPort.postMessage(null);
