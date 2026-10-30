function Component() {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[2](); }; }
  const args = [() => null, () => count++];
  const copied = [() => count++, ...args, () => null];
  const run = make(...copied); run();
  return null;
}
