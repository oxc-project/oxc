// @validateNoFreezingKnownMutableFunctions
import { useMemo, useSyncExternalStore } from 'react';

const subscribe = () => () => {};
const serverSnapshot = () => false;

export function useValue(read) {
  let renderLocal;
  const getSnapshot = useMemo(() => {
    let cached;
    return () => {
      cached ??= read();
      renderLocal = cached;
      return cached;
    };
  }, [read]);
  return useSyncExternalStore(subscribe, getSnapshot, serverSnapshot);
}
