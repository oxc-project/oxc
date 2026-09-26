function Component() {
  let count = 0;
  function make(fn) { return async () => { await 0; fn(); }; } function wrap(fn) { return make(fn); } wrap(() => count++)();
  return null;
}
