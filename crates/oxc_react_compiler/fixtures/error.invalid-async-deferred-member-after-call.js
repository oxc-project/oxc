function Component({cond}) {
  let count = 0;
  const opts = {cb: () => null};
  const run = async () => { await 0; opts.cb(); }; run();
  opts.cb = () => count++;
  return null;
}
