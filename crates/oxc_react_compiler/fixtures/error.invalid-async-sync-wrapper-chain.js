function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async () => { await 0; cb(); }; const first = () => run(); const second = () => first(); second();
  return null;
}
