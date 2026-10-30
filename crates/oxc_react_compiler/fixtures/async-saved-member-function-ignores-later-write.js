function Component() {
  let count = 0;
  const opts = {cb: () => null};
  const run = async () => { const saved = opts.cb; await 0; saved(); }; run();
  opts.cb = () => count++;
  return null;
}
