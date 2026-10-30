function Component() {
  let count = 0;
  wrapper = () => null;
  function wrapper() {
    count++;
  }
  return () => wrapper();
}
