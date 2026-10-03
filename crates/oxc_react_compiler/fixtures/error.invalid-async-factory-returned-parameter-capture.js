function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const make = (callback) => async () => { await 0; callback(); };
  const run = make(cb);
  run();
  return null;
}
