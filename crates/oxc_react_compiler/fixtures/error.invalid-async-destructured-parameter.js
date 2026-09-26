function Component() {
  let count = 0;
  const run = async ({cb}) => { await 0; cb(); }; run({cb: () => count++});
  return null;
}
