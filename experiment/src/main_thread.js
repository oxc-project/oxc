import { getSourceBuffer } from './util.js';
import run from './impl.js';

const sourceBuffer = await getSourceBuffer();
const sourceText = new TextDecoder().decode(sourceBuffer);

runSingleThread(false, 1);
runSingleThread(false, 8);
runSingleThread(false, 64);
runSingleThread(true, 64);

function runSingleThread(shouldWalk, iterations) {
  const startTime = performance.now();

  run(sourceText, sourceBuffer, iterations, shouldWalk);

  const endTime = performance.now();
  const totalTime = endTime - startTime;
  const iterationTime = totalTime / iterations;

  console.log('----------');
  console.log(iterations, 'iterations in main thread');
  console.log(shouldWalk ? 'With' : 'Without', 'walk.');
  console.log('Total time:', totalTime, 'ms');
  console.log('Time per iteration:', iterationTime, 'ms');
  console.log('Iteration speed:', 1000 / iterationTime, 'Hz');
}
