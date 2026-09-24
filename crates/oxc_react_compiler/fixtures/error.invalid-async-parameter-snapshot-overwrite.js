function Component() {
  let count = 0;
  let cb = () => count++; const run = async cb => { await 0; cb(); }; run(cb); cb = () => null; return null;
}
