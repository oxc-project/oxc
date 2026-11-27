// @validateNoFreezingKnownMutableFunctions
import { useCallback } from 'react';

export function useCounter() {
  return useCallback(count => {
    return () => count++;
  }, []);
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useCounter()(3)();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
