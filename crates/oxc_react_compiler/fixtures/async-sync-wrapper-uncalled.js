function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async () => { await 0; cb(); }; const start = () => run();
  return null;
}
