function Component() {
  let count = 0;
  const mutate = () => count++;
  function make(safe, bad) { return safe; } function wrap(safe, bad) { return make(safe, bad); } wrap(async () => null, async () => { await 0; mutate(); })();
  return null;
}
