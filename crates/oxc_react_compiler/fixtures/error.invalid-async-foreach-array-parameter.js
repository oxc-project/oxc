function Component() {
  let count = 0;
  const callbacks = [() => count++]; callbacks.forEach(async (_, i, array) => { await 0; array[0](); });
  return null;
}
