function useFoo() {
  let count = 0;
  let cb = () => null;
  const wrapper = () => cb();
  cb = () => count++;
  return wrapper;
}
