function Component() {
  let count = 0;
  let cb = () => null;
  [0].filter(async () => {
    await 0;
    cb();
  });
  cb = () => count++;
  return null;
}
