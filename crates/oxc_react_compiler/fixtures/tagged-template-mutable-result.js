// @compilationMode:annotation
const shared = {
  value: 0,
  toString() {
    return this.value;
  },
};
function tag() {
  return shared;
}

function Component({tag}) {
  'use memo';
  const result = tag`value`;
  result.value++;
  return result + '';
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{tag}],
  sequentialRenders: [{tag}, {tag}],
};
