import run from './impl.js';
import { getSourceBuffer } from './util.js';
import { NO_DESER, NO_WALK, variantName, WALK } from './variants.js';

const sourceBuffer = await getSourceBuffer();
const sourceText = new TextDecoder().decode(sourceBuffer);

runSingleThread(NO_WALK, 1);
runSingleThread(NO_WALK, 8);
runSingleThread(NO_WALK, 64);
runSingleThread(WALK, 64);
runSingleThread(NO_DESER, 1);
runSingleThread(NO_DESER, 8);
runSingleThread(NO_DESER, 64);

function runSingleThread(variant, iterations) {
  const startTime = performance.now();

  run(sourceText, sourceBuffer, iterations, variant);

  const endTime = performance.now();
  const totalTime = endTime - startTime;
  const iterationTime = totalTime / iterations;

  console.log('----------');
  console.log(iterations, 'iterations in main thread');
  console.log(variantName(variant));
  console.log('Total time:', totalTime, 'ms');
  console.log('Time per iteration:', iterationTime, 'ms');
  console.log('Iteration speed:', 1000 / iterationTime, 'Hz');
}
