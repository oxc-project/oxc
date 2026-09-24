function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async function* () { await 0; count++; }; run();
  return null;
}
