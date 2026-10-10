function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async fn => { await 0; fn(); }; const start = (fn, unused) => fn(() => {}); start(run, cb);
  return null;
}
