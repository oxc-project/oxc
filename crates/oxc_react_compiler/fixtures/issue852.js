function Component(c) {
  let x = {c};
  mutate(x);
  let a = x;
  let b = a;
}

import {mutate} from 'shared-runtime';
const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
