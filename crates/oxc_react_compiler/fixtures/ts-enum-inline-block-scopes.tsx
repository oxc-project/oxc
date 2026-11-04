function Component() {
  let first;
  let second;

  {
    enum NumberValue {
      Value = 1,
    }
    first = NumberValue;
  }

  {
    enum NumberValue {
      Value = 2,
    }
    second = NumberValue;
  }

  return <span>{first === second ? 'same' : 'different'}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
