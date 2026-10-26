function Component() {
  let count = 0;
  const mutate = () => count++;
  const tag = (strings, safe) => safe; const run = tag`${async () => null}${async () => { await 0; mutate(); }}`; run();
  return null;
}
