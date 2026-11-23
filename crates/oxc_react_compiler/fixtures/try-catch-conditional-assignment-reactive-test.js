// A let assigned from a ternary with constant branches inside `try` is reactive
// through the ternary's test, so the JSX that reads it must depend on it.
function Component({kind}) {
  let label = 'init';
  try {
    label = kind === 1 ? 'one' : 'other';
  } catch {
    label = 'caught';
  }
  return <div>{label}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{kind: 1}],
  sequentialRenders: [{kind: 1}, {kind: 2}, {kind: 1}],
};
