function Component({cond}) {
  let count = 0;
  const opts = {cb: () => count++};
  const run = async () => { await 0; opts.cb(); }; run();
  opts.cb = () => null;
  return null;
}
