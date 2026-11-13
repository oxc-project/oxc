// @validateNoFreezingKnownMutableFunctions
import { useEffect } from 'react';

export function useAfterRun(run) {
  useEffect(() => {
    let timeoutId;
    const go = async () => {
      await run();
      timeoutId = setTimeout(() => {}, 10);
    };
    go();
    return () => clearTimeout(timeoutId);
  }, [run]);
}

const first = () => {
  'use no memo';
  return Promise.resolve();
};
const second = () => {
  'use no memo';
  return Promise.resolve();
};
export const FIXTURE_ENTRYPOINT = {
  fn: run => {
    'use no memo';
    return useAfterRun(run);
  },
  params: [first],
  sequentialRenders: [first, first, second, first],
};
