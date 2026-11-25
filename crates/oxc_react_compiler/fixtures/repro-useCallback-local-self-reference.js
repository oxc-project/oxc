// @validateExhaustiveMemoizationDependencies
function Component() {
  useCallback(() => {
    const bar = () => {
      console.log(bar);
    };
  }, []);
}

import {useCallback} from 'react';
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
  sequentialRenders: sproutInputs,
};
