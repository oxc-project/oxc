// @compilationMode:annotation
function tag(strings, value) {
  return {value};
}

function Component({value}) {
  'use memo';
  const result = tag`${value}`;
  result.value++;
  return result.value;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
  sequentialRenders: [{value: 1}, {value: 1}],
};
