// @panicThreshold:"none"
// `const value = 1; value ??= 2` never assigns, so this must not be reported
// as a Syntax error. The conditional store is still a compiler limitation:
// when the store is taken, JavaScript throws, and lowering it as a
// reassignment would rewrite `const` to `let` and swallow that TypeError.
// A const closed over by a nested function is the same limitation — marking
// it reassigned would reject the declaration as Syntax.
function Component() {
  const value = 1;
  value ??= 2;
  return <div>{value}</div>;
}

function ComponentClosedOver() {
  const value = 1;
  const assign = () => {
    value &&= 2;
    return value;
  };
  return <div>{assign()}</div>;
}
