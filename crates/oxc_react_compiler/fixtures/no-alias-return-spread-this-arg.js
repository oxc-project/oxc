function Component() {
  let count = 0;
  const callbacks = [() => null, () => () => count++]; return [0].map(...callbacks);
}
