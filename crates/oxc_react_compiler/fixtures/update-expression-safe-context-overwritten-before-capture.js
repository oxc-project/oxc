function useFoo() {
  let count = 0;
  let cb = () => count++;
  cb = () => null;
  const wrapper = () => cb();
  return wrapper;
}

export const FIXTURE_ENTRYPOINT = {
  fn: useFoo,
  params: [],
  isComponent: false,
};
