function Component({cond}) {
  let count = 0;
  const cb = () => count++;
  const obj = {run: async () => { await 0; cb(); }};
  const run = obj.run;
  obj.run = () => null;
  run();
  return null;
}
