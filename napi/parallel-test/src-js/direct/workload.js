const DURATION = 100;

export default function(workerId) {
  console.log('> Start job on JS worker', workerId);

  // Eat up the CPU for some time
  const endTime = Date.now() + DURATION;
  while (Date.now() < endTime) {}

  console.log('> Finished job on JS worker', workerId);
}
