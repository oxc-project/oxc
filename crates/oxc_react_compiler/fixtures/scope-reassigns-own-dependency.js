// The block that builds `notes` depends on `n` and reassigns it, so the cache
// key must be the value of `n` before the block runs.
function Component({start}) {
  let n = start;
  const notes = [];
  if (n === 0) {
    notes.push('was zero');
    n = 1;
  }
  return (
    <div>
      {n}: {notes.join()}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{start: 0}],
  sequentialRenders: [{start: 0}, {start: 1}, {start: 1}, {start: 0}],
};
