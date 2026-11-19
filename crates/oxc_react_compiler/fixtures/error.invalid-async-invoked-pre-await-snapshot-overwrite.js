function Component({cond}) {
  let count = 0;
  let cb = () => count++;
  const run = async () => { const saved = cb; await 0; saved(); };
  run(); cb = () => null;
  return null;
}
