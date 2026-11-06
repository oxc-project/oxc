// @validateExhaustiveEffectDependencies:"all" @outputMode:"lint" @panicThreshold:"none" @lintSkipReactiveScopePasses
import {useEffect, useRef, useState} from 'react';

// Without useMemo/useCallback, lint skips the reactive-scope passes but still
// infers reactive places for the effect: props are required dependencies,
// while the state setter and the ref are stable and may be omitted.
function Component({value, other}) {
  const [count, setCount] = useState(0);
  const ref = useRef(null);
  useEffect(() => {
    setCount(value);
    ref.current = other;
  }, []);
  return <div>{count}</div>;
}
