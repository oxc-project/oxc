function Component() {
  let count = 0;
  let opts = {cb: () => null};
  const run = async () => { const saved = opts; await 0; saved.cb(); }; run();
  opts = {cb: () => count++};
  return null;
}
