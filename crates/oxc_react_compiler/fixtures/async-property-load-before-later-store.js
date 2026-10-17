function Component({cond}) {
  let count = 0;
  const cb = () => count++;
  const obj = {run() {}};
  const run = obj.run;
  obj.run = async () => { await 0; cb(); };
  run();
  return null;
}
