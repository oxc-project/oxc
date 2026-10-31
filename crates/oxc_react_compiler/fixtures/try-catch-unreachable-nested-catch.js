// Neither `try` body can throw, so both `catch` blocks are pruned.
function Component(props) {
  let x = props.value;
  try {
    try {
      const unused = 1;
    } catch {
      x = 1;
    }
  } catch {
    x = 2;
  }
  return <div>{x}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 0}, {value: 3}],
};
