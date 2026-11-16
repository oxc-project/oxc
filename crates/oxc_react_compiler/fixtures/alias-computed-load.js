function component(a) {
  let x = {a};
  let y = {};

  y.x = x['a'];
  mutate(y);
  return x;
}

import {mutate} from 'shared-runtime';
const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
