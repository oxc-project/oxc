function Component(props: {value: number}) {
  function Inner() {
    enum NumberValue {
      Value = props.value,
    }

    return <span>{NumberValue.Value}</span>;
  }

  return <Inner />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
};
