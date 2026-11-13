// @validateExhaustiveMemoizationDependencies:false
import {useMemo} from 'react';
function Component(props) {
  const x = useMemo(() => {
    switch (props.key) {
      case 'key': {
        return props.value;
      }
      default: {
        return props.defaultValue;
      }
    }
  });
  return x;
}

const sproutInputs = [
  [{key:"key",value:1,defaultValue:2}],
  [{key:"other",value:3,defaultValue:4}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
