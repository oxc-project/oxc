// @script
// Unlike the TS implementation, which fails an invariant, a call as a for-in head
// (sloppy mode only) reports the `lowerAssignment` default-arm Todo.
function Component(props) {
  for (f() in props) {}
  return null;
}
