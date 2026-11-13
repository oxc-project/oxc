// @compilationMode:"infer"
function callLater(fn) {
  fn();
}

// `run` calls `push`, so passing `run` to an unknown function may push into
// `arr`. The array must be recreated whenever `props.x` changes.
function DirectCall(props) {
  const arr = [];
  const push = () => {
    arr.push(props.x);
  };
  const run = () => {
    push();
  };
  callLater(run);
  return <div>{arr}</div>;
}

// Calling `push` from a nested closure has the same effect.
function NestedCall(props) {
  const arr = [];
  const push = () => {
    arr.push(props.x);
  };
  const run = () => {
    [0].forEach(() => push());
  };
  callLater(run);
  return <div>{arr}</div>;
}

function Component(props) {
  return (
    <>
      <DirectCall x={props.x} />
      <NestedCall x={props.x} />
    </>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{x: 1}],
  sequentialRenders: [{x: 1}, {x: 2}, {x: 2}, {x: 3}],
};
