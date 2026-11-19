function Component() {
  let count = 0;
  const arg = {cb: () => count++}; const run = async ({cb}) => { await 0; cb(); }; run(arg); arg.cb = () => null;
  return null;
}
