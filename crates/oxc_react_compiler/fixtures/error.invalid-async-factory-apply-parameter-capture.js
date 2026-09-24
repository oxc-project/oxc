function Component() {
  let count = 0;
  const cb = () => count++;
  const make = callback => async () => { await 0; callback(); };
  const run = make.apply(null, [cb]);
  run();
  return null;
}
