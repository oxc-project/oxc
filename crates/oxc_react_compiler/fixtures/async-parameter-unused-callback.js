function Component() {
  let count = 0;
  const run = async (unused, cb) => { await 0; cb(); }; run(() => count++, () => null); return null;
}
