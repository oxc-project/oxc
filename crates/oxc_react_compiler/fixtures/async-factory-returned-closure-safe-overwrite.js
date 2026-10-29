function Component({key, cond}) {
  let count = 0;
  let cb = () => count++;
  const make = () => async () => { await 0; cb(); };
  cb = () => null;
  const run = make();
  run();
  return null;
}
