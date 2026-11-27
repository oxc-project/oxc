// @script
// A call as the target of a compound assignment (sloppy mode only).
function Component(props) {
  f() += props.a;
  return null;
}
