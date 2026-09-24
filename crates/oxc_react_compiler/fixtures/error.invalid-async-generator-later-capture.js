function Component() {
  let count = 0;
  const cb = () => count++;
  let selected = () => {}; const run = async function* () { await 0; selected(); }; const iterator = run(); selected = cb; iterator.next();
  return null;
}
