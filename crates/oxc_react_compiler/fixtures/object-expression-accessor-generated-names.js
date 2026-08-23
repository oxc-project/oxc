function Component({value}) {
  const first = {
    method(y) {
      return y;
    },
  };
  const x = value;
  const object = {
    get value() {
      return x;
    },
  };
  return <div>{object.value + first.method(1)}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 1}, {value: 2}],
};
