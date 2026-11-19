function component(a) {
  let x = 'foo';
  if (a) {
    x = 'bar';
  } else {
    x = 'baz';
  }
  let y = x;
  mutate(y);
  return y;
}

import {mutate} from 'shared-runtime';
const sproutInputs = [
  [true],
  [false],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
