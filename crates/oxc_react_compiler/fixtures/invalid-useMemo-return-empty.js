// @loggerTestOnly @validateExhaustiveMemoizationDependencies:false
function component(a) {
  let x = useMemo(() => {
    mutate(a);
  }, []);
  return x;
}

import {useMemo} from 'react';
import {mutate} from 'shared-runtime';
const sproutInputs = [
  [{}],
  [{value:1}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
