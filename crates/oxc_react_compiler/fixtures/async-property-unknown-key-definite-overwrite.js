function Component({key, cond}) {
  let count = 0;
  const bad = async () => { await 0; cb(); };
  const cb = () => count++;
  const safe = async () => null;
  const obj = {};
  obj[key] = bad;
  obj.run = safe;
  obj.run();
  return null;
}
