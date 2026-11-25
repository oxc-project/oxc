function Component() {
  let count = 0;
  const opts = {cb: () => null};
  const run = async () => { const saved = opts; await 0; saved.cb(); }; run();
  opts.cb = () => count++;
  return null;
}
