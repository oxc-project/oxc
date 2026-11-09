// @validateNoFreezingKnownMutableFunctions
import { useCallback } from 'react';

export function useCounter() {
  return useCallback(count => {
    return () => count++;
  }, []);
}
