function Component() {
  let count = 0;
  const run = async ({nested: {cb}}) => { await 0; cb(); }; run({nested: {cb: () => count++}});
  return null;
}
