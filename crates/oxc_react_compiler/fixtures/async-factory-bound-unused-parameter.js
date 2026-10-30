function Component({cond}) {
  let count = 0;
  const make = (unused, callback) => async () => { await 0; callback(); };
  const bound = make.bind(null, () => count++);
  const run = bound(() => null); run();
  return null;
}
