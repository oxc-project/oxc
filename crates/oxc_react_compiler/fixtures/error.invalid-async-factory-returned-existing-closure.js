function Component() {
  let count = 0; const cb = () => count++; const asyncFn = async () => { await 0; cb(); }; const make = () => asyncFn; const run = make(); run();
  return null;
}
