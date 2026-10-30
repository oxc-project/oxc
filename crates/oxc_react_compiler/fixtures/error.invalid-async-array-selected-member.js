function Component() {
  let count = 0;
  const cb = () => count++;
  const obj = [async () => { await 0; cb(); }, () => null];
  obj[0]();
  return null;
}
