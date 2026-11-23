function useFoo() {
  let count = 0;
  let cb = () => count++;
  cb++;
  return cb;
}

export const FIXTURE_ENTRYPOINT = {
  fn: useFoo,
  params: [],
  isComponent: false,
};
