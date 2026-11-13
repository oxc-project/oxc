import {arrayPush, mutate} from 'shared-runtime';

function Component({cond, value}) {
  const object = {value: cond ? 0 : []};
  mutate(object);
  const current = object.value;
  return typeof current === 'number'
    ? current - 1
    : arrayPush(current, value);
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
  sequentialRenders: [
    {cond: false, value: 2},
    {cond: false, value: 3},
    {cond: true, value: 4},
  ],
};
