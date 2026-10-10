function Component() {
  let count = 0;
  const run = async cb => { await 0; cb(); }; const bound = run.bind(null, () => count++); bound(); return null;
}
