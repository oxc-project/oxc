function Component() {
  let count = 0;
  const callbacks = [{cb: () => count++}]; callbacks.forEach(async ({cb}) => { await 0; cb(); });
  return null;
}
