function Component() {
  let count = 0;
  let cb = () => null;
  const run = async () => { const saved = cb; await 0; saved(); };
  run(); cb = () => count++;
  return null;
}
