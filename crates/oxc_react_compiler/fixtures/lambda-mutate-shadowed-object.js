function Component() {
  const x = {};
  {
    const x = [];
    const fn = function () {
      mutate(x);
    };
    fn();
  }
  return x; // should return {}
}

import {mutate} from 'shared-runtime';
const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
