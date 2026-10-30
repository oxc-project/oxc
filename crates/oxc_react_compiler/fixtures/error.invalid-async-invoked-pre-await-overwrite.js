function Component({cond}) {
  let count = 0;
  let cb = () => count++;
  const run = async () => { cb(); await 0; };
  run(); cb = () => null;
  return null;
}
