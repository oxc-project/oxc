function Component() {
  let count = 0;
  const run = async cb => { cb = () => null; await 0; cb(); }; run(() => count++);
  return null;
}
