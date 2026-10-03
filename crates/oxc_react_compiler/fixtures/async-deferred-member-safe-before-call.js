function Component({cond}) {
  let count = 0;
  const opts = {cb: () => count++};
  opts.cb = () => null;
  const run = async () => { await 0; opts.cb(); }; run();
  return null;
}
