function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const run = async () => {
    await 0;
    cb();
  };
  if (cond) {
    run();
    cb = () => count++;
  }
  return null;
}
