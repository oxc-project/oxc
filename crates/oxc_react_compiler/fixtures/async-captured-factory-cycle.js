function Component() {
  let count = 0;
  function make(fn) { return wrap(fn); } function wrap(fn) { return make(fn); } wrap(async () => null);
  return null;
}
