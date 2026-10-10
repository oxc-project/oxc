function Component() {
  let count = 0;
  const mutate = () => count++;
  const bad = async () => { await 0; mutate(); }; const tag = (strings, fn) => fn; const run = tag`${bad}`; run();
  return null;
}
