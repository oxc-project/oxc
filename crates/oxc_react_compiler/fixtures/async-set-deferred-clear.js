function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Set([cb]); const run = async () => { await 0; callbacks.forEach(fn => fn()); }; run(); callbacks.clear();
  return null;
}
