function Component() {
  let count = 0;
  const arg = {cb: () => null}; const run = async ({cb}) => { await 0; cb(); }; run(arg); arg.cb = () => count++;
  return null;
}
