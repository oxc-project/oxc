function Component() {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[0](); }; }
  const run = make.apply(null, [() => null, () => count++]);
  run();
  return null;
}
