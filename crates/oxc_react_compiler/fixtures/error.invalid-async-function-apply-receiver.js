function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const run = async () => { await 0; cb(); };
  run.apply(null, []);
  return null;
}
