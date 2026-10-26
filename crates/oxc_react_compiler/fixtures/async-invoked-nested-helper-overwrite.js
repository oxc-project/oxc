function Component() {
  let count = 0;
  let cb = () => count++;
  const run = async () => { const invoke = () => cb(); await 0; invoke(); };
  run();
  cb = () => null;
  return null;
}
