function Component() {
  let count = 0;
  return [0].map(() => {
    const bad = () => count++;
    if (bad) {
      return 1;
    }
    return 2;
  });
}
