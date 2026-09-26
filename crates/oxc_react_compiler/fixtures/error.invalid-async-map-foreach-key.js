function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Map([[cb, () => {}]]); callbacks.forEach(async (_, key) => { await 0; key(); });
  return null;
}
