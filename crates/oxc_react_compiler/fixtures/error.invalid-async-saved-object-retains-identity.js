function Component() {
  let count = 0;
  let opts = {cb: () => count++};
  const run = async () => { const saved = opts; await 0; saved.cb(); }; run();
  opts = {cb: () => null};
  return null;
}
