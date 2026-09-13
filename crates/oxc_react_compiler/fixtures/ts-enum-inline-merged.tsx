function Component() {
  enum NumberValue {
    First = 1,
  }
  enum NumberValue {
    Second = 2,
  }

  return <span>{NumberValue.First + NumberValue.Second}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
