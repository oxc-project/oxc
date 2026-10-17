function Component({cond}) {
  let count = 0;
  const opts = {cb: () => null, other: () => count++};
  const run = async () => { await 0; opts.cb(); }; run();
  return null;
}
