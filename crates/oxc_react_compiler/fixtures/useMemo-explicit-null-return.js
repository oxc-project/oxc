// @validateNoVoidUseMemo
function Component() {
  const value = useMemo(() => {
    return null;
  }, []);
  return <div>{value}</div>;
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
