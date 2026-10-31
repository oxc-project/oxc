// The `try` body cannot throw, so the `catch` is pruned. The code after the `try`
// must survive even though no variable needs a phi at the join.
function Component(props) {
  try {
    const unused = 1;
  } catch {
    if (props.cond) {
      props.onError();
    } else {
      props.onFallback();
    }
  }
  return <div>{props.value}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 'a', cond: true}],
  sequentialRenders: [
    {value: 'a', cond: true},
    {value: 'b', cond: false},
  ],
};
