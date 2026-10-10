function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async function* () { await 0; cb(); }; const iterator = run(); iterator.next = () => {}; iterator.next();
  return null;
}
