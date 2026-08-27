import {useMemo} from 'react';

function Component({value}) {
  const memo = useMemo(() => value, [value]);
  const read = () => {
    const object = {
      get value() {
        return value;
      },
    };
    return object.value;
  };
  return <div>{memo + read()}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
  sequentialRenders: [{value: 1}, {value: 2}],
};
