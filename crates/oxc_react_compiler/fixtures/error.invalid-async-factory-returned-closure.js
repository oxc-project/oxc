function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const make = () => async () => { await 0; cb(); };
  const run = make();
  run();
  return null;
}
