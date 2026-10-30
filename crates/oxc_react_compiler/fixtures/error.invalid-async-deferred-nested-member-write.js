function Component({cond}) {
  let count = 0;
  const opts = {nested: {cb: () => null}};
  const run = async () => { await 0; opts.nested.cb(); }; run();
  opts.nested.cb = () => count++;
  return null;
}
