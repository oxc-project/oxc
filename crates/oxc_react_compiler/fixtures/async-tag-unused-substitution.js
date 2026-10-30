function Component() {
  let count = 0;
  const tag = async (strings, safe) => { await 0; safe(); }; tag`${() => null}${() => count++}`;
  return null;
}
