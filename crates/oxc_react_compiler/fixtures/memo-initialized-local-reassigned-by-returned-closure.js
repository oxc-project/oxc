// @validateNoFreezingKnownMutableFunctions
import { useMemo, useSyncExternalStore } from 'react';

const subscribe = () => () => {};
const serverSnapshot = () => false;

export function useValue(read) {
  const getSnapshot = useMemo(() => {
    let cached = null;
    return () => cached ?? (cached = read());
  }, [read]);
  return useSyncExternalStore(subscribe, getSnapshot, serverSnapshot);
}

const first = () => {
  'use no memo';
  return { value: 1 };
};
const second = () => {
  'use no memo';
  return { value: 2 };
};
export const FIXTURE_ENTRYPOINT = {
  fn: read => {
    'use no memo';
    return useValue(read);
  },
  params: [first],
  sequentialRenders: [first, first, second, second, first],
};
