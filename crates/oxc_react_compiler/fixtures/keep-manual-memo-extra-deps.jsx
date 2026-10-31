// @compilationMode:"infer" @panicThreshold:"none"
// `tick` is not read by the callback but must still recompute the value when it changes.
import { useMemo } from 'react';

function Clock({ tick, label }) {
  const now = useMemo(() => readGlobal(), [tick]);
  return <span title={label}>{now}</span>;
}
