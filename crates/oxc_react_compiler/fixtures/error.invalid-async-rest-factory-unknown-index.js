function Component({index}) {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks[index](); }; }
  const run = make(() => null, () => count++);
  run();
  return null;
}
