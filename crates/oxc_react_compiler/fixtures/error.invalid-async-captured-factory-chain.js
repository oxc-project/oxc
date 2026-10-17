function Component() {
  let count = 0;
  const mutate = () => count++;
  function make(fn) { return fn; } function wrap(fn) { return make(fn); } function outer(fn) { return wrap(fn); } outer(async () => { await 0; mutate(); })();
  return null;
}
