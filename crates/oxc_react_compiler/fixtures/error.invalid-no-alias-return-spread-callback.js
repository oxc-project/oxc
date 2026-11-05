function Component() {
  let count = 0;
  const callbacks = [() => () => count++, () => null]; return [0].map(...callbacks);
}
