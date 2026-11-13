import {useMemo} from 'react';
function component(a, b) {
  let x = useMemo(() => {
    if (a) {
      return {b};
    }
  }, [a, b]);
  return x;
}

const sproutInputs = [
  [true,1],
  [false,2],
  [true,3],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
