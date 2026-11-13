// @validatePreserveExistingMemoizationGuarantees
import {useCallback} from 'react';

function useHook(maybeRef) {
  return useCallback(() => {
    return [maybeRef.current];
  }, [maybeRef]);
}


const sproutInputs = [
  [{current:1}],
  [{current:2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useHook(args[0])();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
