function Component({cond}) {
  let count = 0;
  const cb = () => count++;
  const obj = {run: async () => { await 0; cb(); }, safe() {}};
  const {safe} = obj;
  safe();
  return null;
}
