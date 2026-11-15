// @validateExhaustiveMemoizationDependencies:false
import {useMemo} from 'react';
function Component(props) {
  const x = useMemo(() => {
    label: {
      return props.value;
    }
  });
  return x;
}

const sproutInputs = [
  [{"value":1}],
  [{"value":11}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
