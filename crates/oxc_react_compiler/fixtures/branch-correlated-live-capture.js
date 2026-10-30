function Component(cond) {
  let count = 0;
  let cb = () => null;
  let wrapper;
  if (cond) {
    wrapper = () => cb();
  } else {
    cb = () => count++;
    wrapper = () => null;
  }
  return wrapper;
}
