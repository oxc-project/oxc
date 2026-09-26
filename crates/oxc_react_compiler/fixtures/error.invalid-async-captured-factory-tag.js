function Component() {
  let count = 0;
  const mutate = () => count++;
  function make(strings, fn) { return fn; } function wrap(fn) { return make`${fn}`; } wrap(async () => { await 0; mutate(); })();
  return null;
}
