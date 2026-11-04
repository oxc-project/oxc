function Component() {
  let count = 0;
  return [0].map(() => () => count++);
}
