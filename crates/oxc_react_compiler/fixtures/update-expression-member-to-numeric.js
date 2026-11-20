// `++` and `--` on a member convert the value with ToNumeric, so a string
// becomes a number and a BigInt stays a BigInt instead of `value + 1`.
function Component({initial}) {
  const counter = {current: initial};
  const items = [initial];
  const before = counter.current++;
  const after = ++counter.current;
  counter.current--;
  items[0]++;
  return (
    <div>
      {typeof before}:{String(before)}|{typeof after}:{String(after)}|
      {String(counter.current)}|{String(items[0])}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{initial: '5'}],
  sequentialRenders: [{initial: '5'}, {initial: 5n}, {initial: 1.5}],
};
