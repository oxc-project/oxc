function useFoo() {
  let count = 0;
  let cb = () => count++;
  cb++;
  return cb;
}
