function createValue(value: number): number {
  return value;
}

function Component(props: {value: number}) {
  enum Unused {
    Value = createValue(props.value),
  }

  return <span>{props.value}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
};
