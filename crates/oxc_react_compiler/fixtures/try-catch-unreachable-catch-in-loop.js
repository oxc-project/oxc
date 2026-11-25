// A `try` that cannot throw inside a loop: the `catch` is pruned on every iteration.
function Component(props) {
  let x = 0;
  for (let i = 0; i < props.count; i++) {
    try {
      const unused = 1;
    } catch {
      x = i;
    }
  }
  return <div>{x}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{count: 3}],
  sequentialRenders: [{count: 3}, {count: 0}],
};
