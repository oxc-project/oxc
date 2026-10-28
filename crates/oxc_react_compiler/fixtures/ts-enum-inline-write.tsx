function Component() {
  let count = 0;
  enum Counter {
    Value = count++,
  }

  return <span>{count}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
