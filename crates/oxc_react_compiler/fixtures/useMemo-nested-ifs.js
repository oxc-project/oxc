// @validateExhaustiveMemoizationDependencies:false
import {useMemo} from 'react';
function Component(props) {
  const x = useMemo(() => {
    if (props.cond) {
      if (props.cond) {
        return props.value;
      }
    }
  }, [props.cond]);
  return x;
}

const sproutInputs = [
  [{"cond":false,"value":2}],
  [{"cond":true,"value":12}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
