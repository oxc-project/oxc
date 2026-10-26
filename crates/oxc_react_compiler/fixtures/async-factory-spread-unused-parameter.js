function Component({cond}) {
  let count = 0;
  const make = (unused, callback) => async () => { await 0; callback(); };
  const args = [() => count++];
  const run = make(...args, () => null); run();
  return null;
}
