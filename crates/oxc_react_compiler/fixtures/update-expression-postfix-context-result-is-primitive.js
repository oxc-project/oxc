function useFoo() {
  let count = 0;
  let cb = () => count++;
  return cb++;
}
