function Component({cond}) {
  let count = 0;
  const make = (unused, callback) => async () => { await 0; callback(); };
  const args = [() => null];
  const run = make(...args, () => count++); run();
  return null;
}
