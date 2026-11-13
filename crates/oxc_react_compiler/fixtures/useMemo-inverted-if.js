// @validateExhaustiveMemoizationDependencies:false
import {useMemo} from 'react';
function Component(props) {
  const x = useMemo(() => {
    label: {
      if (props.cond) {
        break label;
      }
      return props.a;
    }
    return props.b;
  });
  return x;
}

const sproutInputs = [
  [{"cond":false,"a":2,"b":3}],
  [{"cond":true,"a":12,"b":13}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
