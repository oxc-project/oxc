function Component() {
  let count = 0;
  let cb = () => count++;
  const wrapper = async () => cb();
  return wrapper;
}
