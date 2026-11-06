function Component() {
  let count = 0;
  const callbacks = [() => count++]; callbacks.forEach(async cb => { await 0; cb(); });
  return null;
}
