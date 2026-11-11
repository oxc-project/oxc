function Component({cond}) {
  let count = 0;
  const opts = {cb: () => null}; const alias = opts;
  const run = async () => { await 0; opts.cb(); }; run();
  alias.cb = () => count++;
  return null;
}
