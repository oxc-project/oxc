function Component() {
  let count = 0;
  const factory = () => () => count++;
  return [0].map(x => x, factory);
}
