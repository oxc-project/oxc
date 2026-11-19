// @validateNoFreezingKnownMutableFunctions @enableNewMutationAliasingModel
import {useCallback, useEffect, useRef} from 'react';
import {useHook} from 'shared-runtime';

// This was a false positive "can't freeze mutable function" in the old
// inference model, fixed in the new inference model.
function Component() {
  const params = useHook();
  const update = useCallback(
    partialParams => {
      const nextParams = {
        ...params,
        ...partialParams,
      };
      nextParams.param = 'value';
      console.log(nextParams);
    },
    [params]
  );
  const ref = useRef(null);
  useEffect(() => {
    if (ref.current === null) {
      update();
    }
  }, [update]);

  return 'ok';
}


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
  sequentialRenders: sproutInputs,
};
