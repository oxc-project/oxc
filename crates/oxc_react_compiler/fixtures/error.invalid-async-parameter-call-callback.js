function Component() {
  let count = 0;
  const run = async cb => { await 0; cb(); }; run.call(null, () => count++); return null;
}
