function Component({key, cond}) {
  let count = 0;
  let cb = () => null;
  const make = () => async () => { await 0; cb(); };
  const run = make();
  cb = () => count++;
  run();
  return null;
}
