function Component() {
  let count = 0;
  const cb = () => count++;
  let selected = cb; const run = async function* () { await 0; selected(); }; const iterator = run(); selected = () => {}; iterator.next();
  return null;
}
