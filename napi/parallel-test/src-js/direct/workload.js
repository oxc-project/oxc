// `true` if logging is enabled
let log = false;

/**
 * Store flag for whether logging is enabled.
 * @param {boolean} shouldLog - `true` if logging is enabled
 */
export function setLog(shouldLog) {
  log = shouldLog;
}

/**
 * Run workload.
 * @param {number} workerId - Worker ID
 * @param {number} duration - Microseconds to work for
 */
export function workload(workerId, duration) {
  if (log) console.log('> Start job on JS worker', workerId, '-', duration, 'micros');

  // Eat up the CPU for some time
  const endTime = performance.now() + (duration / 1000);
  while (performance.now() < endTime) {}

  if (log) console.log('> Finished job on JS worker', workerId);
}
