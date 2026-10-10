function Component() {
  let count = 0;
  let cb = () => count++;
  const wrapper = async () => cb();
  cb = () => null;
  return wrapper;
}
