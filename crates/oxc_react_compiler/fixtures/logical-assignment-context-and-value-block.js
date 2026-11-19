import {Stringify} from 'shared-runtime';

// A logical assignment to a variable a closure captures, and one nested in
// another value block.
function Component({value, flag}) {
  let cached = value;
  const read = () => cached;
  cached ??= 'fallback';
  let label = value;
  const result = flag ? (label ||= 'default') : 'none';
  return <Stringify cached={read()} label={label} result={result} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: null, flag: true}],
};
