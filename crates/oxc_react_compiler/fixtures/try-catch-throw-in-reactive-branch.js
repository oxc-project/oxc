// `fail` only runs, and throws, when a prop is set, so the value the `catch` assigns
// is reactive through that `if`.
function fail() {
  throw new Error('failed');
}

function Component({strict}) {
  let status = 'ok';
  try {
    if (strict) {
      fail();
    }
    status = 'passed';
  } catch {
    status = 'failed';
  }
  return <div>{status}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{strict: false}],
  sequentialRenders: [{strict: false}, {strict: true}, {strict: false}],
};
