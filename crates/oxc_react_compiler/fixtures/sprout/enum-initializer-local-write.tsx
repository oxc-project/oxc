function Component() {
  let count = 0;
  enum Counter {
    Value = (count += 2),
  }
  return <span>{count}</span>;
}

export const FIXTURE_ENTRYPOINT = {fn: Component, params: []};
