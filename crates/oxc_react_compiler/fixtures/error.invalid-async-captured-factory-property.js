function Component() {
  let count = 0;
  const mutate = () => count++;
  const factories = {make: fn => fn}; const wrap = fn => factories.make(fn); wrap(async () => { await 0; mutate(); })();
  return null;
}
