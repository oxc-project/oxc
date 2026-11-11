function Component({cond}) {
  let count = 0;
  let cb = () => null;
  const safeAsync = async () => null;
  const asyncUsingCb = async () => { await 0; cb(); };
  let run;
  if (cond) { run = safeAsync; } else { run = asyncUsingCb; }
  run();
  cb = () => count++;
  return null;
}
