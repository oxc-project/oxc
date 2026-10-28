function Component(props: {value: number}) {
  enum NumberValue {
    First = 1,
    Second = First,
  }

  return <span>{NumberValue.Second + props.value}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
};
