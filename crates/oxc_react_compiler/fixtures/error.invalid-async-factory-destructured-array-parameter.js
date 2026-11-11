function Component({cond}) {
  let count = 0;
  const make = ([unused, callback]) => async () => { await 0; callback(); };
  const run = make([() => null, () => count++]); run();
  return null;
}
