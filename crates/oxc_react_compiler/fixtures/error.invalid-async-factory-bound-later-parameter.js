function Component({cond}) {
  let count = 0;
  const make = (unused, callback) => async () => { await 0; callback(); };
  const bound = make.bind(null, () => null);
  const run = bound(() => count++); run();
  return null;
}
