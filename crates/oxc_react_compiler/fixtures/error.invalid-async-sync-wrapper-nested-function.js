function Component() {
  let count = 0;
  const cb = () => count++;
  const start = () => { const run = async () => { await 0; cb(); }; run(); }; start();
  return null;
}
