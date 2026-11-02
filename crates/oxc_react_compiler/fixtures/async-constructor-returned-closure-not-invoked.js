function Component() {
  let count = 0;
  const cb = () => count++;
  function Factory() { return async () => { await 0; cb(); }; }
  const run = new Factory();
  return null;
}
