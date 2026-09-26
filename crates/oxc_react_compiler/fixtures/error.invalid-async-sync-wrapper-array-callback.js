function Component() {
  let count = 0;
  const cb = () => count++;
  const run = async fn => { await 0; fn(); }; [cb].forEach(value => run(value));
  return null;
}
