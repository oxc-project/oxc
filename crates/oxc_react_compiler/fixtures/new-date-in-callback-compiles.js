// @validateNoImpureFunctionsInRender
// A component with zero-argument `new Date()` only inside a callback passed to an unknown
// function compiles, because the callback is not reported as running during render.
function wrapHandler(handler) {
  return (event) => handler(event);
}

function Component(props) {
  const onSubmit = (data) => {
    props.onCreate({ ...data, createdAt: new Date() });
  };
  return (
    <form onSubmit={wrapHandler(onSubmit)}>
      <span>{props.label}</span>
    </form>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{ label: "a", onCreate: () => {} }],
  sequentialRenders: [
    { label: "a", onCreate: () => {} },
    { label: "a", onCreate: () => {} },
    { label: "b", onCreate: () => {} },
  ],
};
