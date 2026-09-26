function Component() {
  let count = 0;
  const cb = () => count++;
  const run = () => cb(); const start = () => run(); start();
  return null;
}
