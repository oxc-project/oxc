function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const run = async () => {
    await 0;
    cb();
  };
  run();
  cb = () => count++;
  if (cond) {
    cb = () => null;
  }
  return null;
}
