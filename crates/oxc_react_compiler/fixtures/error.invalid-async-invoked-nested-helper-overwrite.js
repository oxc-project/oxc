function Component() {
  let count = 0;
  let cb = () => count++;
  const run = async () => { const invoke = () => cb(); invoke(); await 0; };
  run();
  cb = () => null;
  return null;
}
