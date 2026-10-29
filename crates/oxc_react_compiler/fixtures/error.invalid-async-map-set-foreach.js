function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Map(); callbacks.set("key", cb).forEach(async value => { await 0; value(); });
  return null;
}
