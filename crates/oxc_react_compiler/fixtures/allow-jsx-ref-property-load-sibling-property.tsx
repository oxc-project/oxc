// @validateRefAccessDuringRender @compilationMode:"infer"

// Passing `props.inputRef` to a JSX `ref` must not make `props.value` a ref value.
export function Example(props) {
  return <input ref={props.inputRef} value={props.value} readOnly />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Example,
  params: [{inputRef: null, value: 'a'}],
  sequentialRenders: [
    {inputRef: null, value: 'a'},
    {inputRef: null, value: 'a'},
    {inputRef: null, value: 'b'},
  ],
};
