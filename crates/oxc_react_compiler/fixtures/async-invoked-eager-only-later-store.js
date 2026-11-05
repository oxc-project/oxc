function Component() {
  let count = 0;
  let cb = () => null;
  const run = async () => { cb(); await 0; };
  run(); cb = () => count++;
  return null;
}
