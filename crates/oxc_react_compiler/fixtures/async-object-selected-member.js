function Component() {
  let count = 0;
  const cb = () => count++;
  const obj = {bad: async () => { await 0; cb(); }, safe() {}};
  obj.safe();
  return null;
}
