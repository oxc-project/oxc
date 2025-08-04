// ID of this worker
let workerId;

// `true` if logging is enabled
let log = false;

// Buffer used to transfer ASTs
let buffer;

/**
 * Store flag for whether logging is enabled.
 * @param {number} id - Worker ID
 * @param {boolean} shouldLog - `true` if logging is enabled
 */
export function setWorkerIdAndLog(id, shouldLog) {
  workerId = id;
  log = shouldLog;
}

/**
 * Store buffer.
 * @param {Uint8Array} uint8Array - Buffer
 */
export function storeBuffer(uint8Array) {
  if (log) console.log('> Received buffer on JS worker', workerId);
  buffer = uint8Array;
  const { buffer: arrayBuffer, byteOffset } = buffer;
  buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
  buffer.float64 = new Float64Array(arrayBuffer, byteOffset);
}

/**
 * Run workload.
 * @param {number} duration - Microseconds to work for
 */
export function workload(duration) {
  if (log) console.log('> Start job on JS worker', workerId, '-', duration, 'micros');

  // Eat up the CPU for some time
  const endTime = performance.now() + (duration / 1000);
  while (performance.now() < endTime) {}

  if (log) console.log('> Finished job on JS worker', workerId);
}
