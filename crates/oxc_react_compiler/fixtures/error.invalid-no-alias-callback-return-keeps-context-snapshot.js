function Component() {
  let count = 0;
  let cb = () => count++;
  const xs = [0].map(() => cb);
  cb = () => null;
  return xs;
}
