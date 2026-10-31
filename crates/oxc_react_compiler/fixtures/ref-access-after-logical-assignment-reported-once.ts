// @outputMode:"lint" @panicThreshold:"none"
import {useEffect, useRef} from 'react';

const clearOwned = (owner: symbol): void => {
  console.log(owner);
};

// The read of `ownerRef.current` during render is reported once.
export const useOwner = (items: number[], index: number, count: number): void => {
  const ownerRef = useRef<symbol | null>(null);
  ownerRef.current ??= Symbol('owner');
  const owner = ownerRef.current;

  useEffect(() => {
    const item = items[index];
    if (count === 0 || item === undefined) {
      clearOwned(owner);
      return;
    }
    console.log(item, owner);
  }, [index, items, count, owner]);

  useEffect(() => () => clearOwned(owner), [owner]);
};
