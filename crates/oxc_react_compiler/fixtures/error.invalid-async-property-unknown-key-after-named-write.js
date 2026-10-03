function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const bad = async () => { await 0; cb(); };
  const obj = {run: async () => null};
  obj[key] = bad;
  obj.run();
  return null;
}
