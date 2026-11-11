function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const safeAsync = async () => null;
  const asyncUsingCb = async () => { await 0; cb(); };
  let run;
  if (cond) { cb = () => count++; run = asyncUsingCb; }
  else { run = safeAsync; }
  cb = () => null;
  run();
  return null;
}
