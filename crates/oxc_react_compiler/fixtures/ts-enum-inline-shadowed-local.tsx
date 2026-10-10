function Component(value: number) {
  {
    const value = 42;
    enum NumberValue {
      Value = value,
    }

    return <span>{NumberValue.Value}</span>;
  }
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [0],
};
