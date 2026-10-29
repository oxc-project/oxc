function Component() {
  let count = 0;
  const callbacks = [() => count++]; callbacks.filter(async cb => { await 0; cb(); });
  return null;
}
