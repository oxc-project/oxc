function Component({cond}) {
  let count = 0;
  let cb = () => count++;
  const run = async () => { await 0; cb(); };
  run(); cb = () => null;
  return null;
}
