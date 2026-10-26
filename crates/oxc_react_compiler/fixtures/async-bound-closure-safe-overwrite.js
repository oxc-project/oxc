function Component() {
  let count = 0;
  let cb = () => count++;
  const run = async () => { await 0; cb(); };
  const bound = run.bind(null); cb = () => null; bound();
  return null;
}
