function Component() {
  let count = 0;
  const cb = () => count++;
  const make = callback => async () => { await 0; callback(); };
  const run = make.call(null, cb);
  run();
  return null;
}
