function useFoo() {
  let count = 0;
  let cb = () => count++;
  cb = () => null;
  const wrapper = () => cb();
  return wrapper;
}
