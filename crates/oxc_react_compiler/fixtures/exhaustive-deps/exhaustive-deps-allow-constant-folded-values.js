// @validateExhaustiveMemoizationDependencies

function Component() {
  const x = 0;
  const y = useMemo(() => {
    return [x];
    // x gets constant-folded but shouldn't count as extraneous,
    // it was referenced in the memo block
  }, [x]);
  return y;
}

import {useMemo} from 'react';
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
