function Component() {
  let count = 0;
  const cb = () => count++;
  let selected = cb; const run = async function* (fn) { await 0; fn(); }; const iterator = run(selected); selected = () => {}; iterator.next();
  return null;
}
