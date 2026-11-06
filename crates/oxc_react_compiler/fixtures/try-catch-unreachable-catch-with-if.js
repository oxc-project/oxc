// The `try` body cannot throw, so the `catch` (and the `if` inside it) is pruned.
function Component(props) {
  let x = props.value;
  try {
    const unused = 1;
  } catch {
    if (props.cond) {
      x = 1;
    } else {
      x = 2;
    }
  }
  return <div>{x}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0, cond: true}],
  sequentialRenders: [
    {value: 0, cond: true},
    {value: 1, cond: false},
    {value: 1, cond: true},
  ],
};
