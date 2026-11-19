function Component() {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[1](); }; }
  const run = new make(() => null, () => count++);
  run();
  return null;
}
