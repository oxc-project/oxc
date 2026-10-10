function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Map([["key", cb]]); callbacks.forEach(async value => { await 0; value(); });
  return null;
}
