function Component() {
  return (() => {
    let count = 0;
    return () => count++;
  })();
}
