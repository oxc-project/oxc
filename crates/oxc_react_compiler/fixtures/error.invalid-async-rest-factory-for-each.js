function Component({cond}) {
  let count = 0;
  function make(...callbacks) { return async () => { await 0; callbacks.forEach(callback => callback()); }; }
  const run = make(() => count++); run();
  return null;
}
