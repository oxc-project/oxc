/**
 * Run workload.
 * @param {number} workerId - Worker ID
 * @param {number} duration - Microseconds to work for
 * @param {boolean} log - `true` if logging is enabled
 */
export default function(workerId, duration, log) {
  if (log) console.log('> Start job on JS worker', workerId, '-', duration, 'micros');

  // Eat up the CPU for some time
  const endTime = performance.now() + (duration / 1000);
  while (performance.now() < endTime) {}

  if (log) console.log('> Finished job on JS worker', workerId);
}
