function Component() {
  let count = 0;
  const cb = () => wrapper();
  function wrapper() {
    count++;
  }
  return cb;
}
