// @validateRefAccessDuringRender @compilationMode:"infer"

export function Button(props) {
  return <button ref={props.ref} />;
}
