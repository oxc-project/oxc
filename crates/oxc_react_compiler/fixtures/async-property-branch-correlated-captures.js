function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const safeAsync = async () => null;
  const asyncUsingCb = async () => { await 0; cb(); };
  const obj = {};
  if (cond) { cb = () => count++; obj.run = safeAsync; }
  else { obj.run = asyncUsingCb; }
  obj.run();
  return null;
}
