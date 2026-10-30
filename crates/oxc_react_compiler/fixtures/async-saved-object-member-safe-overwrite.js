function Component() {
  let count = 0;
  const opts = {cb: () => count++};
  const run = async () => { const saved = opts; await 0; saved.cb(); }; run();
  opts.cb = () => null;
  return null;
}
