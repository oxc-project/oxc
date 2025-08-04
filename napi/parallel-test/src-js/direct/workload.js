// ID of this worker
let workerId;

// `true` if logging is enabled
let log = false;

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
