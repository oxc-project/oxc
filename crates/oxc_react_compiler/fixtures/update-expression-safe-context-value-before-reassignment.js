function useFoo() {
  let count = 0;
  let cb = () => null;
  const result = cb;
  [0].map(() => cb);
  cb = () => count++;
  return result;
}
