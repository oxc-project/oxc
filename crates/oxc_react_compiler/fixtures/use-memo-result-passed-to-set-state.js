import {useMemo, useState} from 'react';

// A value passed to a state setter comes back from `useState`, so it must stay
// memoized for the comparison to settle instead of re-rendering forever.
function Component({a, b}) {
  const next = useMemo(() => ({a, b}), [a, b]);
  const [shown, setShown] = useState(null);
  if (next !== shown) {
    setShown(next);
  }
  return (
    <div>
      {shown?.a}|{shown?.b}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{a: 1, b: 2}],
  sequentialRenders: [
    {a: 1, b: 2},
    {a: 1, b: 2},
    {a: 3, b: 2},
  ],
};
