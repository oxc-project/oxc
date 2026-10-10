function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const bad = async () => { await 0; cb(); };
  const obj = {};
  obj[key] = bad;
  if (cond) { obj.run = async () => null; }
  obj.run();
  return null;
}
