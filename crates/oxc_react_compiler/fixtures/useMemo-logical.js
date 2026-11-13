// @validateExhaustiveMemoizationDependencies:false
import {useMemo} from 'react';
function Component(props) {
  const x = useMemo(() => props.a && props.b);
  return x;
}

const sproutInputs = [
  [{"a":1,"b":2}],
  [{"a":11,"b":12}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
