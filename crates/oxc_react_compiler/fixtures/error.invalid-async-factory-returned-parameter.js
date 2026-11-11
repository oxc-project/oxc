function Component() {
  let count = 0; const cb = () => count++; const asyncFn = async () => { await 0; cb(); }; const make = (callback) => callback; const run = make(asyncFn); run();
  return null;
}
