function Component() {
  let count = 0;
  const cb = () => count++;
  let selected = cb; const run = async () => { await 0; selected(); }; const start = () => run(); start(); selected = () => {};
  return null;
}
