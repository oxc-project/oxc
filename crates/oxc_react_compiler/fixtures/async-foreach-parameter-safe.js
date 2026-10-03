function Component() {
  let count = 0;
  const callbacks = [() => null]; callbacks.forEach(async cb => { await 0; cb(); }, () => count++);
  return null;
}
