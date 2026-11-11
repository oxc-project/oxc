function Component() {
  let count = 0;
  const cb = () => count++;
  const callbacks = new Set([cb]); callbacks.forEach = async () => {}; callbacks.forEach();
  return null;
}
