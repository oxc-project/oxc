function useFoo() {
  let count = 0;
  let cb = () => count++;
  return cb++;
}

export const FIXTURE_ENTRYPOINT = {
  fn: useFoo,
  params: [],
  isComponent: false,
};
