function Component() {
  let count = 0;
  const run = async ({safe}) => { await 0; safe(); }; run({safe: () => null, bad: () => count++});
  return null;
}
