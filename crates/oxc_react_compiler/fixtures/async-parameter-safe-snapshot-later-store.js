function Component() {
  let count = 0;
  let cb = () => null; const run = async cb => { await 0; cb(); }; run(cb); cb = () => count++; return null;
}
