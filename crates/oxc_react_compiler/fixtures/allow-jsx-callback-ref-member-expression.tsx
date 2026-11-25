// @validateRefAccessDuringRender

// Callback refs read from an object, as in the floating-ui `useFloating` docs.
const callbacks = {
  refs: {setReference() {}, setFloating() {}},
  floatingStyles: {position: 'absolute'},
};

function useFloating() {
  return callbacks;
}

function App() {
  const {refs, floatingStyles} = useFloating();
  return (
    <>
      <div ref={refs.setReference} />
      <div ref={refs.setFloating} style={floatingStyles} />
    </>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: App,
  params: [{}],
  sequentialRenders: [{}, {}],
};
