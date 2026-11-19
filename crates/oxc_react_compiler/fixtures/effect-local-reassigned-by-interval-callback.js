// @validateNoFreezingKnownMutableFunctions
import { useEffect, useState } from 'react';

export function useTicker(isPlaying) {
  const [total, setTotal] = useState(0);
  useEffect(() => {
    if (!isPlaying) return;
    let last = Date.now();
    const id = setInterval(() => {
      const now = Date.now();
      setTotal(t => t + (now - last));
      last = now;
    }, 1000);
    return () => clearInterval(id);
  }, [isPlaying]);
  return total;
}
