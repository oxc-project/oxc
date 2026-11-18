// @validateRefAccessDuringRender @compilationMode:"infer"

export function Button(props) {
  return <button ref={props.ref} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Button,
  params: [{ref: null}],
  sequentialRenders: [{ref: null}, {ref: null}, {ref: {current: null}}],
};
