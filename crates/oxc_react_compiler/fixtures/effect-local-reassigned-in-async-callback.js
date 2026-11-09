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
