// Constant assignments in the branches of an `if` inside `try` make the
// variable reactive through the `if`'s test.
function Component({cond}) {
  let value;
  try {
    if (cond) {
      value = 'a';
    } else {
      value = 'b';
    }
  } catch {
    value = 'c';
  }
  return <div>{value}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{cond: true}],
  sequentialRenders: [{cond: true}, {cond: false}, {cond: true}],
};
