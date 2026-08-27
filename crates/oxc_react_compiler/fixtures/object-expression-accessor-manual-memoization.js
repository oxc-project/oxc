import {useCallback, useMemo} from 'react';

function Component({value}) {
  const callback = useCallback(() => value, [value]);
  const memo = useMemo(() => ({value}), [value]);
  const object = {
    get value() {
      return memo.value;
    },
  };
  return <button onClick={callback}>{object.value}</button>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 1}, {value: 2}],
};
