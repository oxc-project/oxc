import {arrayPush, mutate} from 'shared-runtime';

/**
 * A binary expression may coerce an object operand to a primitive without making
 * every use of that value primitive. Reusing the array across renders would return
 * [2] and then [2, 3], instead of [2] and [3].
 */
function useFoo({value}) {
  const x = {value: []};
  mutate(x);

  const xValue = x.value;
  let result;
  if (typeof xValue === 'number') {
    result = xValue + 1;
  } else {
    result = arrayPush(xValue, value);
  }
  return result;
}

export const FIXTURE_ENTRYPOINT = {
  fn: useFoo,
  params: [{value: 1}],
  sequentialRenders: [{value: 2}, {value: 3}],
};
