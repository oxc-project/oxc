function useFoo(condition) {
  let count = 0;
  let cb = () => null;
  let run = condition;
  while (run) {
    cb = () => count++;
    run = false;
  }
  return cb;
}
