// Whether `validate` throws depends on a prop, so the value the `catch` assigns is
// reactive even though both assignments are constants.
function validate(value) {
  if (!value) {
    throw new Error('invalid');
  }
}

function Component({value}) {
  let status = 'valid';
  try {
    validate(value);
  } catch {
    status = 'invalid';
  }
  return <div>{status}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
  sequentialRenders: [{value: 1}, {value: 0}, {value: 1}],
};
