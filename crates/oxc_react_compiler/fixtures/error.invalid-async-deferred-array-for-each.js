function Component() {
  let count = 0;
  const callbacks = [() => null];
  const run = async () => { await 0; callbacks.forEach(cb => cb()); };
  run();
  callbacks[0] = () => count++;
  return null;
}
