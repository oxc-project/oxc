function Component() {
  let count = 0;
  const tag = async (strings, cb) => { await 0; cb(); }; tag`${() => count++}`;
  return null;
}
