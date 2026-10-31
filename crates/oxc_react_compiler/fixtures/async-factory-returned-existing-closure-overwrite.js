function Component() {
  let count = 0; const cb = () => count++; let selected = async () => null; const make = () => selected; const run = make(); selected = async () => { await 0; cb(); }; run();
  return null;
}
