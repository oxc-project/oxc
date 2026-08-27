type Model = {value: number};

function Component({value}: {value: number}) {
  const identity = (x: number) => x;
  const x = value;
  const object = {
    get value(this: Model): typeof x {
      return x;
    },
    set value(this: Model, next: number) {},
  };
  return <div>{identity(object.value)}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 1}, {value: 2}],
};
