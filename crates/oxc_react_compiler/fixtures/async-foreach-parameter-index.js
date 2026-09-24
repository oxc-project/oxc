function Component() {
  let count = 0;
  const callbacks = [() => count++]; callbacks.forEach(async (_, index) => { await 0; return index; });
  return null;
}
