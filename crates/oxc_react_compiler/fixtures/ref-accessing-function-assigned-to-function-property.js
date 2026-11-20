import {useMemo, useRef} from 'react';

// Both functions access the ref at different locations and `cancel` is stored
// as a property of `debounced`. Ref validation must still converge.
function useCancellableDebounce(fn, time) {
  const timeoutRef = useRef(null);
  return useMemo(() => {
    const cancel = () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
    const debounced = (...args) => {
      cancel();
      timeoutRef.current = setTimeout(() => fn(...args), time);
    };
    debounced.cancel = cancel;
    return debounced;
  }, [fn, time]);
}

function log() {}

function Component({time}) {
  const debounced = useCancellableDebounce(log, time);
  return <div>{typeof debounced.cancel}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{time: 100}],
  sequentialRenders: [{time: 100}, {time: 100}, {time: 200}],
};
