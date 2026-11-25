// @compilationMode:"infer" @panicThreshold:"none" @validateExhaustiveMemoizationDependencies
import {useMemo} from 'react';

export function Component({status, tick}) {
  const value = useMemo(() => {
    void tick;
    return status.expiresAt;
  }, [tick, status.expiresAt]);
  return <div>{value}</div>;
}
