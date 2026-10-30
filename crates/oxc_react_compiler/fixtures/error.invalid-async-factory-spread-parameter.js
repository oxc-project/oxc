function Component({cond}) {
  let count = 0;
  const make = callback => async () => { await 0; callback(); };
  const args = [() => count++];
  const run = make(...args); run();
  return null;
}
