function Component() {
  let count = 0;
  let cb = () => null;
  const wrapper = async () => cb();
  cb = () => count++;
  return wrapper;
}
