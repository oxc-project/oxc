// @validateNoImpureFunctionsInRender
import {useEffect, useState} from 'react';

// `last` is declared inside the effect, so reassigning it from the interval
// callback never crosses a render. The pragma matches Oxlint, which enables this
// validation.
function useTicker(isPlaying) {
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
