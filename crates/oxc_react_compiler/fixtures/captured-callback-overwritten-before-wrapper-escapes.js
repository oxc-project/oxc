function useFoo() {
  let count = 0;
  let cb = () => count++;
  const wrapper = () => cb();
  cb = () => null;
  return wrapper;
}
