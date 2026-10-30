function Component() {
  let count = 0;
  const mutate = () => count++;
  function wrap(fn) { return make(fn); } function make(fn) { return fn; } wrap(async () => { await 0; mutate(); })();
  return null;
}
