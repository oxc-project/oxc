function Component() {
  let count = 0;
  let cb = () => null;
  const run = async () => { await 0; cb(); };
  const bound = run.bind(null); cb = () => count++; bound();
  return null;
}
