function Component() {
  let count = 0;
  const mutate = () => count++;
  let make = fn => fn; const wrap = fn => make(fn); make = fn => async () => null; wrap(async () => { await 0; mutate(); })();
  return null;
}
