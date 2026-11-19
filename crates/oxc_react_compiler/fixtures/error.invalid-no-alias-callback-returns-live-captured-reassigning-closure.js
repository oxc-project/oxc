function Component() {
  let count = 0;
  let cb = () => null;
  const xs = [0].map(() => () => cb());
  cb = () => count++;
  return xs;
}
