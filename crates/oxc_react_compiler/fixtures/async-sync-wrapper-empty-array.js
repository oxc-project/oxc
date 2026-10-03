function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async fn => { await 0; fn(); }; [].forEach(value => run(value));
  return null;
}
