function Component() {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[1](); }; }
  const args = [() => null, () => count++];
  const copied = [...args];
  const run = make(...copied); run();
  return null;
}
