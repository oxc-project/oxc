// Constant assignments in the cases of a `switch` inside `try` make the
// variable reactive through the `switch`'s test.
function Component({kind}) {
  let value;
  try {
    switch (kind) {
      case 1:
        value = 'one';
        break;
      default:
        value = 'other';
    }
  } catch {
    value = 'caught';
  }
  return <div>{value}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{kind: 1}],
  sequentialRenders: [{kind: 1}, {kind: 2}, {kind: 1}],
};
